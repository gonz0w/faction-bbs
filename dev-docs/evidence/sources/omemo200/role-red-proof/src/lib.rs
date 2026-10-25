//! Disposable OMEMO-2 research proof. No product vault or transport integration.

use std::{ffi::c_void, ptr::NonNull, sync::Mutex};

pub const MAX_PLAINTEXT: usize = 8192;
pub const MAX_PAYLOAD: usize = MAX_PLAINTEXT + 16;
pub const MAX_KEY: usize = 256;
pub const MAX_SNAPSHOT: usize = 32768;
pub const MAX_LINE: usize = MAX_SNAPSHOT * 2 + 32;
static NATIVE_LOCK: Mutex<()> = Mutex::new(());

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Error;
pub type Result<T> = std::result::Result<T, Error>;

unsafe extern "C" {
    fn proof_private_pipes() -> i32;
    fn proof_alloc() -> *mut c_void;
    fn proof_clone(ctx: *const c_void) -> *mut c_void;
    fn proof_free(ctx: *mut c_void);
    fn proof_setup(ctx: *mut c_void) -> i32;
    fn proof_bundle(ctx: *const c_void, out: *mut u8, ids: *mut u32) -> i32;
    fn proof_trust(ctx: *mut c_void, pin: *const u8) -> i32;
    fn proof_start(ctx: *mut c_void, bundle: *const u8, ids: *const u32) -> i32;
    fn proof_seal(
        ctx: *mut c_void,
        p: *const u8,
        n: usize,
        key: *mut u8,
        keyn: *mut usize,
        prekey: *mut i32,
        payload: *mut u8,
        payloadn: *mut usize,
    ) -> i32;
    fn proof_open(
        ctx: *mut c_void,
        key: *const u8,
        keyn: usize,
        prekey: i32,
        payload: *const u8,
        payloadn: usize,
        out: *mut u8,
        outn: *mut usize,
    ) -> i32;
    fn proof_export(ctx: *const c_void, out: *mut u8, n: *mut usize) -> i32;
    fn proof_import(ctx: *mut c_void, snapshot: *const u8, n: usize) -> i32;
    fn proof_convert(out: *mut u8, ed: *const u8);
}

pub fn require_private_pipes() -> Result<()> {
    if unsafe { proof_private_pipes() } == 0 {
        Ok(())
    } else {
        Err(Error)
    }
}

pub fn wipe(bytes: &mut [u8]) {
    for byte in bytes {
        unsafe { std::ptr::write_volatile(byte, 0) };
    }
    std::sync::atomic::compiler_fence(std::sync::atomic::Ordering::SeqCst);
}
struct Secret(Vec<u8>);
impl Drop for Secret {
    fn drop(&mut self) {
        wipe(&mut self.0);
    }
}

#[derive(Clone, Debug)]
pub struct Bundle {
    pub identity: [u8; 32],
    pub signed_prekey: [u8; 32],
    pub signature: [u8; 64],
    pub signed_prekey_id: u32,
    pub prekey: [u8; 32],
    pub prekey_id: u32,
}
impl Bundle {
    fn packed(&self) -> [u8; 160] {
        let mut out = [0; 160];
        out[..32].copy_from_slice(&self.identity);
        out[32..64].copy_from_slice(&self.signed_prekey);
        out[64..128].copy_from_slice(&self.signature);
        out[128..].copy_from_slice(&self.prekey);
        out
    }
}
#[derive(Clone, Debug)]
pub struct Message {
    pub key: Vec<u8>,
    pub prekey: bool,
    pub payload: Vec<u8>,
}

/// Opaque native ownership; never expose C object layouts to Rust.
/// Every FFI call is serialized because upstream callbacks are global.
pub struct Device {
    ptr: NonNull<c_void>,
    pending: Option<NonNull<c_void>>,
}
impl Drop for Device {
    fn drop(&mut self) {
        let _guard = NATIVE_LOCK.lock().unwrap_or_else(|err| err.into_inner());
        unsafe {
            if let Some(pending) = self.pending.take() {
                proof_free(pending.as_ptr());
            }
            proof_free(self.ptr.as_ptr());
        }
    }
}
impl Device {
    fn empty() -> Result<Self> {
        let _guard = NATIVE_LOCK.lock().map_err(|_| Error)?;
        Ok(Self {
            ptr: NonNull::new(unsafe { proof_alloc() }).ok_or(Error)?,
            pending: None,
        })
    }
    fn ready(&self) -> Result<()> {
        if self.pending.is_some() {
            Err(Error)
        } else {
            Ok(())
        }
    }
    pub fn fresh() -> Result<Self> {
        let device = Self::empty()?;
        let result = {
            let _guard = NATIVE_LOCK.lock().map_err(|_| Error)?;
            unsafe { proof_setup(device.ptr.as_ptr()) }
        };
        if result == 0 { Ok(device) } else { Err(Error) }
    }
    pub fn bundle(&self) -> Result<Bundle> {
        self.ready()?;
        let mut out = [0; 160];
        let mut ids = [0; 2];
        let _guard = NATIVE_LOCK.lock().map_err(|_| Error)?;
        if unsafe { proof_bundle(self.ptr.as_ptr(), out.as_mut_ptr(), ids.as_mut_ptr()) } != 0 {
            return Err(Error);
        }
        Ok(Bundle {
            identity: out[..32].try_into().unwrap(),
            signed_prekey: out[32..64].try_into().unwrap(),
            signature: out[64..128].try_into().unwrap(),
            prekey: out[128..].try_into().unwrap(),
            signed_prekey_id: ids[0],
            prekey_id: ids[1],
        })
    }
    pub fn trust(&mut self, typed: &str) -> Result<()> {
        self.ready()?;
        let pin = decode(typed.strip_prefix("ed25519:").ok_or(Error)?, 32)?;
        if pin.len() != 32 {
            return Err(Error);
        }
        let _guard = NATIVE_LOCK.lock().map_err(|_| Error)?;
        if unsafe { proof_trust(self.ptr.as_ptr(), pin.as_ptr()) } == 0 {
            Ok(())
        } else {
            Err(Error)
        }
    }
    pub fn start(&mut self, bundle: &Bundle) -> Result<()> {
        self.ready()?;
        let _guard = NATIVE_LOCK.lock().map_err(|_| Error)?;
        if unsafe {
            proof_start(
                self.ptr.as_ptr(),
                bundle.packed().as_ptr(),
                [bundle.signed_prekey_id, bundle.prekey_id].as_ptr(),
            )
        } == 0
        {
            Ok(())
        } else {
            Err(Error)
        }
    }
    pub fn seal(&mut self, plain: &[u8]) -> Result<Message> {
        self.ready()?;
        if plain.len() > MAX_PLAINTEXT {
            return Err(Error);
        }
        let mut key = vec![0; MAX_KEY];
        let mut payload = vec![0; MAX_PAYLOAD];
        let mut keyn = 0;
        let mut payloadn = 0;
        let mut prekey = 0;
        let _guard = NATIVE_LOCK.lock().map_err(|_| Error)?;
        let result = unsafe {
            proof_seal(
                self.ptr.as_ptr(),
                plain.as_ptr(),
                plain.len(),
                key.as_mut_ptr(),
                &mut keyn,
                &mut prekey,
                payload.as_mut_ptr(),
                &mut payloadn,
            )
        };
        if result != 0 || keyn > MAX_KEY || payloadn > MAX_PAYLOAD {
            return Err(Error);
        }
        key.truncate(keyn);
        payload.truncate(payloadn);
        Ok(Message {
            key,
            prekey: prekey != 0,
            payload,
        })
    }
    /// Returns private candidate plaintext while retaining a staged native clone.
    /// The local fixture must validate SCE policy, then explicitly commit/abort.
    pub fn open_pending(&mut self, msg: &Message) -> Result<Vec<u8>> {
        self.ready()?;
        if msg.key.len() > MAX_KEY || msg.payload.len() > MAX_PAYLOAD {
            return Err(Error);
        }
        let mut output = Secret(vec![0; MAX_PAYLOAD]);
        let mut n = 0;
        let _guard = NATIVE_LOCK.lock().map_err(|_| Error)?;
        let candidate = NonNull::new(unsafe { proof_clone(self.ptr.as_ptr()) }).ok_or(Error)?;
        let result = unsafe {
            proof_open(
                candidate.as_ptr(),
                msg.key.as_ptr(),
                msg.key.len(),
                i32::from(msg.prekey),
                msg.payload.as_ptr(),
                msg.payload.len(),
                output.0.as_mut_ptr(),
                &mut n,
            )
        };
        if result != 0 || n > MAX_PLAINTEXT {
            unsafe { proof_free(candidate.as_ptr()) };
            return Err(Error);
        }
        self.pending = Some(candidate);
        output.0.truncate(n);
        Ok(std::mem::take(&mut output.0))
    }
    pub fn commit(&mut self) -> Result<()> {
        let _guard = NATIVE_LOCK.lock().map_err(|_| Error)?;
        let candidate = self.pending.take().ok_or(Error)?;
        unsafe { proof_free(self.ptr.as_ptr()) };
        self.ptr = candidate;
        Ok(())
    }
    pub fn abort(&mut self) -> Result<()> {
        let _guard = NATIVE_LOCK.lock().map_err(|_| Error)?;
        let candidate = self.pending.take().ok_or(Error)?;
        unsafe { proof_free(candidate.as_ptr()) };
        Ok(())
    }
    /// Crypto-only test helper; the line protocol always uses open_pending.
    #[cfg(test)]
    fn open(&mut self, msg: &Message) -> Result<Vec<u8>> {
        let output = self.open_pending(msg)?;
        self.commit()?;
        Ok(output)
    }
    /// Private synthetic snapshot. This is not encrypted persistent storage.
    pub fn export(&self) -> Result<Vec<u8>> {
        self.ready()?;
        let mut out = Secret(vec![0; MAX_SNAPSHOT]);
        let mut n = 0;
        let _guard = NATIVE_LOCK.lock().map_err(|_| Error)?;
        let result = unsafe { proof_export(self.ptr.as_ptr(), out.0.as_mut_ptr(), &mut n) };
        if result != 0 || n > MAX_SNAPSHOT {
            return Err(Error);
        }
        out.0.truncate(n);
        Ok(std::mem::take(&mut out.0))
    }
    pub fn import(snapshot: &[u8]) -> Result<Self> {
        if snapshot.len() > MAX_SNAPSHOT {
            return Err(Error);
        }
        let device = Self::empty()?;
        let result = {
            let _guard = NATIVE_LOCK.lock().map_err(|_| Error)?;
            unsafe { proof_import(device.ptr.as_ptr(), snapshot.as_ptr(), snapshot.len()) }
        };
        if result == 0 { Ok(device) } else { Err(Error) }
    }
}

/// Reviewed upstream conversion function; this does not validate Ed point encoding.
pub fn convert(ed: &[u8; 32]) -> [u8; 32] {
    let _guard = NATIVE_LOCK.lock().unwrap_or_else(|err| err.into_inner());
    let mut out = [0; 32];
    unsafe { proof_convert(out.as_mut_ptr(), ed.as_ptr()) };
    out
}
pub fn encode(bytes: &[u8]) -> String {
    if bytes.is_empty() {
        return "-".into();
    }
    const HEX: &[u8] = b"0123456789abcdef";
    let mut text = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        text.push(HEX[(b >> 4) as usize] as char);
        text.push(HEX[(b & 15) as usize] as char);
    }
    text
}
pub fn decode(text: &str, max: usize) -> Result<Vec<u8>> {
    if text == "-" {
        return Ok(Vec::new());
    }
    if text.is_empty() || text.len() > max * 2 || text.len() % 2 != 0 {
        return Err(Error);
    }
    fn digit(b: u8) -> Result<u8> {
        match b {
            b'0'..=b'9' => Ok(b - b'0'),
            b'a'..=b'f' => Ok(b - b'a' + 10),
            _ => Err(Error),
        }
    }
    text.as_bytes()
        .chunks_exact(2)
        .map(|pair| Ok(digit(pair[0])? * 16 + digit(pair[1])?))
        .collect()
}
fn fixed<const N: usize>(text: &str) -> Result<[u8; N]> {
    decode(text, N)?.try_into().map_err(|_| Error)
}
fn id(text: &str) -> Result<u32> {
    if text.is_empty() || text.len() > 10 || !text.bytes().all(|b| b.is_ascii_digit()) {
        return Err(Error);
    }
    let n = text.parse().map_err(|_| Error)?;
    if n == 0 || n > 0x7fffffff {
        Err(Error)
    } else {
        Ok(n)
    }
}
fn ok_hex(bytes: &[u8]) -> String {
    // Build secret-bearing responses directly into the buffer main wipes.
    let mut out = String::with_capacity(3 + bytes.len() * 2);
    out.push_str("ok ");
    if bytes.is_empty() {
        out.push('-');
    } else {
        const HEX: &[u8] = b"0123456789abcdef";
        for b in bytes {
            out.push(HEX[(b >> 4) as usize] as char);
            out.push(HEX[(b & 15) as usize] as char);
        }
    }
    out
}
fn bundle_line(bundle: &Bundle) -> String {
    format!(
        "ok {} {} {} {} {} {}",
        encode(&bundle.identity),
        encode(&bundle.signed_prekey),
        encode(&bundle.signature),
        bundle.signed_prekey_id,
        encode(&bundle.prekey),
        bundle.prekey_id
    )
}
fn command(state: &mut Option<Device>, line: &str) -> Result<String> {
    if line.len() > MAX_LINE || line.bytes().any(|b| b < 0x20 || b > 0x7e) {
        return Err(Error);
    }
    // At most seven tokens; reject repeated spaces and ambiguous framing.
    let tokens: Vec<_> = line.split(' ').take(8).collect();
    if tokens.len() > 7 || tokens.iter().any(|s| s.is_empty()) {
        return Err(Error);
    }
    if state
        .as_ref()
        .is_some_and(|device| device.pending.is_some())
        && !matches!(tokens[0], "commit" | "abort")
    {
        return Err(Error);
    }
    match tokens.as_slice() {
        ["new"] if state.is_none() => {
            let device = Device::fresh()?;
            let reply = bundle_line(&device.bundle()?);
            *state = Some(device);
            Ok(reply)
        }
        ["import", snapshot] if state.is_none() => {
            let bytes = Secret(decode(snapshot, MAX_SNAPSHOT)?);
            *state = Some(Device::import(&bytes.0)?);
            Ok("ok".into())
        }
        ["convert", ed] => Ok(format!("ok {}", encode(&convert(&fixed(ed)?)))),
        ["trust", pin] => {
            state.as_mut().ok_or(Error)?.trust(pin)?;
            Ok("ok".into())
        }
        ["start", ik, spk, sig, spkid, pk, pkid] => {
            let bundle = Bundle {
                identity: fixed(ik)?,
                signed_prekey: fixed(spk)?,
                signature: fixed(sig)?,
                signed_prekey_id: id(spkid)?,
                prekey: fixed(pk)?,
                prekey_id: id(pkid)?,
            };
            state.as_mut().ok_or(Error)?.start(&bundle)?;
            Ok("ok".into())
        }
        ["seal", plain] => {
            let bytes = Secret(decode(plain, MAX_PLAINTEXT)?);
            let message = state.as_mut().ok_or(Error)?.seal(&bytes.0)?;
            Ok(format!(
                "ok {} {} {}",
                encode(&message.key),
                u8::from(message.prekey),
                encode(&message.payload)
            ))
        }
        ["open", key, prekey, payload] => {
            let prekey = match *prekey {
                "0" => false,
                "1" => true,
                _ => return Err(Error),
            };
            let message = Message {
                key: decode(key, MAX_KEY)?,
                prekey,
                payload: decode(payload, MAX_PAYLOAD)?,
            };
            let plaintext = Secret(state.as_mut().ok_or(Error)?.open_pending(&message)?);
            Ok(ok_hex(&plaintext.0))
        }
        ["commit"] => {
            state.as_mut().ok_or(Error)?.commit()?;
            Ok("ok".into())
        }
        ["abort"] => {
            state.as_mut().ok_or(Error)?.abort()?;
            Ok("ok".into())
        }
        ["export"] => {
            let snapshot = Secret(state.as_ref().ok_or(Error)?.export()?);
            Ok(ok_hex(&snapshot.0))
        }
        _ => Err(Error),
    }
}
/// No error diagnostics include input, key material, or tentative plaintext.
pub fn dispatch(state: &mut Option<Device>, line: &str) -> String {
    command(state, line).unwrap_or_else(|_| "err".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn paired() -> (Device, Device) {
        let mut a = Device::fresh().unwrap();
        let mut b = Device::fresh().unwrap();
        a.trust(&format!(
            "ed25519:{}",
            encode(&b.bundle().unwrap().identity)
        ))
        .unwrap();
        b.trust(&format!(
            "ed25519:{}",
            encode(&a.bundle().unwrap().identity)
        ))
        .unwrap();
        a.start(&b.bundle().unwrap()).unwrap();
        (a, b)
    }

    #[test]
    fn generated_library_first_message_and_reply() {
        let (mut a, mut b) = paired();
        let first = a
            .seal(b"<envelope xmlns='urn:xmpp:sce:1'>first</envelope>")
            .unwrap();
        assert!(first.prekey);
        assert_eq!(
            b.open(&first).unwrap(),
            b"<envelope xmlns='urn:xmpp:sce:1'>first</envelope>"
        );
        let reply = b.seal(b"reply").unwrap();
        assert!(!reply.prekey);
        assert_eq!(a.open(&reply).unwrap(), b"reply");
    }

    #[test]
    fn explicit_ed25519_pin_required_and_immutable() {
        let mut a = Device::fresh().unwrap();
        let b = Device::fresh().unwrap();
        assert!(a.start(&b.bundle().unwrap()).is_err());
        assert!(a.seal(b"unverified").is_err());
        assert!(
            a.trust(&format!(
                "curve25519:{}",
                encode(&b.bundle().unwrap().identity)
            ))
            .is_err()
        );
        assert!(a.trust("ed25519:00").is_err());
        let pin = format!("ed25519:{}", encode(&b.bundle().unwrap().identity));
        a.trust(&pin).unwrap();
        a.trust(&pin).unwrap();
        assert!(
            a.trust(&format!(
                "ed25519:{}",
                encode(&a.bundle().unwrap().identity)
            ))
            .is_err()
        );
        let mut changed = b.bundle().unwrap();
        changed.identity[31] ^= 0x80;
        assert!(a.start(&changed).is_err());
    }

    #[test]
    fn independent_edwards_to_montgomery_vectors_and_sign_pin() {
        for (ed, curve) in [
            (
                "d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a",
                "d85e07ec22b0ad881537c2f44d662d1a143cf830c57aca4305d85c7a90f6b62e",
            ),
            (
                "03a107bff3ce10be1d70dd18e74bc09967e4d6309ba50d5f1ddc8664125531b8",
                "4701d08488451f545a409fb58ae3e58581ca40ac3f7f114698cd71deac73ca01",
            ),
        ] {
            assert_eq!(
                encode(&convert(&decode(ed, 32).unwrap().try_into().unwrap())),
                curve
            );
        }
        let (mut a, mut b) = paired();
        let mut wrong_pin = a.bundle().unwrap().identity;
        wrong_pin[31] ^= 0x80;
        // A new receiver explicitly enrolled under the wrong Ed sign must fail
        // even though the two Ed encodings map to the same Curve25519 point.
        let snapshot = b.export().unwrap();
        let mut other = Device::import(&snapshot).unwrap();
        assert!(
            other
                .trust(&format!("ed25519:{}", encode(&wrong_pin)))
                .is_err()
        );
        let mut receiver = Device::fresh().unwrap();
        let mut sender = Device::fresh().unwrap();
        let mut wrong = sender.bundle().unwrap().identity;
        wrong[31] ^= 0x80;
        receiver
            .trust(&format!("ed25519:{}", encode(&wrong)))
            .unwrap();
        sender
            .trust(&format!(
                "ed25519:{}",
                encode(&receiver.bundle().unwrap().identity)
            ))
            .unwrap();
        sender.start(&receiver.bundle().unwrap()).unwrap();
        let before = receiver.export().unwrap();
        assert!(
            receiver
                .open(&sender.seal(b"wrong Ed sign").unwrap())
                .is_err()
        );
        assert_eq!(before, receiver.export().unwrap());
        let msg = a.seal(b"typed pin").unwrap();
        assert_eq!(b.open(&msg).unwrap(), b"typed pin");
    }

    #[test]
    fn altered_signature_and_wrong_bundle_reject_without_state_change() {
        let (mut a, mut b) = paired();
        let before = b.export().unwrap();
        let mut bundle = a.bundle().unwrap();
        bundle.signature[0] ^= 1;
        assert!(b.start(&bundle).is_err());
        assert_eq!(before, b.export().unwrap());
        let third = Device::fresh().unwrap();
        assert!(b.start(&third.bundle().unwrap()).is_err());
        assert_eq!(before, b.export().unwrap());
        assert!(a.seal(&vec![0; MAX_PLAINTEXT + 1]).is_err());
    }

    #[test]
    fn payload_failure_and_truncation_roll_back_entire_context() {
        let (mut a, mut b) = paired();
        let msg = a.seal(b"authenticate before output").unwrap();
        for change in 0..4 {
            let before = b.export().unwrap();
            let mut bad = msg.clone();
            match change {
                0 => bad.payload[0] ^= 1,
                1 => {
                    bad.payload.pop();
                }
                2 => bad.key[0] ^= 1,
                _ => {
                    bad.key.truncate(5);
                }
            }
            assert!(b.open(&bad).is_err());
            assert_eq!(before, b.export().unwrap());
        }
        assert_eq!(b.open(&msg).unwrap(), b"authenticate before output");
        let before = b.export().unwrap();
        assert!(b.open(&msg).is_err());
        assert_eq!(before, b.export().unwrap());
    }

    #[test]
    fn wrong_inbound_pin_and_wrong_payload_key_never_commit() {
        let mut a = Device::fresh().unwrap();
        let mut b = Device::fresh().unwrap();
        let c = Device::fresh().unwrap();
        a.trust(&format!(
            "ed25519:{}",
            encode(&b.bundle().unwrap().identity)
        ))
        .unwrap();
        a.start(&b.bundle().unwrap()).unwrap();
        b.trust(&format!(
            "ed25519:{}",
            encode(&c.bundle().unwrap().identity)
        ))
        .unwrap();
        let msg = a.seal(b"wrong identity").unwrap();
        let before = b.export().unwrap();
        assert!(b.open(&msg).is_err());
        assert_eq!(before, b.export().unwrap());
        let (mut a, mut b) = paired();
        let one = a.seal(b"one").unwrap();
        let two = a.seal(b"two").unwrap();
        let mut mixed = one.clone();
        mixed.payload = two.payload;
        assert!(b.open(&mixed).is_err());
        assert_eq!(b.open(&one).unwrap(), b"one");
    }

    #[test]
    fn skipped_keys_survive_restart_and_are_consumed_once() {
        let (mut a, mut b) = paired();
        let one = a.seal(b"one").unwrap();
        let two = a.seal(b"two").unwrap();
        let three = a.seal(b"three").unwrap();
        let mut tampered = three.clone();
        tampered.payload[0] ^= 1;
        let before = b.export().unwrap();
        assert!(b.open(&tampered).is_err());
        assert_eq!(before, b.export().unwrap());
        assert_eq!(b.open(&three).unwrap(), b"three");
        let mut b = Device::import(&b.export().unwrap()).unwrap();
        assert_eq!(b.open(&one).unwrap(), b"one");
        assert!(b.open(&one).is_err());
        assert_eq!(b.open(&two).unwrap(), b"two");
        let reply = b.seal(b"restart reply").unwrap();
        let mut a = Device::import(&a.export().unwrap()).unwrap();
        assert_eq!(a.open(&reply).unwrap(), b"restart reply");
    }

    #[test]
    fn max_skip_and_aggregate_cache_are_bounded_with_rollback() {
        let (mut a, mut b) = paired();
        let mut msgs = Vec::new();
        for _ in 0..34 {
            msgs.push(a.seal(b"bounded").unwrap());
        }
        let before = b.export().unwrap();
        assert!(b.open(&msgs[33]).is_err());
        assert_eq!(before, b.export().unwrap());
        assert_eq!(b.open(&msgs[32]).unwrap(), b"bounded");
        let late = a.seal(b"over aggregate").unwrap();
        let before = b.export().unwrap();
        assert!(b.open(&late).is_err());
        assert_eq!(before, b.export().unwrap());
        assert_eq!(b.open(&msgs[0]).unwrap(), b"bounded");
        assert_eq!(b.open(&late).unwrap(), b"over aggregate");
    }

    #[test]
    fn snapshot_rejects_truncation_trailing_bytes_and_invalid_header() {
        let (a, _) = paired();
        let snapshot = a.export().unwrap();
        assert_eq!(
            Device::import(&snapshot).unwrap().export().unwrap(),
            snapshot
        );
        for n in [0, 1, 8, 12, snapshot.len() - 1] {
            assert!(Device::import(&snapshot[..n]).is_err());
        }
        let mut bad = snapshot.clone();
        bad.push(0);
        assert!(Device::import(&bad).is_err());
        let mut bad = snapshot.clone();
        bad[0] ^= 1;
        assert!(Device::import(&bad).is_err());
        assert!(Device::import(&vec![0; MAX_SNAPSHOT + 1]).is_err());
    }

    #[test]
    fn empty_and_maximum_plaintext_roundtrip() {
        let (mut a, mut b) = paired();
        for plaintext in [Vec::new(), vec![42; MAX_PLAINTEXT]] {
            assert_eq!(b.open(&a.seal(&plaintext).unwrap()).unwrap(), plaintext);
        }
    }

    #[test]
    fn pending_sce_acceptance_commits_or_aborts_whole_transaction() {
        let (mut a, mut b) = paired();
        let msg = a.seal(b"SCE awaiting local policy").unwrap();
        let before = b.export().unwrap();
        assert_eq!(b.open_pending(&msg).unwrap(), b"SCE awaiting local policy");
        assert!(b.export().is_err());
        assert!(b.seal(b"blocked pending").is_err());
        assert!(b.open(&msg).is_err());
        b.abort().unwrap();
        assert_eq!(before, b.export().unwrap());
        assert_eq!(b.open_pending(&msg).unwrap(), b"SCE awaiting local policy");
        b.commit().unwrap();
        assert!(b.open_pending(&msg).is_err());
        assert!(b.commit().is_err());
    }

    #[test]
    fn command_state_and_decoding_rejects_without_secret_output() {
        let mut state = None;
        for line in [
            "",
            "new extra",
            "open 00 0 00",
            "import 00",
            "convert 00",
            "start",
            "unknown",
        ] {
            assert_eq!(dispatch(&mut state, line), "err");
        }
        assert!(dispatch(&mut state, "new").starts_with("ok "));
        for line in [
            "new",
            "seal 00",
            "trust curve25519:00",
            "open 00 2 00",
            "seal gg",
            "commit",
            "abort",
        ] {
            assert_eq!(dispatch(&mut state, line), "err");
        }
    }

    #[test]
    fn raw_ffi_error_zeroes_entire_output_and_length() {
        let (_, b) = paired();
        let mut output = vec![0xa5; MAX_PAYLOAD];
        let mut n = 999usize;
        let _guard = NATIVE_LOCK.lock().unwrap();
        let result = unsafe {
            proof_open(
                b.ptr.as_ptr(),
                [0u8; 1].as_ptr(),
                1,
                0,
                [0u8; 16].as_ptr(),
                16,
                output.as_mut_ptr(),
                &mut n,
            )
        };
        assert_ne!(result, 0);
        assert_eq!(n, 0);
        assert!(output.iter().all(|x| *x == 0));
    }

    #[test]
    fn all_raw_error_buffers_are_cleared() {
        let _guard = NATIVE_LOCK.lock().unwrap();
        let mut key = vec![0xa5; MAX_KEY];
        let mut payload = vec![0xa5; MAX_PAYLOAD];
        let mut keyn = 999;
        let mut payloadn = 999;
        let mut prekey = 999;
        let result = unsafe {
            proof_seal(
                std::ptr::null_mut(),
                [0u8; 1].as_ptr(),
                MAX_PLAINTEXT + 1,
                key.as_mut_ptr(),
                &mut keyn,
                &mut prekey,
                payload.as_mut_ptr(),
                &mut payloadn,
            )
        };
        assert_ne!(result, 0);
        assert_eq!((keyn, payloadn, prekey), (0, 0, 0));
        assert!(key.iter().chain(payload.iter()).all(|b| *b == 0));
        let mut snapshot = vec![0xa5; MAX_SNAPSHOT];
        let mut n = 999;
        assert_ne!(
            unsafe { proof_export(std::ptr::null(), snapshot.as_mut_ptr(), &mut n) },
            0
        );
        assert_eq!(n, 0);
        assert!(snapshot.iter().all(|b| *b == 0));
        let mut bundle = [0xa5; 160];
        let mut ids = [999; 2];
        assert_ne!(
            unsafe { proof_bundle(std::ptr::null(), bundle.as_mut_ptr(), ids.as_mut_ptr()) },
            0
        );
        assert_eq!(ids, [0, 0]);
        assert!(bundle.iter().all(|b| *b == 0));
    }

    #[test]
    fn existing_session_prekey_outer_identity_change_rejects() {
        let (mut a, mut b) = paired();
        let first = a.seal(b"first").unwrap();
        let second = a.seal(b"second").unwrap();
        assert!(first.prekey && second.prekey);
        assert_eq!(b.open(&first).unwrap(), b"first");
        let identity = a.bundle().unwrap().identity;
        // Test fixture lookup only: production decoding stays in Picomemo.
        let offset = second
            .key
            .windows(32)
            .position(|window| window == identity)
            .unwrap();
        let mut changed = second.clone();
        changed.key[offset + 31] ^= 0x80;
        let before = b.export().unwrap();
        assert!(b.open(&changed).is_err());
        assert_eq!(before, b.export().unwrap());
        assert_eq!(b.open(&second).unwrap(), b"second");
    }

    fn snapshot_session(snapshot: &[u8]) -> (usize, usize) {
        let store_n=u32::from_be_bytes(snapshot[8..12].try_into().unwrap()) as usize;
        let length_at=12+store_n;
        let session_n=u32::from_be_bytes(snapshot[length_at..length_at+4].try_into().unwrap()) as usize;
        (length_at+4,session_n)
    }

    fn session_role(snapshot: &[u8]) -> Option<u8> {
        let (start,n)=snapshot_session(snapshot);
        let bytes=&snapshot[start..start+n];
        fn varint(bytes:&[u8],pos:&mut usize)->u64 {
            let mut result=0;
            for shift in (0..64).step_by(7) {
                let byte=bytes[*pos];*pos+=1;
                result|=u64::from(byte&0x7f)<<shift;
                if byte&0x80==0 {return result;}
            }
            panic!("invalid synthetic session varint")
        }
        let mut pos=0;
        while pos<bytes.len() {
            let tag=varint(bytes,&mut pos);
            match tag&7 {
                0=>{let value=varint(bytes,&mut pos);if tag>>3==16 {return Some(value.try_into().unwrap())}},
                2=>{let n=varint(bytes,&mut pos);pos+=n as usize;},
                _=>panic!("unexpected synthetic session field"),
            }
        }
        None
    }

    #[test]
    fn initiation_role_serializes_and_malformed_or_missing_role_rejects() {
        let corrected=option_env!("PICOMEMO_CANDIDATE")==Some("ad-order-v1");
        let (mut a,mut b)=paired();
        let active=a.export().unwrap();
        let empty=b.export().unwrap();
        assert_eq!(session_role(&active),if corrected {Some(1)} else {None});
        assert_eq!(session_role(&empty),if corrected {Some(0)} else {None});
        let first=a.seal(b"role persistence").unwrap();
        assert_eq!(b.open(&first).unwrap(),b"role persistence");
        let passive=b.export().unwrap();
        assert_eq!(session_role(&passive),if corrected {Some(2)} else {None});
        if corrected {
            for snapshot in [&active,&empty,&passive] {
                let (start,n)=snapshot_session(snapshot);
                assert_eq!(&snapshot[start+n-3..start+n-1],&[0x80,0x01]);
                for role in [3,255] {
                    let mut bad=snapshot.clone();bad[start+n-1]=role;
                    assert!(Device::import(&bad).is_err());
                }
                let mut missing=snapshot.clone();
                missing.drain(start+n-3..start+n);
                missing[start-4..start].copy_from_slice(&((n-3) as u32).to_be_bytes());
                assert!(Device::import(&missing).is_err());
            }
            let (start,n)=snapshot_session(&active);
            let mut bad=active.clone();bad[start+n-1]=0;
            assert!(Device::import(&bad).is_err());
            let (start,n)=snapshot_session(&empty);
            let mut bad=empty.clone();bad[start+n-1]=1;
            assert!(Device::import(&bad).is_err());
        }
        let mut a=Device::import(&a.export().unwrap()).unwrap();
        let mut b=Device::import(&b.export().unwrap()).unwrap();
        assert_eq!(a.open(&b.seal(b"reply after restart").unwrap()).unwrap(),b"reply after restart");
        assert_eq!(b.open(&a.seal(b"next after restart").unwrap()).unwrap(),b"next after restart");
        assert_eq!(session_role(&a.export().unwrap()),if corrected {Some(1)} else {None});
        assert_eq!(session_role(&b.export().unwrap()),if corrected {Some(2)} else {None});
    }

    #[test]
    fn hex_decoder_has_exact_bounds_and_no_ambiguous_text() {
        assert_eq!(decode("00ff", 2).unwrap(), [0, 255]);
        for bad in ["0", "gg", "00 0", "\n00", "FF"] {
            assert!(decode(bad, 4).is_err());
        }
        assert!(decode("000000", 2).is_err());
    }
}

//! Fixed version, length-bounded public policy codec. No session or key encoding.
use super::{Error, MAX_SNAPSHOT, Snapshot};
use crate::authority::{CredentialBinding, MAX_BINDINGS, MAX_HOMES, Registry};
use faction_files::private_blobs::Principal;
#[cfg(feature = "delegation-issuer")]
#[path = "issuer_codec.rs"]
mod issuer;
#[cfg(feature = "delegation-services")]
#[path = "service_codec.rs"]
mod services;
const MAGIC: &[u8; 8] = b"FPPY0001";
struct Writer(Vec<u8>);
impl Writer {
    fn u64(&mut self, n: u64) {
        self.0.extend(n.to_be_bytes())
    }
    fn string(&mut self, s: &str) {
        self.0.extend((s.len() as u16).to_be_bytes());
        self.0.extend(s.as_bytes())
    }
    fn principal(&mut self, p: &Principal) {
        self.string(&p.account);
        self.0.extend(p.device.to_be_bytes());
        self.0.extend(p.identity)
    }
}
struct Reader<'a> {
    raw: &'a [u8],
    offset: usize,
}
impl<'a> Reader<'a> {
    fn take(&mut self, n: usize) -> Result<&'a [u8], Error> {
        let end = self.offset.checked_add(n).ok_or(Error::Invalid)?;
        let b = self.raw.get(self.offset..end).ok_or(Error::Invalid)?;
        self.offset = end;
        Ok(b)
    }
    fn byte(&mut self) -> Result<u8, Error> {
        Ok(self.take(1)?[0])
    }
    fn boolean(&mut self) -> Result<bool, Error> {
        match self.byte()? {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(Error::Invalid),
        }
    }
    fn u64(&mut self) -> Result<u64, Error> {
        Ok(u64::from_be_bytes(
            self.take(8)?.try_into().map_err(|_| Error::Invalid)?,
        ))
    }
    fn id(&mut self) -> Result<[u8; 32], Error> {
        self.take(32)?.try_into().map_err(|_| Error::Invalid)
    }
    fn string(&mut self, cap: usize) -> Result<String, Error> {
        let n = u16::from_be_bytes(self.take(2)?.try_into().map_err(|_| Error::Invalid)?) as usize;
        if n > cap {
            return Err(Error::Invalid);
        }
        Ok(std::str::from_utf8(self.take(n)?)
            .map_err(|_| Error::Invalid)?
            .to_owned())
    }
    fn principal(&mut self) -> Result<Principal, Error> {
        let p = Principal {
            account: self.string(254)?,
            device: u32::from_be_bytes(self.take(4)?.try_into().map_err(|_| Error::Invalid)?),
            identity: self.id()?,
        };
        p.validate()?;
        Ok(p)
    }
    fn count(&mut self, cap: usize) -> Result<usize, Error> {
        let n = self.byte()? as usize;
        if n > cap { Err(Error::Invalid) } else { Ok(n) }
    }
}
pub(super) fn encode(s: &Snapshot) -> Result<Vec<u8>, Error> {
    s.validate()?;
    #[cfg(feature = "delegation-issuer")]
    if s.service_format_v3 {
        return issuer::encode(s);
    }
    #[cfg(feature = "delegation-services")]
    if s.service_format_v2 {
        return services::encode(s);
    }
    let mut w = Writer(MAGIC.to_vec());
    w.u64(s.generation);
    w.u64(s.reset_epoch);
    w.0.push(u8::from(s.registry.is_some()));
    w.0.push(s.history.len() as u8);
    for (digest, p) in &s.history {
        w.0.extend(digest);
        w.principal(p)
    }
    if let Some(r) = &s.registry {
        w.0.push(r.homes().len() as u8);
        for h in r.homes() {
            w.string(h)
        }
        w.0.push(r.bindings().len() as u8);
        for e in r.bindings() {
            w.0.extend(e.certificate_sha256);
            w.principal(&e.principal);
            w.u64(e.valid_from);
            w.u64(e.valid_until);
            w.0.extend([
                u8::from(e.approved),
                u8::from(e.account_enabled),
                u8::from(e.device_enabled),
            ]);
            w.u64(e.generation)
        }
    }
    if w.0.len() > MAX_SNAPSHOT {
        return Err(Error::Invalid);
    }
    Ok(w.0)
}
pub(super) fn decode(raw: &[u8]) -> Result<Snapshot, Error> {
    if raw.len() > MAX_SNAPSHOT {
        return Err(Error::Invalid);
    }
    #[cfg(feature = "delegation-issuer")]
    if raw.get(..8) == Some(b"FPPY0003") {
        return issuer::decode(raw);
    }
    #[cfg(feature = "delegation-services")]
    if raw.get(..8) == Some(b"FPPY0002") {
        return services::decode(raw);
    }
    if raw.get(..8) != Some(MAGIC) {
        return Err(Error::Invalid);
    }
    let mut r = Reader { raw, offset: 8 };
    let generation = r.u64()?;
    let reset_epoch = r.u64()?;
    let available = r.boolean()?;
    let count = r.count(MAX_BINDINGS)?;
    let mut history = Vec::with_capacity(count);
    for _ in 0..count {
        history.push((r.id()?, r.principal()?))
    }
    let registry = if available {
        let count = r.count(MAX_HOMES)?;
        let mut homes = Vec::with_capacity(count);
        for _ in 0..count {
            homes.push(r.string(189)?)
        }
        let count = r.count(MAX_BINDINGS)?;
        let mut bindings = Vec::with_capacity(count);
        for _ in 0..count {
            bindings.push(CredentialBinding {
                certificate_sha256: r.id()?,
                principal: r.principal()?,
                valid_from: r.u64()?,
                valid_until: r.u64()?,
                approved: r.boolean()?,
                account_enabled: r.boolean()?,
                device_enabled: r.boolean()?,
                generation: r.u64()?,
            })
        }
        Some(Registry::new(generation, reset_epoch, homes, bindings)?)
    } else {
        None
    };
    if r.offset != raw.len() {
        return Err(Error::Invalid);
    }
    let s = Snapshot {
        generation,
        reset_epoch,
        registry,
        history,
        #[cfg(feature = "delegation-services")]
        service_policy: None,
        #[cfg(feature = "delegation-services")]
        service_history: Vec::new(),
        #[cfg(feature = "delegation-services")]
        service_format_v2: false,
        #[cfg(feature = "delegation-issuer")]
        service_format_v3: false,
    };
    s.validate()?;
    Ok(s)
}

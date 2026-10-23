//! Bounded local mail state. No transport can construct a verified delivery yet.
pub fn invalid() -> std::io::Error {
    std::io::Error::new(
        std::io::ErrorKind::InvalidData,
        "invalid local mail data or unlock secret",
    )
}
use std::io;
use zeroize::Zeroizing;
pub const MAX_DRAFTS: usize = 32;
pub const MAX_PINS: usize = 64;
pub const MAX_BODY: usize = 4096;
pub const MAX_PLAIN: usize = 256 * 1024;
pub struct Draft {
    pub to: String,
    pub subject: Zeroizing<String>,
    pub body: Zeroizing<String>,
}
pub struct Pin {
    pub address: String,
    pub device: u32,
    pub fingerprint: [u8; 32],
}
pub struct Data {
    pub home: String,
    pub drafts: Vec<Draft>,
    pub pins: Vec<Pin>,
}
/// Restricted preview syntax, not complete PRECIS/IDNA or XMPP address support.
pub fn address(s: &str) -> bool {
    let Some((user, host)) = s.split_once('@') else {
        return false;
    };
    s.len() <= 254
        && !user.is_empty()
        && user.len() <= 64
        && user
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b"._-".contains(&b))
        && host.contains('.')
        && host.len() <= 189
        && host.split('.').all(|p| {
            !p.is_empty()
                && p.len() <= 63
                && !p.starts_with('-')
                && !p.ends_with('-')
                && p.bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
        })
}
pub fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
pub fn unhex(s: &str) -> io::Result<[u8; 32]> {
    if s.len() != 64
        || !s
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return Err(invalid());
    }
    let mut out = [0; 32];
    for (i, b) in out.iter_mut().enumerate() {
        *b = u8::from_str_radix(&s[2 * i..2 * i + 2], 16).map_err(|_| invalid())?;
    }
    Ok(out)
}
fn text(s: &str, cap: usize) -> bool {
    s.len() <= cap && s.bytes().all(|b| (32..=126).contains(&b))
}
impl Data {
    pub fn new(home: String) -> io::Result<Self> {
        if !address(&home) {
            return Err(invalid());
        }
        Ok(Self {
            home,
            drafts: Vec::new(),
            pins: Vec::new(),
        })
    }
    pub fn draft(
        &mut self,
        to: String,
        subject: Zeroizing<String>,
        body: Zeroizing<String>,
    ) -> io::Result<()> {
        if self.drafts.len() == MAX_DRAFTS
            || !address(&to)
            || !text(&subject, 128)
            || !text(&body, MAX_BODY)
            || body.is_empty()
        {
            return Err(invalid());
        }
        self.drafts.push(Draft { to, subject, body });
        Ok(())
    }
    /// Explicit local verification only. A changed key needs removal and a fresh
    /// out-of-band verification. No server result or typed address proves identity.
    pub fn pin(&mut self, address: String, device: u32, fingerprint: [u8; 32]) -> io::Result<()> {
        if !self::address(&address)
            || device == 0
            || device > i32::MAX as u32
            || self.pins.len() == MAX_PINS
            || self
                .pins
                .iter()
                .any(|p| p.address == address && p.device == device)
        {
            return Err(invalid());
        }
        self.pins.push(Pin {
            address,
            device,
            fingerprint,
        });
        Ok(())
    }
    pub fn encode(&self) -> io::Result<Zeroizing<Vec<u8>>> {
        let mut out = Zeroizing::new(Vec::with_capacity(MAX_PLAIN));
        fn field(out: &mut Vec<u8>, value: &[u8]) {
            out.extend_from_slice(&(value.len() as u32).to_be_bytes());
            out.extend_from_slice(value);
        }
        field(&mut out, self.home.as_bytes());
        out.extend_from_slice(&(self.pins.len() as u32).to_be_bytes());
        for p in &self.pins {
            field(&mut out, p.address.as_bytes());
            out.extend_from_slice(&p.device.to_be_bytes());
            out.extend_from_slice(&p.fingerprint);
        }
        out.extend_from_slice(&(self.drafts.len() as u32).to_be_bytes());
        for d in &self.drafts {
            field(&mut out, d.to.as_bytes());
            field(&mut out, d.subject.as_bytes());
            field(&mut out, d.body.as_bytes());
        }
        if out.len() > MAX_PLAIN {
            return Err(invalid());
        }
        Ok(out)
    }
    pub fn decode(raw: &[u8]) -> io::Result<Self> {
        struct Reader<'a>(&'a [u8]);
        impl<'a> Reader<'a> {
            fn take(&mut self, len: usize) -> io::Result<&'a [u8]> {
                if len > self.0.len() {
                    return Err(invalid());
                }
                let (a, b) = self.0.split_at(len);
                self.0 = b;
                Ok(a)
            }
            fn number(&mut self) -> io::Result<u32> {
                Ok(u32::from_be_bytes(
                    self.take(4)?.try_into().map_err(|_| invalid())?,
                ))
            }
            fn string(&mut self, cap: usize) -> io::Result<Zeroizing<String>> {
                let len = self.number()? as usize;
                if len > cap {
                    return Err(invalid());
                }
                Ok(Zeroizing::new(
                    std::str::from_utf8(self.take(len)?)
                        .map_err(|_| invalid())?
                        .to_owned(),
                ))
            }
        }
        if raw.len() > MAX_PLAIN {
            return Err(invalid());
        }
        let mut r = Reader(raw);
        let mut data = Self::new(r.string(254)?.to_string())?;
        let count = r.number()? as usize;
        if count > MAX_PINS {
            return Err(invalid());
        }
        for _ in 0..count {
            let who = r.string(254)?.to_string();
            let id = r.number()?;
            let fp = r.take(32)?.try_into().map_err(|_| invalid())?;
            data.pin(who, id, fp)?;
        }
        let count = r.number()? as usize;
        if count > MAX_DRAFTS {
            return Err(invalid());
        }
        for _ in 0..count {
            data.draft(
                r.string(254)?.to_string(),
                r.string(128)?,
                r.string(MAX_BODY)?,
            )?;
        }
        if !r.0.is_empty() {
            return Err(invalid());
        }
        Ok(data)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bounded_state_and_unambiguous_addresses() {
        for s in [
            "alice",
            "alice@host",
            "Alice@home.test",
            "a@b..test",
            "a@-b.test",
            "a@b.test/r",
            "a@@b.test",
        ] {
            assert!(!address(s));
        }
        let mut d = Data::new("alice@home.test".into()).unwrap();
        d.pin("bob@away.test".into(), 1, [1; 32]).unwrap();
        assert!(d.pin("bob@away.test".into(), 1, [2; 32]).is_err());
        assert!(d.pin("bob@away.test".into(), 0, [2; 32]).is_err());
        assert!(
            d.pin("bob@away.test".into(), i32::MAX as u32 + 1, [2; 32])
                .is_err()
        );
        assert!(
            d.draft(
                "bob@away.test".into(),
                Zeroizing::new("\x1b]52;evil".into()),
                Zeroizing::new("x".into())
            )
            .is_err()
        );
        for _ in 0..MAX_DRAFTS {
            d.draft(
                "bob@away.test".into(),
                Zeroizing::new("subject".into()),
                Zeroizing::new("body".into()),
            )
            .unwrap();
        }
        assert!(
            d.draft(
                "bob@away.test".into(),
                Zeroizing::new("x".into()),
                Zeroizing::new("y".into())
            )
            .is_err()
        );
        let encoded = d.encode().unwrap();
        let decoded = Data::decode(&encoded).unwrap();
        assert_eq!(decoded.drafts.len(), MAX_DRAFTS);
        assert_eq!(decoded.pins[0].fingerprint, [1; 32]);
        for i in 0..encoded.len() {
            assert!(Data::decode(&encoded[..i]).is_err());
        }
        let mut trailing = encoded.clone();
        trailing.push(0);
        assert!(Data::decode(&trailing).is_err());
        assert_eq!(unhex(&hex(&[7; 32])).unwrap(), [7; 32]);
        assert!(unhex(&"X".repeat(64)).is_err());
    }
}

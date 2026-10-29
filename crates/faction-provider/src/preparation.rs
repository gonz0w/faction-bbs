//! Bounded self-asserted public data for deliberate offline preparation.
//! No filesystem, private-key, runtime, network or approval authority is held.
use faction_files::private_blobs::Principal;
use openssl::{base64, sha::sha256, x509::X509};
use std::fmt;

pub const MAX_CARD: usize = 8192;
pub const MAX_PEM: usize = 65_536;
/// Fixed diagnostics never include public identifiers or supplied input.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Error;
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("invalid offline preparation data")
    }
}
impl std::error::Error for Error {}
type Result<T> = std::result::Result<T, Error>;

fn digest(text: &str) -> Result<[u8; 32]> {
    if text.len() != 64 {
        return Err(Error);
    }
    let mut bytes = [0; 32];
    for (out, pair) in bytes.iter_mut().zip(text.as_bytes().chunks_exact(2)) {
        let digit = |b| match b {
            b'0'..=b'9' => Ok(b - b'0'),
            b'a'..=b'f' => Ok(b - b'a' + 10),
            _ => Err(Error),
        };
        *out = digit(pair[0])? * 16 + digit(pair[1])?;
    }
    Ok(bytes)
}
fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
fn string(t: &mut toml::Table, key: &str) -> Result<String> {
    t.remove(key)
        .and_then(|v| v.as_str().map(str::to_owned))
        .ok_or(Error)
}

/// Self-asserted routing metadata, not possession, membership or approval proof.
/// This local file contract is not an adopted transport wire protocol.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Card {
    pub principal: Principal,
    pub certificate_sha256: [u8; 32],
}
impl Card {
    pub fn new(principal: Principal, certificate_sha256: [u8; 32]) -> Result<Self> {
        let card = Self {
            principal,
            certificate_sha256,
        };
        card.validate()?;
        Ok(card)
    }
    fn validate(&self) -> Result<()> {
        self.principal.validate().map_err(|_| Error)?;
        if self.certificate_sha256 == [0; 32] {
            return Err(Error);
        }
        Ok(())
    }
    /// Parse exactly the bounded version1 fields; the TOML parser rejects duplicates.
    pub fn decode(bytes: &[u8]) -> Result<Self> {
        if bytes.is_empty() || bytes.len() > MAX_CARD {
            return Err(Error);
        }
        let mut t = std::str::from_utf8(bytes)
            .map_err(|_| Error)?
            .parse::<toml::Table>()
            .map_err(|_| Error)?;
        if t.remove("version").and_then(|v| v.as_integer()) != Some(1) {
            return Err(Error);
        }
        let account = string(&mut t, "account")?;
        let device = t
            .remove("device")
            .and_then(|v| v.as_integer())
            .and_then(|v| u32::try_from(v).ok())
            .ok_or(Error)?;
        let identity = digest(&string(&mut t, "identity")?)?;
        let certificate_sha256 = digest(&string(&mut t, "certificate_sha256")?)?;
        if !t.is_empty() {
            return Err(Error);
        }
        Self::new(
            Principal {
                account,
                device,
                identity,
            },
            certificate_sha256,
        )
    }
    /// Public fields are revalidated on every serialization, including after mutation.
    pub fn encode(&self) -> Result<Vec<u8>> {
        self.validate()?;
        let mut t = toml::Table::new();
        t.insert("version".into(), 1i64.into());
        t.insert("account".into(), self.principal.account.clone().into());
        t.insert("device".into(), i64::from(self.principal.device).into());
        t.insert("identity".into(), hex(&self.principal.identity).into());
        t.insert(
            "certificate_sha256".into(),
            hex(&self.certificate_sha256).into(),
        );
        let bytes = t.to_string().into_bytes();
        if bytes.is_empty() || bytes.len() > MAX_CARD {
            return Err(Error);
        }
        Ok(bytes)
    }
}

/// Hash one explicitly selected public certificate's exact canonical DER.
/// Only surrounding ASCII whitespace is permitted outside the single PEM block.
/// No subject mapping, chain trust, validity, approval or key-possession claim follows.
pub fn certificate_sha256(pem: &[u8]) -> Result<[u8; 32]> {
    if pem.is_empty() || pem.len() > MAX_PEM {
        return Err(Error);
    }
    let body = pem
        .trim_ascii()
        .strip_prefix(b"-----BEGIN CERTIFICATE-----")
        .ok_or(Error)?
        .strip_suffix(b"-----END CERTIFICATE-----")
        .ok_or(Error)?;
    if body.is_empty()
        || !body
            .iter()
            .all(|b| b.is_ascii_alphanumeric() || b"+/=".contains(b) || b.is_ascii_whitespace())
    {
        return Err(Error);
    }
    let encoded: String = body
        .iter()
        .filter(|b| !b.is_ascii_whitespace())
        .map(|b| char::from(*b))
        .collect();
    let der = base64::decode_block(&encoded).map_err(|_| Error)?;
    if base64::encode_block(&der) != encoded {
        return Err(Error);
    }
    let cert = X509::from_der(&der).map_err(|_| Error)?;
    // Refuse trailing/ambiguous DER or noncanonical encodings hidden in a PEM block.
    if cert.to_der().map_err(|_| Error)? != der {
        return Err(Error);
    }
    Ok(sha256(&der))
}

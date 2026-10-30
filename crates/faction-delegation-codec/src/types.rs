use crate::{Error, MAX_TOKEN};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};

/// Sensitive unverified bearer text. Exposure requires an explicit method call.
/// No Debug/Display implementation is supplied, including on containing types.
/// ```compile_fail
/// use faction_delegation_codec::UnverifiedToken;
/// println!("{:?}", UnverifiedToken::parse("private").unwrap());
/// ```
/// ```compile_fail
/// use faction_delegation_codec::UnverifiedToken;
/// println!("{}", UnverifiedToken::parse("private").unwrap());
/// ```
pub struct UnverifiedToken(String);
impl UnverifiedToken {
    pub fn parse(value: &str) -> Result<Self, Error> {
        let stem = value.trim_end_matches('=');
        if value.is_empty()
            || value.len() > MAX_TOKEN
            || stem.is_empty()
            || !stem
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"-._~+/".contains(&b))
        {
            return Err(Error);
        }
        Ok(Self(value.into()))
    }
    pub fn expose(&self) -> &str {
        &self.0
    }
}

/// An unverified canonical unpadded base64url spelling of exactly 32 bytes.
/// It is used for a server-record scope and RFC 8705 `cnf.x5t#S256` syntax.
/// This checks no certificate digest or possession. The record scope is sensitive.
pub struct UnverifiedReference([u8; 32]);
impl UnverifiedReference {
    pub fn parse(value: &str) -> Result<Self, Error> {
        if value.len() != 43 {
            return Err(Error);
        }
        let mut bytes = [0; 32];
        if URL_SAFE_NO_PAD
            .decode_slice(value, &mut bytes)
            .map_err(|_| Error)?
            != 32
            || URL_SAFE_NO_PAD.encode(bytes) != value
        {
            return Err(Error);
        }
        Ok(Self(bytes))
    }
    pub fn encoded(&self) -> String {
        URL_SAFE_NO_PAD.encode(self.0)
    }
}

pub struct UnverifiedTokenRequest {
    /// Asserted client identifier; TLS registration matching remains mandatory.
    pub client_id: String,
    /// Faction's single opaque operation-record reference in the standard scope.
    pub scope: UnverifiedReference,
}
pub struct UnverifiedIntrospectionRequest {
    pub client_id: String,
    pub token: UnverifiedToken,
}
pub struct UnverifiedTokenResponse {
    pub token: UnverifiedToken,
    pub expires_in: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UnverifiedPrincipal {
    pub account: String,
    pub device: u32,
    pub identity: String,
}
impl UnverifiedPrincipal {
    pub(crate) fn validate(&self) -> Result<(), Error> {
        let (user, home) = self.account.split_once('@').ok_or(Error)?;
        if user.is_empty()
            || user.len() > 64
            || self.account.len() > 254
            || home.len() > 189
            || !user
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b"._-".contains(&b))
            || self.device == 0
            || self.identity == "00".repeat(32)
        {
            return Err(Error);
        }
        host(home)?;
        hex(&self.identity)
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UnverifiedBinding {
    pub sender: UnverifiedPrincipal,
    pub recipient: UnverifiedPrincipal,
    pub id: String,
    pub size: u64,
    pub created: u64,
    pub expires: u64,
}
impl UnverifiedBinding {
    pub(crate) fn validate(&self) -> Result<(), Error> {
        self.sender.validate()?;
        self.recipient.validate()?;
        hex(&self.id)?;
        if !(16..=1_048_592).contains(&self.size)
            || self.expires <= self.created
            || self.expires - self.created > 604_800
        {
            return Err(Error);
        }
        Ok(())
    }
}

/// Faction application mapping of standard and extension claims. Public fields
/// remain editable untrusted data; encoders revalidate them. No clock or registry
/// check, certificate authentication or operation approval is performed here.
pub struct UnverifiedClaims {
    pub issuer: String,
    pub issuer_home: String,
    pub client_id: String,
    pub audience: String,
    pub subject: String,
    pub issued_at: u64,
    pub expires_at: u64,
    pub generation: u64,
    pub reset: u64,
    pub certificate_thumbprint: UnverifiedReference,
    pub binding: UnverifiedBinding,
}
impl UnverifiedClaims {
    pub(crate) fn validate(&self) -> Result<(), Error> {
        host(&self.issuer)?;
        host(&self.issuer_home)?;
        client_id(&self.client_id)?;
        let home = self.audience.strip_suffix("/attachments").ok_or(Error)?;
        host(home)?;
        self.binding.validate()?;
        if self.issued_at == 0
            || self.expires_at <= self.issued_at
            || self.expires_at - self.issued_at > 10
            || self.expires_at > self.binding.expires
            || self.issued_at < self.binding.created
            || self.generation == 0
        {
            return Err(Error);
        }
        Ok(())
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UnverifiedRetrievalAction {
    GetChunk { offset: u64 },
    Status,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UnverifiedObservationAction {
    Publish,
    Status,
    Delete,
}

/// Service-subject policy observations cannot be confused with device-subject
/// retrieval wire data. Neither variant provides a verified capability.
pub enum UnverifiedIntrospection {
    Inactive,
    Retrieval {
        claims: UnverifiedClaims,
        action: UnverifiedRetrievalAction,
    },
    RecipientCurrent {
        claims: UnverifiedClaims,
        action: UnverifiedObservationAction,
    },
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UnverifiedRefusal {
    InvalidRequest,
    InvalidToken,
    Unavailable,
}

pub(crate) fn client_id(value: &str) -> Result<(), Error> {
    if value.is_empty()
        || value.len() > 64
        || !value
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b"._-".contains(&b))
    {
        return Err(Error);
    }
    Ok(())
}
pub(crate) fn host(value: &str) -> Result<(), Error> {
    if value.len() > 253
        || !value.contains('.')
        || !value.split('.').all(|p| {
            !p.is_empty()
                && p.len() <= 63
                && !p.starts_with('-')
                && !p.ends_with('-')
                && p.bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
        })
    {
        return Err(Error);
    }
    Ok(())
}
pub(crate) fn hex(value: &str) -> Result<(), Error> {
    if value.len() != 64
        || !value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return Err(Error);
    }
    Ok(())
}

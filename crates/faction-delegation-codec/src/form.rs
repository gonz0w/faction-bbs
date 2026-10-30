use crate::types::client_id;
use crate::{
    Error, MAX_FIELDS, MAX_FORM, UnverifiedIntrospectionRequest, UnverifiedReference,
    UnverifiedToken, UnverifiedTokenRequest,
};
use std::collections::BTreeMap;

// The upstream form parser intentionally replaces malformed UTF-8. Prevalidate
// each percent-decoded component so that replacement never hides hostile bytes.
fn component(value: &[u8]) -> Result<(), Error> {
    let mut decoded = Vec::with_capacity(value.len());
    let mut index = 0;
    while index < value.len() {
        match value[index] {
            b'%' => {
                let a = *value.get(index + 1).ok_or(Error)?;
                let b = *value.get(index + 2).ok_or(Error)?;
                let digit = |c: u8| (c as char).to_digit(16).map(|d| d as u8).ok_or(Error);
                decoded.push(digit(a)? * 16 + digit(b)?);
                index += 3;
            }
            b'+' => {
                decoded.push(b' ');
                index += 1;
            }
            b => {
                decoded.push(b);
                index += 1;
            }
        }
    }
    std::str::from_utf8(&decoded).map_err(|_| Error)?;
    Ok(())
}
fn parse(body: &[u8]) -> Result<BTreeMap<String, String>, Error> {
    if body.is_empty() || body.len() > MAX_FORM {
        return Err(Error);
    }
    let mut count = 0;
    for pair in body.split(|b| *b == b'&') {
        count += 1;
        if count > MAX_FIELDS {
            return Err(Error);
        }
        let mut fields = pair.split(|b| *b == b'=');
        let key = fields.next().ok_or(Error)?;
        let value = fields.next().ok_or(Error)?;
        if key.is_empty() || fields.next().is_some() {
            return Err(Error);
        }
        component(key)?;
        component(value)?;
    }
    let mut fields = BTreeMap::new();
    for (key, value) in form_urlencoded::parse(body) {
        if fields
            .insert(key.into_owned(), value.into_owned())
            .is_some()
        {
            return Err(Error);
        }
    }
    Ok(fields)
}
fn encode(pairs: &[(&str, &str)]) -> Result<Vec<u8>, Error> {
    let mut serializer = form_urlencoded::Serializer::new(String::new());
    for (key, value) in pairs {
        serializer.append_pair(key, value);
    }
    let bytes = serializer.finish().into_bytes();
    if bytes.len() > MAX_FORM {
        return Err(Error);
    }
    Ok(bytes)
}
pub fn encode_token_form(request: &UnverifiedTokenRequest) -> Result<Vec<u8>, Error> {
    client_id(&request.client_id)?;
    encode(&[
        ("grant_type", "client_credentials"),
        ("client_id", &request.client_id),
        ("scope", &request.scope.encoded()),
    ])
}
pub fn decode_token_form(body: &[u8]) -> Result<UnverifiedTokenRequest, Error> {
    let mut fields = parse(body)?;
    if fields.len() != 3 || fields.remove("grant_type").ok_or(Error)? != "client_credentials" {
        return Err(Error);
    }
    let id = fields.remove("client_id").ok_or(Error)?;
    client_id(&id)?;
    let scope = UnverifiedReference::parse(&fields.remove("scope").ok_or(Error)?)?;
    Ok(UnverifiedTokenRequest {
        client_id: id,
        scope,
    })
}
pub fn encode_introspection_form(
    request: &UnverifiedIntrospectionRequest,
) -> Result<Vec<u8>, Error> {
    client_id(&request.client_id)?;
    encode(&[
        ("token", request.token.expose()),
        ("token_type_hint", "access_token"),
        ("client_id", &request.client_id),
    ])
}
pub fn decode_introspection_form(body: &[u8]) -> Result<UnverifiedIntrospectionRequest, Error> {
    let mut fields = parse(body)?;
    if fields.len() != 3 || fields.remove("token_type_hint").ok_or(Error)? != "access_token" {
        return Err(Error);
    }
    let id = fields.remove("client_id").ok_or(Error)?;
    client_id(&id)?;
    let token = UnverifiedToken::parse(&fields.remove("token").ok_or(Error)?)?;
    Ok(UnverifiedIntrospectionRequest {
        client_id: id,
        token,
    })
}

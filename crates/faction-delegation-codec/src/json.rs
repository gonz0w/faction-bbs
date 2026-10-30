use crate::*;
use serde::de::{self, DeserializeSeed, MapAccess, Visitor};
use serde_json::{Map, Value};
use std::{collections::BTreeMap, fmt};

// A duplicate-aware visitor is essential: Value's normal map deserializer keeps
// the last duplicate. Integers only, no arrays/null, and explicit depth/field
// bounds make this exact private profile smaller than arbitrary JSON.
struct Seed(usize);
impl<'de> DeserializeSeed<'de> for Seed {
    type Value = Value;
    fn deserialize<D: de::Deserializer<'de>>(self, d: D) -> Result<Value, D::Error> {
        if self.0 > MAX_DEPTH {
            return Err(de::Error::custom("invalid wire data"));
        }
        d.deserialize_any(self)
    }
}
impl<'de> Visitor<'de> for Seed {
    type Value = Value;
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("bounded wire data")
    }
    fn visit_bool<E: de::Error>(self, v: bool) -> Result<Value, E> {
        Ok(Value::Bool(v))
    }
    fn visit_u64<E: de::Error>(self, v: u64) -> Result<Value, E> {
        Ok(Value::from(v))
    }
    fn visit_str<E: de::Error>(self, v: &str) -> Result<Value, E> {
        Ok(Value::String(v.into()))
    }
    fn visit_string<E: de::Error>(self, v: String) -> Result<Value, E> {
        Ok(Value::String(v))
    }
    fn visit_map<M: MapAccess<'de>>(self, mut input: M) -> Result<Value, M::Error> {
        let mut fields = Map::new();
        while let Some(key) = input.next_key::<String>()? {
            if key.len() > 64 || fields.len() >= MAX_FIELDS || fields.contains_key(&key) {
                return Err(de::Error::custom("invalid wire data"));
            }
            fields.insert(key, input.next_value_seed(Seed(self.0 + 1))?);
        }
        Ok(Value::Object(fields))
    }
}
fn parse(body: &[u8]) -> Result<Map<String, Value>, Error> {
    if body.is_empty() || body.len() > MAX_JSON {
        return Err(Error);
    }
    let mut deserializer = serde_json::Deserializer::from_slice(body);
    let value = Seed(0).deserialize(&mut deserializer).map_err(|_| Error)?;
    deserializer.end().map_err(|_| Error)?;
    object(value)
}
fn object(value: Value) -> Result<Map<String, Value>, Error> {
    value.as_object().cloned().ok_or(Error)
}
fn take(fields: &mut Map<String, Value>, key: &str) -> Result<Value, Error> {
    fields.remove(key).ok_or(Error)
}
fn string(fields: &mut Map<String, Value>, key: &str) -> Result<String, Error> {
    match take(fields, key)? {
        Value::String(value) => Ok(value),
        _ => Err(Error),
    }
}
fn integer(fields: &mut Map<String, Value>, key: &str) -> Result<u64, Error> {
    take(fields, key)?.as_u64().ok_or(Error)
}
fn finish(fields: Map<String, Value>) -> Result<(), Error> {
    if fields.is_empty() {
        Ok(())
    } else {
        Err(Error)
    }
}
fn serialize(value: Value) -> Result<Vec<u8>, Error> {
    let bytes = serde_json::to_vec(&value).map_err(|_| Error)?;
    if bytes.len() > MAX_JSON {
        return Err(Error);
    }
    Ok(bytes)
}

pub fn decode_token_json(body: &[u8]) -> Result<UnverifiedTokenResponse, Error> {
    let mut fields = parse(body)?;
    let token = UnverifiedToken::parse(&string(&mut fields, "access_token")?)?;
    let expires_in = integer(&mut fields, "expires_in")?;
    if string(&mut fields, "token_type")? != "Bearer" || !(1..=10).contains(&expires_in) {
        return Err(Error);
    }
    finish(fields)?;
    Ok(UnverifiedTokenResponse { token, expires_in })
}
pub fn encode_token_json(response: &UnverifiedTokenResponse) -> Result<Vec<u8>, Error> {
    if !(1..=10).contains(&response.expires_in) {
        return Err(Error);
    }
    serialize(
        serde_json::json!({"access_token":response.token.expose(),"expires_in":response.expires_in,"token_type":"Bearer"}),
    )
}
fn principal(value: Value) -> Result<UnverifiedPrincipal, Error> {
    let mut fields = object(value)?;
    let principal = UnverifiedPrincipal {
        account: string(&mut fields, "account")?,
        device: integer(&mut fields, "device")?
            .try_into()
            .map_err(|_| Error)?,
        identity: string(&mut fields, "identity")?,
    };
    finish(fields)?;
    principal.validate()?;
    Ok(principal)
}
fn binding(value: Value) -> Result<UnverifiedBinding, Error> {
    let mut fields = object(value)?;
    let binding = UnverifiedBinding {
        sender: principal(take(&mut fields, "sender")?)?,
        recipient: principal(take(&mut fields, "recipient")?)?,
        id: string(&mut fields, "id")?,
        size: integer(&mut fields, "size")?,
        created: integer(&mut fields, "created")?,
        expires: integer(&mut fields, "expires")?,
    };
    finish(fields)?;
    binding.validate()?;
    Ok(binding)
}

pub fn decode_introspection_json(body: &[u8]) -> Result<UnverifiedIntrospection, Error> {
    let mut fields = parse(body)?;
    let active = take(&mut fields, "active")?.as_bool().ok_or(Error)?;
    if !active {
        finish(fields)?;
        return Ok(UnverifiedIntrospection::Inactive);
    }
    let mut cnf = object(take(&mut fields, "cnf")?)?;
    let certificate_thumbprint = UnverifiedReference::parse(&string(&mut cnf, "x5t#S256")?)?;
    finish(cnf)?;
    let claims = UnverifiedClaims {
        issuer: string(&mut fields, "iss")?,
        issuer_home: string(&mut fields, "faction_issuer_home")?,
        client_id: string(&mut fields, "client_id")?,
        audience: string(&mut fields, "aud")?,
        subject: string(&mut fields, "sub")?,
        issued_at: integer(&mut fields, "iat")?,
        expires_at: integer(&mut fields, "exp")?,
        generation: integer(&mut fields, "faction_generation")?,
        reset: integer(&mut fields, "faction_reset")?,
        certificate_thumbprint,
        binding: binding(take(&mut fields, "faction_binding")?)?,
    };
    claims.validate()?;
    let purpose = string(&mut fields, "faction_purpose")?;
    let action = string(&mut fields, "faction_action")?;
    let result = match purpose.as_str() {
        "retrieval" => {
            if claims.subject != claims.binding.recipient.account {
                return Err(Error);
            }
            let action = match action.as_str() {
                "get_chunk" => {
                    let offset = integer(&mut fields, "faction_offset")?;
                    if offset > claims.binding.size {
                        return Err(Error);
                    }
                    UnverifiedRetrievalAction::GetChunk { offset }
                }
                "status" => UnverifiedRetrievalAction::Status,
                _ => return Err(Error),
            };
            UnverifiedIntrospection::Retrieval { claims, action }
        }
        "recipient_current" => {
            if claims.subject != claims.client_id {
                return Err(Error);
            }
            let action = match action.as_str() {
                "publish" => UnverifiedObservationAction::Publish,
                "status" => UnverifiedObservationAction::Status,
                "delete" => UnverifiedObservationAction::Delete,
                _ => return Err(Error),
            };
            UnverifiedIntrospection::RecipientCurrent { claims, action }
        }
        _ => return Err(Error),
    };
    finish(fields)?;
    Ok(result)
}

fn principal_value(principal: &UnverifiedPrincipal) -> Value {
    serde_json::json!({"account":principal.account,"device":principal.device,"identity":principal.identity})
}
fn binding_value(binding: &UnverifiedBinding) -> Value {
    serde_json::json!({"sender":principal_value(&binding.sender),"recipient":principal_value(&binding.recipient),
        "id":binding.id,"size":binding.size,"created":binding.created,"expires":binding.expires})
}
pub fn encode_introspection_json(response: &UnverifiedIntrospection) -> Result<Vec<u8>, Error> {
    let (claims, purpose, action, offset) = match response {
        UnverifiedIntrospection::Inactive => return Ok(b"{\"active\":false}".to_vec()),
        UnverifiedIntrospection::Retrieval { claims, action } => {
            if claims.subject != claims.binding.recipient.account {
                return Err(Error);
            }
            let (action, offset) = match action {
                UnverifiedRetrievalAction::GetChunk { offset } => {
                    if *offset > claims.binding.size {
                        return Err(Error);
                    }
                    ("get_chunk", Some(*offset))
                }
                UnverifiedRetrievalAction::Status => ("status", None),
            };
            (claims, "retrieval", action, offset)
        }
        UnverifiedIntrospection::RecipientCurrent { claims, action } => {
            if claims.subject != claims.client_id {
                return Err(Error);
            }
            let action = match action {
                UnverifiedObservationAction::Publish => "publish",
                UnverifiedObservationAction::Status => "status",
                UnverifiedObservationAction::Delete => "delete",
            };
            (claims, "recipient_current", action, None)
        }
    };
    claims.validate()?;
    let mut fields = BTreeMap::from([
        ("active", Value::Bool(true)),
        ("iss", Value::String(claims.issuer.clone())),
        ("sub", Value::String(claims.subject.clone())),
        ("client_id", Value::String(claims.client_id.clone())),
        ("aud", Value::String(claims.audience.clone())),
        ("iat", Value::from(claims.issued_at)),
        ("exp", Value::from(claims.expires_at)),
        (
            "cnf",
            serde_json::json!({"x5t#S256":claims.certificate_thumbprint.encoded()}),
        ),
        ("faction_purpose", Value::String(purpose.into())),
        ("faction_action", Value::String(action.into())),
        (
            "faction_issuer_home",
            Value::String(claims.issuer_home.clone()),
        ),
        ("faction_generation", Value::from(claims.generation)),
        ("faction_reset", Value::from(claims.reset)),
        ("faction_binding", binding_value(&claims.binding)),
    ]);
    if let Some(offset) = offset {
        fields.insert("faction_offset", Value::from(offset));
    }
    serialize(Value::Object(
        fields
            .into_iter()
            .map(|(key, value)| (key.into(), value))
            .collect(),
    ))
}

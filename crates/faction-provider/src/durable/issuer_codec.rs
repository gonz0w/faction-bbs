//! V3 retains the byte-defined v1 device payload, with a role-aware service section.
//! V2's deliberately foreign-only service decoder is never used for local relays.
use super::{Error, MAX_BINDINGS, MAX_SNAPSHOT, Reader, Snapshot, Writer};
use crate::services::{
    IssuerRoute, ServiceIdentity, ServicePolicy, ServiceRegistration, ServiceUse,
};
pub(super) fn encode(s: &Snapshot) -> Result<Vec<u8>, Error> {
    s.validate()?;
    let mut device = s.clone();
    device.service_format_v2 = false;
    device.service_format_v3 = false;
    device.service_history.clear();
    device.service_policy = None;
    let original = super::encode(&device)?;
    let mut w = Writer(b"FPPY0003".to_vec());
    w.0.extend((original.len() as u32).to_be_bytes());
    w.0.extend(original);
    w.0.push(s.service_history.len() as u8);
    for (digest, id) in &s.service_history {
        w.0.extend(digest);
        w.string(&id.home);
        w.string(&id.client_id);
    }
    w.0.push(u8::from(s.service_policy.is_some()));
    if let Some(p) = &s.service_policy {
        w.string(p.local_home());
        w.0.push(p.registrations().len() as u8);
        for e in p.registrations() {
            w.string(&e.home);
            w.string(&e.client_id);
            w.string(&e.dns_san);
            w.0.extend(e.certificate_sha256);
            w.0.push(u8::from(e.enabled));
            w.u64(e.valid_from);
            w.u64(e.valid_until);
            w.0.push(match p.service_use(&e.identity()).ok_or(Error::Invalid)? {
                ServiceUse::Observation => 0,
                ServiceUse::Relay => 1,
                ServiceUse::Introspection => 2,
                #[cfg(feature = "delegation-verifier")]
                ServiceUse::ResourceClient
                | ServiceUse::OutboundIntrospection
                | ServiceUse::IssuerEndpoint => return Err(Error::Invalid),
            });
        }
        w.0.push(u8::from(p.issuer.is_some()));
        if let Some(i) = &p.issuer {
            w.string(&i.issuer);
            w.0.push(i.routes.len() as u8);
            for route in &i.routes {
                w.string(&route.sender_home);
                w.string(&route.audience);
                w.string(&route.relay_client_id);
                w.string(&route.introspection_client_id);
            }
        }
    }
    if w.0.len() > MAX_SNAPSHOT {
        return Err(Error::Invalid);
    }
    Ok(w.0)
}
pub(super) fn decode(raw: &[u8]) -> Result<Snapshot, Error> {
    if raw.len() > MAX_SNAPSHOT || raw.get(..8) != Some(b"FPPY0003") {
        return Err(Error::Invalid);
    }
    let mut r = Reader { raw, offset: 8 };
    let length = u32::from_be_bytes(r.take(4)?.try_into().map_err(|_| Error::Invalid)?) as usize;
    let original = r.take(length)?;
    if original.get(..8) != Some(b"FPPY0001") {
        return Err(Error::Invalid);
    }
    let mut s = super::decode(original)?;
    s.service_format_v2 = true;
    s.service_format_v3 = true;
    for _ in 0..r.count(MAX_BINDINGS)? {
        s.service_history.push((
            r.id()?,
            ServiceIdentity {
                home: r.string(189)?,
                client_id: r.string(254)?,
            },
        ));
    }
    if r.boolean()? {
        let home = r.string(189)?;
        let mut entries = Vec::new();
        for _ in 0..r.count(MAX_BINDINGS)? {
            let e = ServiceRegistration {
                home: r.string(189)?,
                client_id: r.string(254)?,
                dns_san: r.string(189)?,
                certificate_sha256: r.id()?,
                enabled: r.boolean()?,
                valid_from: r.u64()?,
                valid_until: r.u64()?,
            };
            let purpose = match r.byte()? {
                0 => ServiceUse::Observation,
                1 => ServiceUse::Relay,
                2 => ServiceUse::Introspection,
                _ => return Err(Error::Invalid),
            };
            entries.push((e, purpose));
        }
        s.service_policy = Some(if r.boolean()? {
            let issuer = r.string(189)?;
            let mut routes = Vec::new();
            for _ in 0..r.count(crate::authority::MAX_HOMES)? {
                routes.push(IssuerRoute {
                    sender_home: r.string(189)?,
                    audience: r.string(254)?,
                    relay_client_id: r.string(64)?,
                    introspection_client_id: r.string(64)?,
                });
            }
            ServicePolicy::new_issuer(home, issuer, entries, routes)?
        } else {
            if entries.iter().any(|(_, u)| *u != ServiceUse::Observation) {
                return Err(Error::Invalid);
            }
            ServicePolicy::new(home, entries.into_iter().map(|(e, _)| e).collect())?
        });
    }
    if r.offset != raw.len() {
        return Err(Error::Invalid);
    }
    s.validate()?;
    Ok(s)
}

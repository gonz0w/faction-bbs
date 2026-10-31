//! V4 retains device bytes and canonical service history; only public policy persists.
use super::{Error, MAX_BINDINGS, MAX_SNAPSHOT, Reader, Snapshot, Writer};
use crate::services::{
    IssuerRoute, RemoteFloor, ResourceAssociation, ServiceIdentity, ServicePolicy,
    ServiceRegistration, ServiceUse,
};
pub(super) fn encode(s: &Snapshot) -> Result<Vec<u8>, Error> {
    s.validate()?;
    let mut device = s.clone();
    device.service_format_v2 = false;
    device.service_format_v3 = false;
    device.service_format_v4 = false;
    device.service_history.clear();
    device.service_policy = None;
    device.remote_history.clear();
    let original = super::encode(&device)?;
    let mut w = Writer(b"FPPY0004".to_vec());
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
                ServiceUse::ResourceClient => 3,
                ServiceUse::OutboundIntrospection => 4,
                ServiceUse::IssuerEndpoint => 5,
            });
        }
        w.0.push(u8::from(p.issuer.is_some()));
        if let Some(i) = &p.issuer {
            w.string(&i.issuer);
            w.0.push(i.routes.len() as u8);
            for r in &i.routes {
                w.string(&r.sender_home);
                w.string(&r.audience);
                w.string(&r.relay_client_id);
                w.string(&r.introspection_client_id);
            }
        }
        w.0.push(u8::from(p.resource.is_some()));
        if let Some(associations) = &p.resource {
            w.0.push(associations.len() as u8);
            for a in associations {
                w.string(&a.recipient_home);
                w.string(&a.issuer);
                w.string(&a.audience);
                w.string(&a.relay_client_id);
                w.string(&a.outbound_client_id);
                w.string(&a.endpoint_id);
                w.string(&a.address.to_string());
                w.string(&a.http_host);
                w.0.extend(a.ca_sha256);
                w.u64(a.remote_generation);
                w.u64(a.remote_reset);
            }
        }
    }
    w.0.push(s.remote_history.len() as u8);
    for r in &s.remote_history {
        w.string(&r.home);
        w.u64(r.generation);
        w.u64(r.reset);
    }
    if w.0.len() > MAX_SNAPSHOT {
        return Err(Error::Invalid);
    }
    Ok(w.0)
}
pub(super) fn decode(raw: &[u8]) -> Result<Snapshot, Error> {
    if raw.len() > MAX_SNAPSHOT || raw.get(..8) != Some(b"FPPY0004") {
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
    s.service_format_v4 = true;
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
                3 => ServiceUse::ResourceClient,
                4 => ServiceUse::OutboundIntrospection,
                5 => ServiceUse::IssuerEndpoint,
                _ => return Err(Error::Invalid),
            };
            entries.push((e, purpose));
        }
        let issuer = if r.boolean()? {
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
            Some((issuer, routes))
        } else {
            None
        };
        let resource = if r.boolean()? {
            let mut associations = Vec::new();
            for _ in 0..r.count(MAX_BINDINGS)? {
                associations.push(ResourceAssociation {
                    recipient_home: r.string(189)?,
                    issuer: r.string(253)?,
                    audience: r.string(254)?,
                    relay_client_id: r.string(64)?,
                    outbound_client_id: r.string(64)?,
                    endpoint_id: r.string(254)?,
                    address: r.string(128)?.parse().map_err(|_| Error::Invalid)?,
                    http_host: r.string(253)?,
                    ca_sha256: r.id()?,
                    remote_generation: r.u64()?,
                    remote_reset: r.u64()?,
                });
            }
            Some(associations)
        } else {
            None
        };
        s.service_policy = Some(match (issuer, resource) {
            (None, Some(associations)) => ServicePolicy::new_resource(home, entries, associations)?,
            (Some((issuer, routes)), None) => {
                ServicePolicy::new_issuer(home, issuer, entries, routes)?
            }
            (None, None) => {
                if entries.iter().any(|(_, u)| *u != ServiceUse::Observation) {
                    return Err(Error::Invalid);
                }
                ServicePolicy::new(home, entries.into_iter().map(|(e, _)| e).collect())?
            }
            (Some((issuer, routes)), Some(associations)) => {
                ServicePolicy::new_composed(home, issuer, entries, routes, associations)?
            }
        });
    }
    for _ in 0..r.count(crate::authority::MAX_HOMES)? {
        s.remote_history.push(RemoteFloor {
            home: r.string(189)?,
            generation: r.u64()?,
            reset: r.u64()?,
        });
    }
    if r.offset != raw.len() {
        return Err(Error::Invalid);
    }
    s.validate()?;
    Ok(s)
}

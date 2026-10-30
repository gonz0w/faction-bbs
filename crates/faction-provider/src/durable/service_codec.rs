//! v2 keeps the complete byte-defined v1 device section and adds separately
//! typed service mappings under the same aggregate validation and byte ceiling.
use super::{Error, MAX_BINDINGS, MAX_SNAPSHOT, Reader, Snapshot, Writer};
use crate::services::{ServiceIdentity, ServicePolicy, ServiceRegistration};
pub(super) fn encode(s: &Snapshot) -> Result<Vec<u8>, Error> {
    s.validate()?;
    let mut device = s.clone();
    device.service_format_v2 = false;
    device.service_history.clear();
    device.service_policy = None;
    let original = super::encode(&device)?;
    let mut w = Writer(b"FPPY0002".to_vec());
    w.0.extend((original.len() as u32).to_be_bytes());
    w.0.extend(original);
    w.0.push(s.service_history.len() as u8);
    for (digest, identity) in &s.service_history {
        w.0.extend(digest);
        w.string(&identity.home);
        w.string(&identity.client_id);
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
        }
    }
    if w.0.len() > MAX_SNAPSHOT {
        return Err(Error::Invalid);
    }
    Ok(w.0)
}
pub(super) fn decode(raw: &[u8]) -> Result<Snapshot, Error> {
    if raw.len() > MAX_SNAPSHOT || raw.get(..8) != Some(b"FPPY0002") {
        return Err(Error::Invalid);
    }
    let mut r = Reader { raw, offset: 8 };
    let length = u32::from_be_bytes(r.take(4)?.try_into().map_err(|_| Error::Invalid)?) as usize;
    let original = r.take(length)?;
    // Exactly one original v1 section: nested v2 or ambiguous encodings deny.
    if original.get(..8) != Some(b"FPPY0001") {
        return Err(Error::Invalid);
    }
    let mut s = super::decode(original)?;
    s.service_format_v2 = true;
    let count = r.count(MAX_BINDINGS)?;
    for _ in 0..count {
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
        let count = r.count(MAX_BINDINGS)?;
        let mut entries = Vec::with_capacity(count);
        for _ in 0..count {
            entries.push(ServiceRegistration {
                home: r.string(189)?,
                client_id: r.string(254)?,
                dns_san: r.string(189)?,
                certificate_sha256: r.id()?,
                enabled: r.boolean()?,
                valid_from: r.u64()?,
                valid_until: r.u64()?,
            });
        }
        s.service_policy = Some(ServicePolicy::new(home, entries)?);
    }
    if r.offset != raw.len() {
        return Err(Error::Invalid);
    }
    s.validate()?;
    Ok(s)
}

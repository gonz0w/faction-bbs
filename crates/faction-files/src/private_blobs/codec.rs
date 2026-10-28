//! Canonical fixed-version binary frames. Input length is checked before parsing;
//! every allocation has a checked field cap. Actor authentication is never encoded.
use super::*;
const REQUEST: &[u8; 8] = b"FBRQ0001";
const RESPONSE: &[u8; 8] = b"FBRP0001";
const SNAPSHOT: &[u8; 8] = b"FBCS0001";
struct Writer(Vec<u8>);
impl Writer {
    fn u64(&mut self, n: u64) {
        self.0.extend(n.to_be_bytes())
    }
    fn principal(&mut self, p: &Principal) {
        self.0.extend((p.account.len() as u16).to_be_bytes());
        self.0.extend(p.account.as_bytes());
        self.0.extend(p.device.to_be_bytes());
        self.0.extend(p.identity)
    }
    fn binding(&mut self, b: &Binding) {
        self.principal(&b.sender);
        self.principal(&b.recipient);
        self.0.extend(b.id);
        self.u64(b.size);
        self.u64(b.created);
        self.u64(b.expires)
    }
    fn bytes(&mut self, b: &[u8]) {
        self.0.extend((b.len() as u32).to_be_bytes());
        self.0.extend(b)
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
    fn u32(&mut self) -> Result<u32, Error> {
        Ok(u32::from_be_bytes(
            self.take(4)?.try_into().map_err(|_| Error::Invalid)?,
        ))
    }
    fn u64(&mut self) -> Result<u64, Error> {
        Ok(u64::from_be_bytes(
            self.take(8)?.try_into().map_err(|_| Error::Invalid)?,
        ))
    }
    fn id(&mut self) -> Result<[u8; 32], Error> {
        self.take(32)?.try_into().map_err(|_| Error::Invalid)
    }
    fn principal(&mut self) -> Result<Principal, Error> {
        let n = u16::from_be_bytes(self.take(2)?.try_into().map_err(|_| Error::Invalid)?) as usize;
        if n > 254 {
            return Err(Error::Invalid);
        }
        let account = std::str::from_utf8(self.take(n)?)
            .map_err(|_| Error::Invalid)?
            .to_owned();
        let p = Principal {
            account,
            device: self.u32()?,
            identity: self.id()?,
        };
        p.validate()?;
        Ok(p)
    }
    fn binding(&mut self) -> Result<Binding, Error> {
        let b = Binding {
            sender: self.principal()?,
            recipient: self.principal()?,
            id: self.id()?,
            size: self.u64()?,
            created: self.u64()?,
            expires: self.u64()?,
        };
        b.validate()?;
        Ok(b)
    }
    fn bytes(&mut self, cap: usize) -> Result<Vec<u8>, Error> {
        let n = self.u32()? as usize;
        if n > cap {
            return Err(Error::Invalid);
        }
        Ok(self.take(n)?.to_vec())
    }
    fn end(&self) -> Result<(), Error> {
        if self.offset == self.raw.len() {
            Ok(())
        } else {
            Err(Error::Invalid)
        }
    }
}
fn reader<'a>(raw: &'a [u8], magic: &[u8; 8], cap: usize) -> Result<Reader<'a>, Error> {
    if raw.len() > cap || raw.get(..8) != Some(magic) {
        return Err(Error::Invalid);
    }
    Ok(Reader { raw, offset: 8 })
}
fn valid_request(r: &Request) -> Result<(), Error> {
    match r {
        Request::Begin(b) => b.validate(),
        Request::PutChunk { offset, bytes, .. } => {
            if bytes.is_empty()
                || bytes.len() > CHUNK
                || offset
                    .checked_add(bytes.len() as u64)
                    .is_none_or(|n| n > MAX_CIPHERTEXT as u64)
            {
                Err(Error::Invalid)
            } else {
                Ok(())
            }
        }
        Request::GetChunk { offset, .. } => {
            if *offset > MAX_CIPHERTEXT as u64 {
                Err(Error::Invalid)
            } else {
                Ok(())
            }
        }
        _ => Ok(()),
    }
}
fn valid_response(r: &Response) -> Result<(), Error> {
    match r {
        Response::Progress {
            binding,
            offset,
            committed,
        } => {
            binding.validate()?;
            if *offset > binding.size || (*committed && *offset != binding.size) {
                Err(Error::Invalid)
            } else {
                Ok(())
            }
        }
        Response::Chunk {
            offset,
            bytes,
            total,
        } => {
            if !(16..=MAX_CIPHERTEXT as u64).contains(total)
                || bytes.len() > CHUNK
                || offset
                    .checked_add(bytes.len() as u64)
                    .is_none_or(|n| n > *total)
                || (bytes.is_empty() && offset != total)
            {
                Err(Error::Invalid)
            } else {
                Ok(())
            }
        }
        Response::Deleted => Ok(()),
    }
}
/// Encode only public routing metadata and ciphertext; never an authenticated actor.
pub fn encode_request(request: &Request) -> Result<Vec<u8>, Error> {
    valid_request(request)?;
    let mut w = Writer(REQUEST.to_vec());
    match request {
        Request::Begin(b) => {
            w.0.push(1);
            w.binding(b)
        }
        Request::Status { id } => {
            w.0.push(2);
            w.0.extend(id)
        }
        Request::PutChunk { id, offset, bytes } => {
            w.0.push(3);
            w.0.extend(id);
            w.u64(*offset);
            w.bytes(bytes)
        }
        Request::Commit { id } => {
            w.0.push(4);
            w.0.extend(id)
        }
        Request::GetChunk { id, offset } => {
            w.0.push(5);
            w.0.extend(id);
            w.u64(*offset)
        }
        Request::Delete { id } => {
            w.0.push(6);
            w.0.extend(id)
        }
    }
    if w.0.len() > MAX_WIRE {
        return Err(Error::Invalid);
    }
    Ok(w.0)
}
pub fn decode_request(raw: &[u8]) -> Result<Request, Error> {
    let mut r = reader(raw, REQUEST, MAX_WIRE)?;
    let request = match r.byte()? {
        1 => Request::Begin(r.binding()?),
        2 => Request::Status { id: r.id()? },
        3 => Request::PutChunk {
            id: r.id()?,
            offset: r.u64()?,
            bytes: r.bytes(CHUNK)?,
        },
        4 => Request::Commit { id: r.id()? },
        5 => Request::GetChunk {
            id: r.id()?,
            offset: r.u64()?,
        },
        6 => Request::Delete { id: r.id()? },
        _ => return Err(Error::Invalid),
    };
    r.end()?;
    valid_request(&request)?;
    Ok(request)
}
pub fn encode_response(response: &Response) -> Result<Vec<u8>, Error> {
    valid_response(response)?;
    let mut w = Writer(RESPONSE.to_vec());
    match response {
        Response::Progress {
            binding,
            offset,
            committed,
        } => {
            w.0.push(1);
            w.binding(binding);
            w.u64(*offset);
            w.0.push(u8::from(*committed))
        }
        Response::Chunk {
            offset,
            bytes,
            total,
        } => {
            w.0.push(2);
            w.u64(*offset);
            w.u64(*total);
            w.bytes(bytes)
        }
        Response::Deleted => w.0.push(3),
    }
    if w.0.len() > MAX_WIRE {
        return Err(Error::Invalid);
    }
    Ok(w.0)
}
pub fn decode_response(raw: &[u8]) -> Result<Response, Error> {
    let mut r = reader(raw, RESPONSE, MAX_WIRE)?;
    let response = match r.byte()? {
        1 => Response::Progress {
            binding: r.binding()?,
            offset: r.u64()?,
            committed: r.boolean()?,
        },
        2 => Response::Chunk {
            offset: r.u64()?,
            total: r.u64()?,
            bytes: r.bytes(CHUNK)?,
        },
        3 => Response::Deleted,
        _ => return Err(Error::Invalid),
    };
    r.end()?;
    valid_response(&response)?;
    Ok(response)
}
pub(super) fn encode_state(state: &State) -> Result<Vec<u8>, Error> {
    state.validate()?;
    let mut w = Writer(SNAPSHOT.to_vec());
    w.u64(state.clock);
    w.0.push(state.objects.len() as u8);
    for o in &state.objects {
        w.binding(&o.binding);
        w.0.push(u8::from(o.committed));
        w.bytes(&o.bytes)
    }
    if w.0.len() > MAX_SNAPSHOT {
        return Err(Error::Invalid);
    }
    Ok(w.0)
}
pub(super) fn decode_state(raw: &[u8]) -> Result<State, Error> {
    let mut r = reader(raw, SNAPSHOT, MAX_SNAPSHOT)?;
    let clock = r.u64()?;
    let count = r.byte()? as usize;
    if count > MAX_OBJECTS {
        return Err(Error::Invalid);
    }
    let mut objects = Vec::with_capacity(count);
    for _ in 0..count {
        let binding = r.binding()?;
        let committed = r.boolean()?;
        let bytes = r.bytes(binding.size as usize)?;
        objects.push(Object {
            binding,
            committed,
            bytes,
        })
    }
    r.end()?;
    if objects
        .windows(2)
        .any(|p| p[0].binding.id >= p[1].binding.id)
    {
        return Err(Error::Invalid);
    }
    let state = State { clock, objects };
    state.validate()?;
    Ok(state)
}

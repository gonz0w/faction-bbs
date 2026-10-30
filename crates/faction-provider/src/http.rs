//! One consuming, bounded HTTP operation on an existing authenticated TLS session.
use super::*;
use crate::issuer::HttpInput;
use faction_delegation_codec as codec;
fn read_http(reader: &mut impl Read) -> Result<(codec::UnverifiedRequestHead, Vec<u8>), Stage> {
    let mut header = [0; codec::MAX_HEADER];
    let mut used = 0;
    let end = loop {
        if used == header.len() {
            return Err(Stage::Frame);
        }
        let count = reader.read(&mut header[used..]).map_err(|_| Stage::Frame)?;
        if count == 0 {
            return Err(Stage::Frame);
        }
        used += count;
        if let Some(index) = header[..used].windows(4).position(|w| w == b"\r\n\r\n") {
            break index + 4;
        }
    };
    let head = codec::decode_http_request_head(&header[..end]).map_err(|_| Stage::Frame)?;
    let total = end.checked_add(head.content_length).ok_or(Stage::Frame)?;
    if head.content_length > codec::MAX_FORM || used > total {
        return Err(Stage::Frame);
    }
    // Length/framing and buffered surplus have been checked before allocating.
    let mut bytes = Vec::with_capacity(total);
    bytes.extend_from_slice(&header[..used]);
    let mut scratch = [0; codec::MAX_FORM + 1];
    while bytes.len() < total {
        let bound = (total - bytes.len() + 1).min(scratch.len());
        let count = reader
            .read(&mut scratch[..bound])
            .map_err(|_| Stage::Frame)?;
        if count == 0 || count > total - bytes.len() {
            return Err(Stage::Frame);
        }
        bytes.extend_from_slice(&scratch[..count]);
    }
    Ok((head, bytes))
}
impl ServiceSession {
    /// Consume one operation and close/release the carrier on every outcome.
    /// Parsing supplies unverified data. Current private issuer policy authorizes it.
    /// ```compile_fail
    /// fn reuse(s:faction_provider::ServiceSession){s.serve_http_once();s.serve_http_once();}
    /// ```
    pub fn serve_http_once(mut self) -> Result<(), Error> {
        self.serve_http_inner()
    }
    fn serve_http_inner(&mut self) -> Result<(), Error> {
        let state = self.issuer_attempt()?;
        let decoded = read_http(&mut self.stream);
        self.issuer_check(&state)?;
        let reply = match decoded {
            Ok((head, bytes)) if self.stream.ssl().pending() == 0 => {
                let input = match head.endpoint {
                    codec::UnverifiedEndpoint::Token => {
                        codec::decode_token_http(&bytes).map(HttpInput::Token)
                    }
                    codec::UnverifiedEndpoint::Introspection => {
                        codec::decode_introspection_http(&bytes).map(HttpInput::Introspection)
                    }
                };
                match input {
                    Ok(input) => self.http_dispatch_charged(&state, &head, input)?,
                    Err(_) => crate::issuer::HttpReply {
                        bytes: codec::encode_refusal_http(codec::UnverifiedRefusal::InvalidRequest),
                        deadline: None,
                        wall_expires: None,
                        record: None,
                        failure: Some(Error::Transport(Stage::Frame)),
                    },
                }
            }
            _ => crate::issuer::HttpReply {
                bytes: codec::encode_refusal_http(codec::UnverifiedRefusal::InvalidRequest),
                deadline: None,
                wall_expires: None,
                record: None,
                failure: Some(Error::Transport(Stage::Frame)),
            },
        };
        if let Some(deadline) = reply.deadline {
            self.deadline = self.deadline.min(deadline);
            self.stream.get_mut().deadline = self.deadline;
        }
        // Policy and cancellation are checked after all reads and immediately
        // before output. No authority lock survives into either network phase.
        let wall_checked = self.http_write_current(&state, reply.record.as_ref())?;
        if let Some(expires) = reply.wall_expires {
            self.stream.get_mut().wire_lifetime = Some(WireLifetime {
                expires,
                highwater: wall_checked,
                invalid: false,
            });
        }
        #[cfg(test)]
        {
            self.stream.get_mut().response_phase = true;
        }
        self.stream
            .write_all(&reply.bytes)
            .map_err(|_| Error::Transport(classify(self.deadline, &self.cancel, Stage::Frame)))?;
        self.stream
            .flush()
            .map_err(|_| Error::Transport(classify(self.deadline, &self.cancel, Stage::Frame)))?;
        self.issuer_check(&state)?;
        match reply.failure {
            Some(error) => Err(error),
            None => Ok(()),
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn message() -> Vec<u8> {
        codec::encode_token_http(
            "issuer.test",
            &codec::UnverifiedTokenRequest {
                client_id: "relay".into(),
                scope: codec::UnverifiedReference::from_bytes([1; 32]),
            },
        )
        .unwrap()
    }
    #[test]
    fn buffered_surplus_and_pipeline_refuse_before_a_dispatch_can_receive_bytes() {
        let bytes = message();
        for suffix in [b"x".as_slice(), bytes.as_slice()] {
            let mut extra = bytes.clone();
            extra.extend_from_slice(suffix);
            assert!(read_http(&mut io::Cursor::new(extra)).is_err());
        }
    }
    #[test]
    fn exact_header_body_bounds_and_declared_lengths_are_checked_before_allocation() {
        let raw = String::from_utf8(message()).unwrap();
        let end = raw.find("\r\n\r\n").unwrap() + 4;
        let body = raw.len() - end;
        let bad = raw.replace(&format!("Content-Length: {body}"), "Content-Length: 8193");
        assert!(read_http(&mut io::Cursor::new(bad.as_bytes())).is_err());
        let bad = raw.replace(
            "\r\nHost:",
            &format!("\r\nX-Pad: {}\r\nHost:", "x".repeat(8192)),
        );
        assert!(read_http(&mut io::Cursor::new(bad.as_bytes())).is_err());
        for end in [0, 2, raw.len() - 1] {
            assert!(read_http(&mut io::Cursor::new(&raw.as_bytes()[..end])).is_err());
        }
    }
    #[test]
    fn fragmented_reader_does_not_require_eof_or_restart_framing() {
        struct Fragmented(io::Cursor<Vec<u8>>);
        impl Read for Fragmented {
            fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
                let n = out.len().min(3);
                self.0.read(&mut out[..n])
            }
        }
        let bytes = message();
        let (_, got) = read_http(&mut Fragmented(io::Cursor::new(bytes.clone()))).unwrap();
        assert_eq!(got, bytes);
    }
}
#[cfg(test)]
#[path = "http_tls_tests.rs"]
mod tls_tests;

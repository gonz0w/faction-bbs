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
                    codec::UnverifiedEndpoint::PublicationCurrent => Err(codec::Error),
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

fn array(hex: &str) -> Result<[u8; 32], Stage> {
    if hex.len() != 64 {
        return Err(Stage::Frame);
    }
    let mut result = [0; 32];
    for (i, pair) in hex.as_bytes().chunks_exact(2).enumerate() {
        let text = std::str::from_utf8(pair).map_err(|_| Stage::Frame)?;
        result[i] = u8::from_str_radix(text, 16).map_err(|_| Stage::Frame)?;
    }
    Ok(result)
}
fn publication_input(
    q: &codec::UnverifiedPublicationQuery,
) -> Result<
    (
        faction_files::private_blobs::Binding,
        crate::services::PublicationAction,
    ),
    Stage,
> {
    use crate::services::PublicationAction as A;
    use faction_files::private_blobs::{Binding, Principal};
    let p = |p: &codec::UnverifiedPrincipal| -> Result<Principal, Stage> {
        Ok(Principal {
            account: p.account.clone(),
            device: p.device,
            identity: array(&p.identity)?,
        })
    };
    let b = &q.binding;
    let binding = Binding {
        sender: p(&b.sender)?,
        recipient: p(&b.recipient)?,
        id: array(&b.id)?,
        size: b.size,
        created: b.created,
        expires: b.expires,
    };
    let action = match &q.action {
        codec::UnverifiedPublicationAction::Begin => A::Begin,
        codec::UnverifiedPublicationAction::PutChunk {
            offset,
            length,
            ciphertext_sha256,
        } => A::PutChunk {
            offset: *offset,
            length: *length,
            ciphertext_sha256: array(ciphertext_sha256)?,
        },
        codec::UnverifiedPublicationAction::Commit => A::Commit,
        codec::UnverifiedPublicationAction::Status => A::Status,
        codec::UnverifiedPublicationAction::Delete => A::Delete,
    };
    Ok((binding, action))
}
/// One precise observation history for the whole publication carrier. No
/// timestamp is invented before the first actual check; failed time stays failed.
#[derive(Default)]
struct PublicationClock {
    lifetime: Option<WireLifetime>,
    invalid: bool,
}
impl PublicationClock {
    fn observe(&mut self, now: Option<Duration>) -> bool {
        if self.invalid {
            return false;
        }
        let Some(now) = now else {
            self.invalid = true;
            return false;
        };
        let lifetime = self.lifetime.get_or_insert(WireLifetime {
            expires: Duration::MAX,
            highwater: now,
            invalid: false,
        });
        if lifetime.check(now).is_err() {
            self.invalid = true;
            return false;
        }
        true
    }
}
impl ServiceSession {
    fn publication_probe(
        &self,
        profile: &crate::services::PublicationWireProfile,
        query: Option<codec::UnverifiedPublicationQuery>,
        clock_state: Arc<std::sync::Mutex<PublicationClock>>,
    ) -> Arc<dyn Fn() -> bool + Send + Sync> {
        self.publication_probe_clock(
            profile,
            query,
            Arc::new(|| SystemTime::now().duration_since(UNIX_EPOCH).ok()),
            clock_state,
        )
    }
    fn publication_probe_clock(
        &self,
        profile: &crate::services::PublicationWireProfile,
        query: Option<codec::UnverifiedPublicationQuery>,
        clock: Arc<dyn Fn() -> Option<Duration> + Send + Sync>,
        clock_state: Arc<std::sync::Mutex<PublicationClock>>,
    ) -> Arc<dyn Fn() -> bool + Send + Sync> {
        let state = self.state.clone();
        let profile = profile.clone();
        let credential = self.credential.clone();
        let leaf = self.leaf.clone();
        Arc::new(move || {
            let wall = clock();
            // Drop the private clock lock before acquiring authority.
            if !clock_state.lock().is_ok_and(|mut c| c.observe(wall)) {
                return false;
            }
            let Some(state) = state.upgrade() else {
                return false;
            };
            if state.stopped.load(Ordering::SeqCst) {
                return false;
            }
            let Ok(a) = state.authority.lock() else {
                return false;
            };
            if state.stopped.load(Ordering::SeqCst) {
                return false;
            }
            // Authority may have been contended past an expiry. Preserve both
            // precise samples in the same sticky history, then use fresh time.
            let wall = clock();
            if !clock_state.lock().is_ok_and(|mut c| c.observe(wall)) {
                return false;
            }
            let Some(wall) = wall else {
                return false;
            };
            let now = wall.as_secs();
            if profile.current(&a, &credential, &leaf, now).is_err() {
                return false;
            }
            if let Some(q) = &query {
                if q.client_id != credential.identity.client_id {
                    return false;
                }
                let Ok((binding, action)) = publication_input(q) else {
                    return false;
                };
                return a
                    .observe_recipient(&mut credential.clone(), &leaf, binding, action, now)
                    .is_ok();
            }
            true
        })
    }
    /// One exact publication query; the profile only restricts installed policy.
    /// The original incoming carrier is consumed on every result.
    pub fn serve_publication_http_once(
        mut self,
        profile: &crate::services::PublicationWireProfile,
    ) -> Result<(), Error> {
        let state = self.issuer_attempt()?;
        let clock_state = Arc::new(std::sync::Mutex::new(PublicationClock::default()));
        let probe = self.publication_probe(profile, None, clock_state.clone());
        if !probe() {
            return Err(Error::Transport(Stage::Denied));
        }
        self.stream.get_mut().policy_probe = Some(probe);
        let decoded = read_http(&mut self.stream);
        self.issuer_check(&state)?;
        let query = decoded.ok().and_then(|(head, bytes)| {
            if head.host != profile.host()
                || head.endpoint != codec::UnverifiedEndpoint::PublicationCurrent
                || self.stream.ssl().pending() != 0
            {
                return None;
            }
            codec::decode_publication_query_http(&bytes).ok()
        });
        let result = query.as_ref().and_then(|q| {
            if q.client_id != self.credential.identity.client_id {
                return None;
            }
            let (binding, action) = publication_input(q).ok()?;
            let a = state.authority.lock().ok()?;
            profile
                .current(&a, &self.credential, &self.leaf, wall_now())
                .ok()?;
            a.observe_recipient(
                &mut self.credential,
                &self.leaf,
                binding,
                action,
                wall_now(),
            )
            .ok()
        });
        let observation = if let (Some(q), Some(o)) = (query.as_ref(), result.as_ref()) {
            let hex = |v: [u8; 32]| v.iter().map(|b| format!("{b:02x}")).collect();
            self.stream.get_mut().policy_probe =
                Some(self.publication_probe(profile, Some(q.clone()), clock_state));
            codec::UnverifiedPublicationObservation::Current {
                query: Box::new(q.clone()),
                home: profile.host().into(),
                service_home: o.service_home().into(),
                service_certificate_sha256: hex(o.service_certificate_sha256()),
                recipient_certificate_sha256: hex(o.recipient_certificate_sha256()),
                generation: o.generation(),
                reset: o.reset_epoch(),
                checked_at: o.checked_at(),
            }
        } else {
            codec::UnverifiedPublicationObservation::Unavailable
        };
        let bytes = codec::encode_publication_response_http(&observation)
            .map_err(|_| Error::Transport(Stage::Frame))?;
        self.issuer_check(&state)?;
        #[cfg(test)]
        {
            self.stream.get_mut().response_phase = true;
        }
        self.stream
            .write_all(&bytes)
            .and_then(|_| self.stream.flush())
            .map_err(|_| Error::Transport(classify(self.deadline, &self.cancel, Stage::Denied)))?;
        self.issuer_check(&state)?;
        if result.is_some() {
            Ok(())
        } else {
            Err(Error::Transport(Stage::Denied))
        }
    }
}

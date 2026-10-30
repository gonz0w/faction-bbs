//! Bounded unverified wire data reaches the same current issuer checks as core calls.
use super::*;
use faction_delegation_codec as codec;
pub(crate) enum HttpInput {
    Token(codec::UnverifiedTokenRequest),
    Introspection(codec::UnverifiedIntrospectionRequest),
}
pub(crate) struct HttpReply {
    pub(crate) bytes: Vec<u8>,
    pub(crate) deadline: Option<Instant>,
    pub(crate) wall_expires: Option<Duration>,
    pub(crate) record: Option<Arc<()>>,
    pub(crate) failure: Option<Error>,
}
fn hex(bytes: &[u8; 32]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
fn principal(p: &Principal) -> codec::UnverifiedPrincipal {
    codec::UnverifiedPrincipal {
        account: p.account.clone(),
        device: p.device,
        identity: hex(&p.identity),
    }
}
fn claims(m: RetrievalMetadata) -> codec::UnverifiedIntrospection {
    let binding = codec::UnverifiedBinding {
        sender: principal(&m.binding.sender),
        recipient: principal(&m.binding.recipient),
        id: hex(&m.binding.id),
        size: m.binding.size,
        created: m.binding.created,
        expires: m.binding.expires,
    };
    codec::UnverifiedIntrospection::Retrieval {
        claims: codec::UnverifiedClaims {
            issuer: m.issuer,
            issuer_home: m.issuer_home,
            client_id: m.client_id,
            audience: m.audience,
            subject: m.binding.recipient.account,
            issued_at: m.issued_at,
            expires_at: m.expires_at,
            generation: m.generation,
            reset: m.reset_epoch,
            certificate_thumbprint: codec::UnverifiedReference::from_bytes(
                m.relay_certificate_sha256,
            ),
            binding,
        },
        action: match m.action {
            RetrievalAction::GetChunk { offset } => {
                codec::UnverifiedRetrievalAction::GetChunk { offset }
            }
            RetrievalAction::Status => codec::UnverifiedRetrievalAction::Status,
        },
    }
}
impl tls::ServiceSession {
    pub(crate) fn http_dispatch_charged(
        &mut self,
        state: &State,
        head: &codec::UnverifiedRequestHead,
        input: HttpInput,
    ) -> Result<HttpReply, Error> {
        self.issuer_check(state)?;
        let mut a = state
            .authority
            .lock()
            .map_err(|_| Error::Transport(tls::Stage::Denied))?;
        if state.stopped.load(Ordering::SeqCst) {
            return Err(Error::Transport(tls::Stage::Denied));
        }
        if a.service_policy
            .as_ref()
            .and_then(|p| p.issuer.as_ref())
            .is_none_or(|p| p.issuer != head.host)
        {
            return Ok(HttpReply {
                bytes: codec::encode_refusal_http(codec::UnverifiedRefusal::InvalidRequest),
                deadline: None,
                wall_expires: None,
                record: None,
                failure: Some(Error::Transport(tls::Stage::Frame)),
            });
        }
        // Resolve only a deadline here, never an actor or a grant. Even refusal
        // output for a resolved operation remains inside its original lifetime.
        let record = match &input {
            HttpInput::Token(r) => a
                .records
                .slots
                .iter()
                .find(|s| {
                    s.issuer
                        .as_ref()
                        .is_some_and(|i| memcmp::eq(&i.reference, r.scope.expose_bytes()))
                })
                .map(|s| s.token.clone()),
            HttpInput::Introspection(r) => r.token.decode_bytes().ok().and_then(|raw| {
                a.records
                    .slots
                    .iter()
                    .find(|s| {
                        s.issuer
                            .as_ref()
                            .and_then(|i| i.token.as_ref())
                            .is_some_and(|t| memcmp::eq(&t.secret, &raw))
                    })
                    .map(|s| s.token.clone())
            }),
        };
        let bound = |a: &Authority| -> Result<Option<(Instant, Duration)>, BlobError> {
            let Some(slot) = record
                .as_ref()
                .and_then(|key| a.records.slots.iter().find(|s| Arc::ptr_eq(key, &s.token)))
            else {
                return Ok(None);
            };
            let mut cap = slot.binding.expires.min(certificate_expiry(&slot.leaf)?);
            if let Some(device) = a
                .registry
                .as_ref()
                .and_then(|r| r.bindings().iter().find(|b| b.principal == slot.principal))
            {
                cap = cap.min(device.valid_until);
            }
            let mut deadline = slot.deadline;
            if let Some(reference) = &slot.issuer {
                cap = cap.min(reference.relay.valid_until);
                if let Some(token) = &reference.token {
                    cap = cap.min(token.expires_at);
                    deadline = deadline.min(token.deadline);
                }
            }
            Ok(Some((deadline, Duration::from_secs(cap))))
        };
        let before = bound(&a).map_err(Error::Store)?;
        let now = a.live_moment().map_err(Error::Store)?;
        if before.is_some_and(|(deadline, wall)| now.after >= deadline || now.wall >= wall) {
            return Err(Error::Transport(tls::Stage::Deadline));
        }
        let (bytes, failure) = match input {
            HttpInput::Token(r) => {
                let first = a.issue_at(
                    &mut self.credential,
                    &self.leaf,
                    r.scope.expose_bytes(),
                    &r.client_id,
                    now,
                    &mut secure_random,
                );
                let issued = if first.is_ok() {
                    let now = a.live_moment().map_err(Error::Store)?;
                    a.issue_at(
                        &mut self.credential,
                        &self.leaf,
                        r.scope.expose_bytes(),
                        &r.client_id,
                        now,
                        &mut secure_random,
                    )
                } else {
                    first
                };
                match issued {
                    Ok(token) => {
                        let response = codec::UnverifiedTokenResponse {
                            token: codec::UnverifiedToken::from_bytes(
                                *token.token().expose_bytes(),
                            ),
                            expires_in: token.expires_in(),
                        };
                        (
                            codec::encode_token_response_http(&response)
                                .map_err(|_| Error::Transport(tls::Stage::Frame))?,
                            None,
                        )
                    }
                    Err(_) => (
                        codec::encode_refusal_http(codec::UnverifiedRefusal::InvalidToken),
                        Some(Error::Transport(tls::Stage::Denied)),
                    ),
                }
            }
            HttpInput::Introspection(r) => {
                let metadata = r.token.decode_bytes().ok().and_then(|token| {
                    a.introspect_at(&mut self.credential, &self.leaf, &token, &r.client_id, now)
                        .ok()
                });
                let result = metadata.map_or(codec::UnverifiedIntrospection::Inactive, claims);
                (
                    codec::encode_introspection_response_http(&result)
                        .map_err(|_| Error::Transport(tls::Stage::Frame))?,
                    None,
                )
            }
        };
        // First issuance created a token after the initial reference lookup.
        // Recover its fixed deadline without changing expiry, slots or authority.
        let after = bound(&a).map_err(Error::Store)?;
        let limits = match (before, after) {
            (Some((d, w)), Some((d2, w2))) => Some((d.min(d2), w.min(w2))),
            (old, current) => old.or(current),
        };
        self.issuer_check(state)?;
        Ok(HttpReply {
            bytes,
            deadline: limits.map(|(d, _)| d),
            wall_expires: limits.map(|(_, w)| w),
            record,
            failure,
        })
    }
    pub(crate) fn http_write_current(
        &mut self,
        state: &State,
        record: Option<&Arc<()>>,
    ) -> Result<Duration, Error> {
        self.issuer_check(state)?;
        let mut a = state
            .authority
            .lock()
            .map_err(|_| Error::Transport(tls::Stage::Denied))?;
        let now = a.live_moment().map_err(Error::Store)?;
        a.issuer_prune(now).map_err(Error::Store)?;
        if let Some(record) = record {
            let index = a.issuer_slot(record, now).map_err(Error::Store)?;
            a.reference_current(index, now.wall.as_secs())
                .map_err(Error::Store)?;
            if a.records.slots[index]
                .issuer
                .as_ref()
                .and_then(|r| r.token.as_ref())
                .is_some_and(|t| {
                    now.after >= t.deadline || now.wall >= Duration::from_secs(t.expires_at)
                })
            {
                return Err(Error::Transport(tls::Stage::Deadline));
            }
        }

        let purpose = a
            .service_policy
            .as_ref()
            .and_then(|p| p.service_use(&self.credential.identity))
            .ok_or(Error::Transport(tls::Stage::Denied))?;
        let id = self.credential.identity.client_id.clone();
        a.issuer_service(
            &mut self.credential,
            &self.leaf,
            &id,
            purpose,
            now.wall.as_secs(),
        )
        .map_err(|_| Error::Transport(tls::Stage::Denied))?;
        self.issuer_check(state)?;
        Ok(now.wall)
    }
}

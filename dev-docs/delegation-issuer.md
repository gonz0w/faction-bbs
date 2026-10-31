# Retrieval token issuer for embedders

The default-off `delegation-issuer` feature composes `delegation-records` and
`delegation-services` for a trusted library embedder. It does not enable an ordinary
client or operator, start a listener, or deploy a service. Device mail plaintext
and private encryption keys remain on the device.

## Policy and authenticated sessions

Construct `services::ServicePolicy::new_issuer` with the local home, issuer,
complete `(ServiceRegistration, ServiceUse)` entries and `IssuerRoute` values;
install the complete registry and policy with `Control::replace_delegation_policy`.
Each registration has one explicit use: `Observation`, `Relay`, or `Introspection`.
The relay is local; the introspection service belongs to the exact invited sender
home. Each route names both client IDs and the exact `sender-home/attachments`
audience. Issuer-role client IDs contain 1–64 lowercase ASCII letters, digits,
`.`, `_` or `-`. The observations-only `ServicePolicy::new` and its existing
client-ID rules remain unchanged. The single registration collection, certificate
identity/history, generation and reset floors and aggregate 32-entry credential,
history and home limits continue to apply.

A service session comes from `tls::Server::accept_service` and actual mutual TLS,
not a supplied client ID. The issuer checks the observed registered certificate,
role and asserted client ID against current policy. The provider carrier uses TLS
1.3 only; this does not establish RFC 7662 TLS 1.2 support.

## Provision, issue and inspect

Create an opaque `RetrievalRecord` from the approved device session as described
in [retrieval intents](retrieval-intents.md). The exact original session's strong
owners must have disappeared before `Control::provision_retrieval(&record)` can
return a sensitive `issuer::OperationReference`; beginning session cleanup alone
is insufficient. Public intent metadata cannot provision one. The record, route,
issuer and current relay and introspection policy are rechecked. The recipient
home on the device-derived record must equal the issuer policy's local home;
individually valid policies for different homes cannot be combined to issue it.

A verified relay `ServiceSession` calls `issue_retrieval(&reference, client_id)`.
The resulting `IssuedToken` exposes its `token()`, `expires_in()` and
`expires_at()`. The separately authenticated permitted introspection service calls
`introspect_retrieval(&token, client_id)`, which returns
`Result<Option<RetrievalMetadata>, Error>`. A successful `None` is a
metadata-free inactive result; handle an error as denial. Its getters describe the
exact binding, action, audience, issuer, relay client and SHA-256 digest of the
relay leaf observed at issuance, times, generation and reset epoch. Unknown,
expired and unauthorized tokens reveal no member metadata. Compare the full
returned tuple with the intended operation.

References and tokens are sensitive opaque values; `expose_bytes()` is explicit.
They have no public authority constructor or implicit `Debug`/`Display`. Protect
any exposed bytes. Metadata is not a device actor, Store `Access`, current sender
approval or proof of a committed object. It supplies no remote Store grant.

The existing eight aggregate record slots include untokenized records and allow at
most one token per record. An exact retry may return the same token after current
checks; its original expiry is not renewed and `expires_in()` reflects the
remaining conservatively rounded lifetime. Original session deadline, object,
device and relay certificate and registration expiry constrain it; no positive
whole-second lifetime means refusal. `GetChunk { offset }` retains the existing
valid end-of-file offset equal to the binding size. Current policy, invitations,
credentials, generation and reset are checked again. Device-record checks retain
their whole-second wall-clock high-water mark. Issuer checks additionally detect
decreases between precise issuer observations beginning with the first issuer
check, including a failed attempt. Changes not represented by those observations
are not detected; the original fixed monotonic record and token deadlines remain
binding. A detected rollback invalidates rather than revives authority. The original
single runtime lane, eight attempts, five-second admission and ten-second cycle
remain.

## Optional HTTP carrier

Enable `delegation-wire` explicitly to compose the issuer with the qualified
[delegation codec](delegation-codecs.md). On an actual authenticated service TLS
connection, `ServiceSession::serve_http_once(self) -> Result<(), Error>` consumes
the session, serves at most one token or introspection request and releases the
connection. It accepts no caller-selected host, actor or stream: the request Host
must match current issuer policy. This library interface does not start a listener.

One attempt is charged before reading, and the same private dispatcher applies
current role, certificate and issuer checks. HTTP headers and form/JSON bodies
are each bounded to 8192 bytes; the request head is checked before body allocation.
Current authority is rechecked after reads and before mutation and writes; writes
remain bounded by the applicable original deadline. Network reads and writes do
not hold the authority lock. Stop and cancellation retain the existing session
lifecycle.

The carrier does not wait for client EOF or drain to dispatch a complete request.
It rejects surplus already ingested and already-decrypted pending application bytes
before dispatch. A later arrival cannot retroactively invalidate a completed first
request, but can never trigger a second operation on that connection. The codec
accepts only `Connection: close` when that header is present and emits it; absent
headers do not enable reuse. No generic HTTP or OAuth interoperability is implied.
Only retrieval is mapped to wire introspection here. A separate explicit
[publication carrier](service-observations.md#optional-publication-http-carrier)
preserves exact actions and leaves this issuer interface and its Host derivation
unchanged; it does not collapse observations into generic `Publish`.

The separate [retrieval verifier](delegation-verifier.md) consumes fresh
introspection for private resource dispatch. An explicit `ServicePolicy::new_composed`
combines issuer routes and resource associations in one canonical policy and
shared runtime; `new_issuer` retains its original restrictions.

## Durable policy

A deliberate complete higher-generation issuer policy update uses role-aware v3
state. Version 1 and version 2 are read without rewrite; v2 registrations import
as observations only. Builds without issuer support refuse active and tombstone v3
state without fallback or rewrite. Reset or device-only replacement disables issuer
policy while retaining the format marker and typed certificate history. Records,
references, tokens and sessions are transient and never persist; reopening requires
fresh authority and TLS. Existing snapshot, ownership and locking bounds remain.
The retained floors do not protect against replacing the entire trusted root with
an older valid copy. See [durable policy](provider-policy.md).

## Software test boundaries

Actual mutual TLS tests exercise issuance and introspection over the HTTP carrier.
A private post-handshake response BIO gate provides deterministic scheduling for
write, cancellation, policy and deadline controls; it does not demonstrate kernel
backpressure on a small TLS response. A separate existing plaintext socket-BIO
fixture observes actual write retry or timeout under kernel pressure. Precise clock
expiry and rollback controls use a synthetic clock, not a change to the host clock.
These controls do not establish deployment or hardware capacity.

The source-bound [host and NetBSD evidence](evidence/delegation-issuer/README.md)
records these software checks, their failures, private retention and owned cleanup.

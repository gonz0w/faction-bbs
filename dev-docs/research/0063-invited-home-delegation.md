# 0063: Invited-home attachment delegation

[Issue #299](https://github.com/gonz0w/faction-bbs/issues/299) examines the
settled [owner trust decision](https://github.com/gonz0w/faction-bbs/issues/294#issuecomment-6513127326).
Source was inspected at [Faction e2aeeb12dff7244edeb84dc208dd6927230114fc](https://github.com/gonz0w/faction-bbs/tree/e2aeeb12dff7244edeb84dc208dd6927230114fc)
and primary standards on 2026-10-30. This is source research, not a runtime,
deployment, interoperability, NetBSD or test result. Historical
[0061](0061-private-attachment-delivery-contract.md) remains unchanged.

## Decision

**GO to finite implementation qualification:** opaque OAuth access tokens,
online [RFC 7662 introspection](https://www.rfc-editor.org/rfc/rfc7662.html#section-2),
[RFC 8705 certificate-bound tokens](https://www.rfc-editor.org/rfc/rfc8705.html#section-3),
and [RFC 6749 client credentials](https://www.rfc-editor.org/rfc/rfc6749.html#section-4.4)
for explicitly arranged home-service permissions. This is not approval to enable
the unchanged provider. [#300](https://github.com/gonz0w/faction-bbs/issues/300)
qualifies inputs and authority, including remote-recipient checks;
[#301](https://github.com/gonz0w/faction-bbs/issues/301) depends on it for the
own-home relay and actual two-home proof. #294 and #88 remain open.

| Candidate | Assessment |
| --- | --- |
| Opaque tokens, online introspection and certificate binding | Selected for qualification: fresh online policy can support current revocation without JWT signing-key distribution. The issuer, device mapping, invitations and bounded executor remain missing. |
| [RFC 9068 JWT access tokens](https://www.rfc-editor.org/rfc/rfc9068.html#section-4), with [RFC 8693 token exchange](https://www.rfc-editor.org/rfc/rfc8693.html#section-2.1) if needed | Not selected: offline checks do not establish current device policy. Online checks remove that advantage while adding signing-key lifecycle and input-token trust. Exchange does not supply authenticated device input. |

An explicitly invited recipient home may vouch for its currently approved exact
account, device and full identity for bounded ciphertext retrieval. A compromised
home can impersonate those claims and obtain ciphertext, but receives no private
mail or attachment keys or plaintext. It cannot enroll encryption devices, replace
verified identities or pins, or select encryption recipients; see the [scope
clarification](https://github.com/gonz0w/faction-bbs/issues/294#issuecomment-6513142126).
Reachability, home-service authentication, device authority, resource authorization
and encryption identity are distinct. BBS accounts, roles, circles, files and
approval stores remain local.

## Standard and application mapping

RFC 7662 specifies an authenticated state query and response metadata; its
[security considerations](https://www.rfc-editor.org/rfc/rfc7662.html#section-4)
address revocation and stale caches. This profile forbids positive caching. RFC
8705 distinguishes [client authentication](https://www.rfc-editor.org/rfc/rfc8705.html#section-2)
from [certificate-bound token verification](https://www.rfc-editor.org/rfc/rfc8705.html#section-3.2):
`cnf.x5t#S256` is base64url SHA-256 of the presenting client's DER certificate,
not the existing hex spelling. The attachment claims and checks below are Faction
requirements, not behavior supplied by these standards alone.

The recipient home has authorization-server and separately authenticated relay
roles; the sender has resource-server and introspection-client roles. Roles may
share a process, but credentials and authority remain distinct. Protected invitation
configuration binds the exact home and issuer to token and introspection endpoints,
CA, name and pin, client registrations, allowed audience, enabled state and
generation/reset floor. A locator or claim cannot choose an endpoint; no wildcard
issuer or central account is assumed. Use `tls_client_auth` with RFC 8705 chain
verification and one registered DN or SAN plus Faction's exact pin and current
invitation checks. Today's digest registry alone is not that OAuth method. RFC 7662
section 4 requires TLS 1.2 support, while the provider uses TLS 1.3; the backchannel
needs explicit qualification, not a silent provider change or unrestricted
conformance claim.

Two purpose-specific paths preserve both-principal checks:

1. **Retrieval grant:** an actual recipient-device `Session` creates a private,
   immutable short-lived operation record. Account, device, full identity,
   certificate digest, generation and reset come from the session, not request
   actor fields. The request selects resource and action. The relay calls the token
   endpoint with its own mTLS certificate; the issuer checks the record and current
   authority and binds the token to that observed relay certificate. It must not
   mint a relay-bound token from a member-certificate request. The standing
   confidential home-service permission plus fresh record is Faction's arrangement
   under RFC 6749 section 4.4. No refresh token, password or browser grant is
   required, and generic OAuth-client interoperability is not claimed.
2. **Recipient-current observation:** the invited sender-home confidential service
   asks whether the exact intended recipient remains approved. The authenticated
   subject is the service client; the device tuple is registry-checked query data,
   not a fabricated recipient session. This distinct purpose names the recipient,
   sender service, complete blob binding and policy-check action, but grants no
   actor, read, upload or mutation authority. Each foreign upload operation, and
   status or deletion where policy requires, retains the actual sender-device
   session as actor and requires a fresh observation. Recipient fetch/status checks
   current local sender authority and fresh recipient retrieval authority. Neither
   path copies remote enrollment into the sender registry.

A retrieval grant binds the exact invited issuer and home; internally derived
account, nonzero device and full 32-byte identity; registered relay and observed
relay certificate; one exact sender-home attachment-service audience; ciphertext
SHA-256 identifier and complete immutable binding (sender, recipient, size,
creation and expiry); and either one `GetChunk` offset or recipient `Status`, for
a committed object. It expires within ten seconds and no later than the originating
cycle, certificate, record or object expiry. It includes issuer generation/reset,
current device/peer policy and exact retry tuple. A changed resource, recipient,
offset, action or audience needs new authorization. Observation and retrieval
results remain structurally distinct even when identity fields match.

For every attempted delegated operation the sender authenticates the relay and
introspects the configured issuer. The issuer authenticates and authorizes the
sender service, checks current issuer/account/device/certificate/peer/reset state,
and returns minimal exact-operation metadata. Inactive or unauthorized queries
reveal no reason or member metadata. The sender checks purpose, tuple, audience,
certificate binding and expiry, then rechecks its invitation, sender binding,
generation/reset, cancellation and deadline after the network call and immediately
before store access. Only a private verified result supplies authority, never
unverified JSON or a public principal. Online state is current at the issuer's
check, not a distributed atomic guarantee: revocation cannot recall an operation
already authorized, but the next operation requires a fresh check.

An exact retry of the same resource, action and offset requires fresh authorization;
one-shot consumption must not prevent recovery after a lost response. A new token
may authorize the same persisted operation without extending retention. Stop, reset
or restart discards transient records and tokens; a new real session is required.
No cached ciphertext result bypasses current checks. Lost replies retain the
existing reopen and exact-retry treatment, including `PublicationUncertain`.
Outage, malformed response, unknown purpose, clock rollback, exhausted admission
or unavailable policy denies the operation without alternate routing. Local
verification and save retain their existing rules.

## Maintained interfaces and missing work

The pinned [TLS server](https://github.com/gonz0w/faction-bbs/blob/e2aeeb12dff7244edeb84dc208dd6927230114fc/crates/faction-provider/src/tls.rs#L277)
owns one active connection and derives a handle from verified certificate DER;
its [session](https://github.com/gonz0w/faction-bbs/blob/e2aeeb12dff7244edeb84dc208dd6927230114fc/crates/faction-provider/src/tls.rs#L385)
does not expose a public actor. A private session-to-record interface is needed.
The pinned [registry](https://github.com/gonz0w/faction-bbs/blob/e2aeeb12dff7244edeb84dc208dd6927230114fc/crates/faction-provider/src/authority.rs#L98)
checks current actor, sender and recipient; `LocalServiceAccess` rejects foreign
homes. A fetch grant alone cannot make cross-home upload valid. Its
[handle checks](https://github.com/gonz0w/faction-bbs/blob/e2aeeb12dff7244edeb84dc208dd6927230114fc/crates/faction-provider/src/authority.rs#L430)
need a separate private delegated verifier result. Publication and durable policy
remain governed by [runtime](https://github.com/gonz0w/faction-bbs/blob/e2aeeb12dff7244edeb84dc208dd6927230114fc/crates/faction-provider/src/lib.rs#L41)
and [durable](https://github.com/gonz0w/faction-bbs/blob/e2aeeb12dff7244edeb84dc208dd6927230114fc/crates/faction-provider/src/durable.rs#L22)
authority, including floors; invitation mapping cannot bypass them.

[Store principals](https://github.com/gonz0w/faction-bbs/blob/e2aeeb12dff7244edeb84dc208dd6927230114fc/crates/faction-files/src/private_blobs.rs#L54)
are validated data, while access supplies trust. The [store](https://github.com/gonz0w/faction-bbs/blob/e2aeeb12dff7244edeb84dc208dd6927230114fc/crates/faction-files/src/private_blobs.rs#L230)
requires the exact recipient and committed state for recipient reads and status;
sender status also supports uncommitted upload progress. Denied authorization
precedes store clock mutation, though authorized expiry may update it. The
[client](https://github.com/gonz0w/faction-bbs/blob/e2aeeb12dff7244edeb84dc208dd6927230114fc/crates/faction-client/src/mail_device_provider.rs#L33)
uses a trusted loopback profile and vault match, with no delegated foreign route.
The maintained binary service remains same-home only.

Missing components include real token and introspection endpoints, registrations,
bounded HTTP/form/JSON handling, secure opaque token generation and storage, both
purpose-specific policy types, private session-record and verifier-store interfaces,
durable invitation/floor data, callback executor and native/operator integration.
Existing constructors provide none of these merely by naming a standard.

## Bounds, qualification and finite oracles

Proposed ceilings, not measured capacity: one outstanding delegated operation and
at most eight live operation-record/token pairs in aggregate across clients and
cycles. Each remains bounded by its original cycle deadline and expiry; free expired
records and reject busy when full, without an unbounded queue or tombstones. A
token is at most 256 bytes, and
each HTTP header section, form body and JSON body at most 8192 bytes, checked
before allocation. Reject duplicate fields and ambiguous framing; no redirects,
compression, dynamic discovery, automatic retries or unbounded token/replay log.
Existing provider limits remain one active connection/handshake per runtime, eight
attempted operations, five-second TCP/TLS admission and ten-second cycle; network
phases spend the original deadline. Existing 32 bindings/homes/history, eight
handles, store limits and five-second drain remain. Filesystem deadlines remain
cooperative.

The current [operator loop](https://github.com/gonz0w/faction-bbs/blob/e2aeeb12dff7244edeb84dc208dd6927230114fc/crates/faction-provider/src/operator.rs#L75)
is sequential and cannot answer a callback while blocked relaying. A new bounded
executor must serialize issuance and release its TLS session before relay, keep the
issuer backchannel schedulable, and never hold registry or store locks across
network waits. The minting device TLS session closes to release the one-active
admission; the proposed operation record does not preserve, reuse or clone its
connection or handle, which become invalid on close. A new purpose-specific private
factory and verifier must bound the record by the original cycle and current
registry generation, reset and certificate state; #300 must test this interface.
It may not add a runtime to evade the one-active limit. All logical
roles fit the existing aggregate three owner-helper budget including existing
helpers; lack of capacity refuses work. Ordinary eight sessions/5000 ms admission,
two credential permits and two kernel slots remain unchanged. A disabled or
failing optional profile leaves ordinary mail, BBS and files independent.

The inspected `crates/faction-provider/Cargo.toml` and
`crates/faction-provider/Cargo.lock` have SHA-256 respectively
`4c609f676808d114e2a57590d4759d18094f8b60e11041b1dacf8b0c92215bcc` and
`3678cb86d394c72e24fab5f9ef61544a9a15b295fefc2568678cfc6699693164`.
They pin OpenSSL 0.10.81 and openssl-sys 0.9.117; a locked serde_json 1.0.154
does not qualify a new direct parser or its features. No OAuth server is selected.
Before executable work, #300 must identify exact implementation inputs and features,
inspect the complete selected target/build/tool graph, hashes, licenses and no-JS/TS
boundary, and preserve product and third-party terms. A standard is not an adopted
server or relicensed code. No third-party service, dependency, public HTTP endpoint
or host is adopted here.

Future tests must show only a current approved device session can create a retrieval record; a copied
card, public actor, wrong certificate or service observation cannot. Independently
configured homes must exercise approved upload and committed fetch/status with
remote-recipient checks. Wrong issuer, client, certificate, identity, audience,
binding, purpose, action or offset must fail before unauthorized mutation. Revoke,
rotate or reset policy between operations and during callbacks; the next operation
must deny and a stale post-network local snapshot must fail. Drop replies before
and after dispatch and prove fresh-authority exact retry without renewed retention
or mail release without authenticated confirmation. Saturate budgets, stall
callbacks, TLS and bodies, cancel and stop; prove no deadlock, lock held over
network, extra runtime or deadline extension. Test expired, uncommitted, malformed,
oversized and unavailable cases, private device-only keys and explicit save, and
ordinary service independence. #301 requires actual host and NetBSD two-home proof
and owned cleanup.

A missing private authority interface, inability to preserve both-principal current
checks, failed input qualification or exceeding existing budgets is a falsifiable
NO-GO. This source research passes none of the runtime oracles. Protocol and source
inspection created no downloaded-source or runtime fixtures; committed-source
archive validation is separate. No compilation, deployment, interoperability or
TDD result is claimed.

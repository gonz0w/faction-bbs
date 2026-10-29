# Optional attachment provider embedding

This explicitly compiled Rust library exposes the qualified synthetic device/TLS
boundary for an embedding consumer. It is outside ordinary workspace builds and
creates no listener, service, thread or enrollment automatically. The experiment
under `experiments/270-provider-auth` remains its historical reference.

Create a `Runtime` from an explicitly validated `authority::Registry`, retain that
non-cloneable owner, and obtain its weak `Control`. Configure `tls::Server` from
preloaded server certificate/key and client CA. On a caller-owned loopback listener,
`Server::accept` yields an opaque `Session` only after actual TLS 1.3 peer
verification and exact DER digest lookup. No public constructor accepts a claimed
actor, digest or verified flag; neither handles nor the TLS stream escape.

The consumer calls `Session::dispatch(&mut Store, route_account, Request)` or
`dispatch_bytes(&mut Store, route_account, bytes)`. The account is a routing
assertion, compared internally to the authenticated binding; it cannot select the
actor, including for Status/GetChunk. Each call charges an attempt before decoding,
checks internal wall/monotonic clocks, cancellation and actual certificate validity,
and rechecks current registry/device/full identity/invited-peer policy before the
unchanged ciphertext store action. Binary input remains at most 8192 bytes. `Session::serve_next` reads one existing
four-byte-length-prefixed binary request from that same verified TLS stream,
dispatches it, and writes the existing encoded response; EOF returns false.
Every call charges once before reading, and failure closes the stream. A successful
write does not establish that the peer received or persisted the result.

`Control::replace_registry`, `unavailable`, `reset` and `stop` remain usable while
a session exists. A rejected typed replacement preserves the previous snapshot.
A valid generation replacement invalidates old handles; reconnect explicitly.
Unavailable policy denies; restoring policy requires another valid generation.
Reset grants no new binding or key recovery. Runtime stop/drop is terminal: it
invalidates authority and shuts down its active socket. Controls and sessions hold
weak references and cannot keep a dropped runtime's authority alive. Session drop
closes its socket and releases admission. A new Runtime cannot revive old sessions.
Remote TCP disappearance is observed by transport I/O, not independently attested
by an in-process dispatch call.

Only one provider connection/handshake is active per runtime. TCP wait and TLS share
five seconds of admission; an accepted cycle lasts ten seconds from TCP acceptance.
Each session allows eight attempted operations, including denied/malformed calls.
The inherited authority also bounds eight handles and 32 bindings, invited homes
and lifetime digest tombstones. TLS resumption, tickets and early data are disabled.
The `tls::Client` is an opaque verified loopback counterpart with server CA/name/pin
checks; `Client::request` supplies the matching bounded binary request/response carrier.
Neither API implements an XMPP stream or IQ carrier.

The store retains eight objects, 1,048,592 ciphertext bytes per object, 8,388,736
reserved bytes, 4096-byte chunks and seven-day retention. A denied operation cannot
advance unauthorized store state. Authorized expiry/status observations can advance
the durable high-water clock. Lost responses and PublicationUncertain do not imply
rollback: reopen and use the exact retry to resolve their outcome. Filesystem calls
remain cooperative and cannot be forcibly preempted by these deadlines or stop.
The embedding owner must perform bounded drain/cleanup; tests allow five seconds.

Explicit local compilation uses the existing locked OpenSSL 0.10.81 sources and
an existing OpenSSL 3 development installation, without vendoring. Keep Cargo's
target outside this source directory. Run `cargo +1.96.0 test --manifest-path
crates/faction-provider/Cargo.toml --locked --offline -- --test-threads=1` only after
selected-source/license/no-JS preflight. Compiler and runtime costs are separate.

These APIs establish TLS key possession and approved routing, not OMEMO private-key
possession, hardware attestation, production enrollment/recovery or durable
anti-rollback. In-memory history is lost with the runtime; the explicitly selected `durable-policy` feature retains it across ordinary restart as described in the [durable policy contract](../../dev-docs/provider-policy.md). Device mail/attachment
keys and decryption stay on member devices. Ordinary classic/SSH and native mail
remain independent; no account-login-to-device upgrade is added. The optional [private operator](../../docs/provider-operator.md) runs the same-home loopback service with preapproved policy and independently supplied TLS material. Production activation, XMPP server/SASL, own-home delegation and independent client interoperability remain outside its claims.

## Service-bound binary integration

`BinaryService::new(home)` fixes the trusted local service. Use
`Session::dispatch_service` or `Session::serve_service_next` to derive the actor
from the current verified session without a caller-selected routing account. Both
sender and recipient bindings must be local to that service, including reads of
existing objects. The older routed APIs keep their separate invited-home policy.
The explicitly selected native `device-provider` feature uses this loopback binary
carrier for same-home attachments and the ordinary encrypted-mail path separately
for release. No store reply means no release authorization; retry the exact durable
operation after an uncertain outcome. Default, `native-mail` and `network-mail`
client builds exclude this provider. See the [integration contract](../../dev-docs/provider-integration.md).


## Optional typed IQ integration

Select `--features blob-iq` explicitly. `IqService::new("home.test")` validates a
trusted local service before accepting requests. `Session::dispatch_iq(&mut Store,
&IqService, &Element)` reuses `faction_xmpp::blob_iq`; the native client retains its
compatible `mail_blob_wire` export. The namespace, binary payload, canonical base64,
IDs/addresses and fixed failure encoding are unchanged. The optional edge is to
the protocol crate, never to the native client, OMEMO or UI.

Every IQ call charges the same session attempt as binary operations before bounded
DOM validation, decoding or route/policy denial. The qualified output validator
bounds DOM depth/events/raw/escaped bytes before text concatenation. The handler
requires the configured service and its local account domain; it compares `from`
to the exact current certificate binding inside authenticated dispatch. XML
resources, copied public identities and forwarded foreign claims cannot select an
actor. Clocks, cancellation and session lifetime are checked before decoding and
around the store dispatch; current policy and actual certificate validity gate
each store action. Bounded local IQ response construction sends nothing over the
network and cannot roll back completed authorized store work. Stop or expiry does
not retroactively suppress an already authorized typed result.
Safe valid requests receive the existing uniform result/failure; malformed or
unsafe envelopes receive a generic local refusal. No internal diagnostic or
existence class is serialized.

This API accepts typed in-process IQs after real device mTLS. It adds no IQ frame
carrier, XMPP stream/server, SASL mechanism or connectivity for the ordinary
PLAIN-authenticated native session. Interhome delegated identities and direct
foreign-home requests remain unsupported. The original member-side own-home route,
private checkpoints and classic defaults remain independent.

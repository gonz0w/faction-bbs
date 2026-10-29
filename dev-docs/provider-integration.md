# Optional provider embedding

`crates/faction-provider` is an explicitly compiled, standalone Rust workspace;
it is not a root workspace member or a dependency of ordinary board or native
client builds. It exposes the [experimental authority boundary](provider-authority.md)
for an embedding application. It starts no listener, daemon, enrollment or file
loader. A caller supplies a validated registry, preloaded TLS credentials and a
listener; production enrollment and durable anti-rollback remain separate work.
The crate uses OpenSSL 0.10.81 with an existing OpenSSL 3 installation.

## Verified session and binary carrier

Create a `Runtime` with an `authority::Registry`, retain the runtime and configure
`tls::Server` with the server certificate, private key and client CA. `Server::accept`
accepts on the caller's loopback listener and yields a `Session` after TLS 1.3
client-certificate verification and exact certificate-DER digest lookup. The
binding supplies the actor; request text cannot choose one. The caller passes a
routing account to `Session::dispatch` or `dispatch_bytes`; authorization compares
it to that binding, including for status and reads. The latter accepts the existing
binary codec, bounded to 8192 bytes.

For the same verified connection, `Session::serve_next` reads one four-byte-length
prefixed binary request, dispatches it and writes the encoded reply; `tls::Client::request`
is the corresponding verified loopback client. EOF ends the server loop. This
carrier is distinct from the native client's XMPP connection and does not carry IQs.
A successful write does not prove receipt. A lost response or `PublicationUncertain`
requires reopening the store and resolving the outcome with the exact retry.

The non-cloneable runtime owns authority. Its weak `Control` can replace the
registry, mark it unavailable, reset or stop it between operations. Invalid typed
replacements preserve the previous snapshot; a valid generation change invalidates
old handles. Stop or runtime drop invalidates authority and shuts down the active
socket; weak controls and sessions cannot keep it alive. The caller owns cleanup.
Remote TCP loss is observed through transport I/O, not independently by an
in-process dispatch.

One connection or handshake is active per runtime. TCP wait and TLS share a
five-second admission budget; the cycle has ten seconds from TCP acceptance and
at most eight attempted operations. Internal clocks and current certificate,
registry and invited-peer policy are checked on each operation. Authority is
bounded to eight handles and 32 bindings, invited homes and lifetime certificate
mapping tombstones. TLS tickets, resumption and early data are disabled. Filesystem
operations are cooperative and cannot be forcibly preempted; the embedding owner
must drain and clean up within its own bounds. The existing store's object, byte,
chunk and retention limits remain in the [storage contract](private-attachment-storage.md).

## In-process IQ adapter

The optional `blob-iq` feature adds `IqService` and `Session::dispatch_iq` using
`faction_xmpp::blob_iq`, the codec also re-exported by the native client as
`mail_blob_wire`. Construct `IqService::new(home)` from a trusted configured local
service, then pass the store, service and parsed `Element` to `dispatch_iq` on an
already verified device TLS session. The IQ `from` is only a routing assertion
checked against the authenticated binding; `to` must match the configured service.
The adapter charges the same session attempt and checks the deadline before
validating or decoding the DOM, including for malformed and denied calls. It checks
the deadline and current authority around actual store dispatch. Bounded local IQ
response construction performs no store or authorization operation and sends no
network data. Stop or revocation applies to the next operation; it cannot undo
completed store work or guarantee suppression of an already authorized typed result
while that response is being constructed. The binary carrier retains its I/O
deadline and cancellation checks. The adapter retains the codec's canonical base64,
identifier and address checks, 8192-byte blob and 16384-byte XML limits and uniform
safe remote errors for valid requests. Malformed input can instead fail locally.

This is an in-process embedding interface, not an XMPP stream, server or SASL
implementation. An ordinary native client PLAIN session does not establish the
verified device session needed here. It adds no framed IQ protocol, native route,
checkpoint or menu. Own-home delegation, production provider activation and
independent modern-client interoperability remain open in [#88](https://github.com/gonz0w/faction-bbs/issues/88).

The provider sees public routing and ciphertext metadata, not device mail or
attachment private keys or plaintext. TLS key possession and registry approval do
not establish possession of an OMEMO private identity key or hardware attestation.
Classic board access and ordinary mail remain independently available. See the
[provider crate README](../crates/faction-provider/README.md) for its current API
and explicit build instructions. The [host and fresh NetBSD software record](evidence/provider-integration/README.md)
qualifies these exact runtime/IQ inputs, including committed-object route controls,
current policy and lifecycle, default/client/store regressions and owned teardown.
The native runtime used two CPUs/about 1 GiB; compilation used two CPUs/2 GiB and
at most two compiler jobs. These are separate guest envelopes, not peak-memory,
hardware or supported-capacity measurements. Earlier experiment observations remain
unchanged and are not reused as new runtime passes.

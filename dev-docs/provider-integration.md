# Optional provider embedding

The optional [durable policy](provider-policy.md) and [private operator](../docs/provider-operator.md) add restart-safe policy and a foreground same-home loopback entry point. Their qualification remains software preview.

`crates/faction-provider` is an explicitly compiled, standalone Rust workspace;
it is not a root workspace member or a dependency of ordinary board or native
client builds; only the explicit `device-provider` client feature adds the local
optional dependency. It exposes the [experimental authority boundary](provider-authority.md)
for an embedding application. It starts no listener, daemon, enrollment or file
loader. A caller supplies a validated registry, preloaded TLS credentials and a
listener; production enrollment and durable anti-rollback remain separate work.
The crate uses OpenSSL 0.10.81 with an existing OpenSSL 3 installation.

The optional [offline preparation guide](../docs/provider-preparation.md) describes
client profile and card creation and the separate checked operator proposal. The
card is local self-asserted data, not an authority or wire protocol; existing TLS
parsing and remote authorization remain separate.

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

## Service-bound native binary route

`BinaryService::new(home)` binds the trusted configured local home. An opaque
verified `Session` can use `dispatch_service` or `serve_service_next` without a
caller-selected account: current certificate authority supplies the actor. Both
sender and recipient bindings must belong to that home, including reads of
existing objects. The existing routed APIs retain their separate current-registry
and invited-home rules. The native `device-provider` feature selects this binary
carrier explicitly; it is not an IQ wire protocol.

The native client checks the configured expected account, device and full identity
against its unlocked vault before connection. Its private profile and transport
credentials are local inputs, not endpoint authority from a locator or remote board.
Cancellation closes and joins the bounded worker; it receives no vault or OMEMO
keys. A complete authenticated store reply permits the existing separate encrypted
XMPP mail release. Failure or a lost reply permits no fallback or release; exact
retry resolves uncertain durable progress. Ordinary mail and board access remain
independent. The [native-client integration evidence](evidence/native-provider-client/README.md)
records host and fresh NetBSD maximum-size production-console transfers and
cancellation/recovery controls, separately from the historical standalone IQ
evidence below. Length and payload now share one bounded TLS frame write; wire
bytes and existing limits are unchanged.

## Invited-home attachment trust

The settled [owner decision](https://github.com/gonz0w/faction-bbs/issues/294#issuecomment-6513127326)
allows an independently operated, explicitly invited recipient home to vouch for
its currently approved exact account, device and full identity for bounded
ciphertext retrieval. The sender home must authenticate the exact issuer and
validate the exact sender-home/service audience, committed ciphertext and
fetch/status action, bounded expiry and current issuer, device, peer and reset
policy on each operation. An exact retry requires fresh authorization. A home
certificate or unverified forwarded principal alone cannot establish the device.

A compromised trusted home can impersonate its device-approval claims and obtain
ciphertext, but receives no end-to-end encryption keys or plaintext. The vouch
cannot automatically enroll an encryption device, replace sender-verified peer
identity or pins, or choose encryption recipients; member-side verification and
encryption checks remain independent. BBS accounts, local approval and roles,
circles and private-mail keys do not federate. See the [scope clarification](https://github.com/gonz0w/faction-bbs/issues/294#issuecomment-6513142126).

The default-off `delegation-services` feature supplies a separate
[service observation](service-observations.md) of current recipient approval for
an authenticated invited service. It does not create a device actor or grant
store access.

The default-off `delegation-issuer` feature adds a library-only, certificate-bound
[retrieval issuer](delegation-issuer.md) for an explicitly permitted relay and
introspection service. Its optional `delegation-wire` carrier serves one bounded
HTTP operation per verified service connection. Its metadata is not a Store grant
or sender verification.

The default-off `delegation-records` feature supplies a local, session-derived
[retrieval intent](retrieval-intents.md) for trusted embedders. It is not a remote
read grant and does not establish sender policy or committed object state. The
maintained native binary service above remains same-home only. Sender-side
authority and independently hosted cross-home operation remain future work under
[issue #300](https://github.com/gonz0w/faction-bbs/issues/300) and
[issue #301](https://github.com/gonz0w/faction-bbs/issues/301).


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
verified device session needed here. It adds no framed IQ protocol or native route by itself. Own-home delegation, production provider activation and
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

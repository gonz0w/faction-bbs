# Provider and device boundary for private attachments

Research date: 2026-10-28. This is a source-backed recommendation for a bounded
software prototype, not a provider selection, production policy or runtime result.
The code baseline is [5731701](https://github.com/gonz0w/faction-bbs/tree/5731701916e18957be6cc6f6bdac5fe9d528f8eb).
The [native qualification](../evidence/netbsd-attachments/README.md) exercised
existing paths, not the future authentication boundary. Independent modern-client
interoperability and real provider integration remain part of [#88](https://github.com/gonz0w/faction-bbs/issues/88).

## Current boundary

The [wire codec](../../crates/faction-client/src/mail_blob_wire.rs) carries requests,
not an authenticated actor. [Store::dispatch](../../crates/faction-files/src/private_blobs.rs)
takes a separate trusted `Principal` and current `Access`; it validates the routing
shape of the account, device and 32-byte public identity and checks current policy
on each operation. It does not prove possession of the Ed25519 private key.
The [client transfer path](../../crates/faction-client/src/mail_blob_transfer.rs)
checks retained device identity and revocation, uses the pinned own-home session,
and selects an invited sender home for fetch; a locator cannot supply a route.
The [XMPP client](../../crates/faction-xmpp/src/lib.rs) configures server CA/name/leaf
verification and PLAIN authentication, but no TLS client identity. Its account
login does not prove a device.

The [Rust fixture](../../crates/faction-client/examples/mail_blob_fixture.rs)
selects its actor from byte 0 or 1. The [Python fixture](../../tools/probe_attachment_transfer.py)
supplies that byte after its PLAIN home-account check. These controlled assertions
exercise the store and transfer path, not an independently authenticated device or
a deployed provider. An XMPP resource, XML field or claimed fingerprint cannot
supply authority. Server-to-server TLS authenticates a home, not its recipient
device. How the recipient's own home obtains and delegates trustworthy device
identity to the sender's home remains a separate unsolved routing problem.

## Alternatives

**A: maintained provider session adapter.** [Prosody 13.0.7](https://prosody.im/doc/release/13.0.7)
(released 2026-09-23) has built-in token authentication, `internal_hashed` and
OAUTHBEARER. In the pinned [official source archive](https://prosody.im/downloads/source/prosody-13.0.7.tar.gz)
(745,079 bytes; SHA-512
`53056342911f26451ef21f39b8ba53a3f27752a65f190516d21892c4740f8ce163f8a9b943feab438fd32e10643466037a42519c7673c0e0dc85e4dc107a7321`),
`plugins/mod_tokenauth.lua` creates and validates grants and tokens, and exposes
revocation and token information. `plugins/mod_auth_internal_hashed.lua` installs
its OAuth-purpose handler; `util/sasl/oauthbearer.lua` implements the mechanism;
`plugins/mod_saslauth.lua` fires `authentication-success` before clearing the
SASL handler. A trusted session hook could consult a separate current registry,
but this is not a drop-in device proof. The token contains a grant ID, secret and
JID; the secret is a bearer credential, never a public principal. See
[RFC 7628](https://datatracker.ietf.org/doc/html/rfc7628).
Password changes invalidate grants created strictly earlier than the recorded
password update, but this comparison alone is not a complete same-tick reset
epoch; established sessions still require fresh policy checks. A password reset
must never create an approved device binding or recover device keys.

The archive's `COPYING` identifies MIT licensing. The
[upstream dependencies](https://prosody.im/doc/depends) include Lua, LuaSocket,
LuaSec/OpenSSL, LuaExpat/Expat and LuaFileSystem. The [NetBSD pkgsrc recipe](https://github.com/NetBSD/pkgsrc/blob/trunk/chat/prosody/Makefile)
adds Lua 5.4 and native dependencies including lua-unbound/libunbound, libidn and
ICU. The libidn recipe lists GPLv3 and LGPLv2.1 classifications; the selected tools and
library licensing have not been separately audited.
A complete selected transitive build/runtime, license and no-JS inventory and
NetBSD provider qualification remain adoption prerequisites, not results here.

**B: independent TLS device transport.** Standard mutual TLS verifies possession
of the TLS private key; a trusted adapter can obtain the verified peer certificate
and look up its exact DER SHA-256 in an operator-approved registry. The
[OpenSSL verification API](https://docs.openssl.org/3.5/man3/SSL_CTX_set_verify/),
[TLS 1.3](https://www.rfc-editor.org/rfc/rfc8446), and pinned
[rust-openssl 0.10.81 API](https://github.com/rust-openssl/rust-openssl/blob/openssl-v0.10.81/openssl/src/ssl/mod.rs)
support this boundary. The existing lock includes `openssl` 0.10.81 (Apache-2.0),
`openssl-sys` 0.9.117 and `native-tls` 0.2.18 (MIT/Apache-2.0). Native evidence
used base OpenSSL 3.5.7; upstream [3.5.9](https://openssl-library.org/source/)
was released 2026-09-29 and the [3.5 LTS line](https://openssl-library.org/policies/releasestrat/)
ends 2030-04-08. This neither upgrades OpenSSL nor proves server mTLS on NetBSD.
The current client does not configure its available client-identity API.
[XEP-0178](https://xmpp.org/extensions/xep-0178.html) describes SASL EXTERNAL,
but does not prove an OMEMO key. The alpha [mod_auth_ccert](https://modules.prosody.im/mod_auth_ccert)
and deferred [XEP-0257](https://xmpp.org/extensions/xep-0257.html) are not selected.

Recommend B for a loopback software prototype only: it tests a concrete credential
boundary without depending on a provider hook. TLS key possession is distinct from
possession of the OMEMO Ed25519 private key and from hardware attestation. An
operator-approved mapping authorizes routing; it proves neither latter property.

## Proposed prototype contract

The registry contract is scoped in [#270](https://github.com/gonz0w/faction-bbs/issues/270)
and its real TLS proof in [#271](https://github.com/gonz0w/faction-bbs/issues/271).

Use synthetic, offline operator-approved enrollment only. The operator associates an
exact certificate DER SHA-256 with one account, device number and full Ed25519 public
identity, and controls account, device and invited-home policy. A device owner must
separately verify its OMEMO identity; a password reset grants no binding. Rotation
requires a new explicit approval and revocation of the old certificate. Lost devices
and account/device/peer revocation deny the next operation on existing sessions.
Expiry denies use; exact request retries retain the existing store's deduplication
and expiry semantics. No copied certificate without its private key authenticates.
Downloaded ciphertext cannot be recalled.

| Boundary | Proposed responsibility |
| --- | --- |
| Registry/current authority | At most 32 bindings and 32 invited homes; store exact certificate digest, account, device, full public identity, `valid_from`, `valid_until`, approval and current generation/reset epoch. Resolve only verified peer certificates; check current account, device, peer and operation policy for every dispatch; missing or corrupt policy denies. |
| TLS adapter | Require independently verified client certificate and server CA/name/pin checks; retrieving a certificate alone is not verification. Disable TLS resumption and 0-RTT. Extract the peer certificate from the verified TLS connection, resolve it in the registry, and issue an opaque connection-bound handle with its own lifetime. Never derive authority from wire XML, resource or claimed fingerprint. The embedding TLS adapter remains trusted; Rust API privacy is not a process sandbox. |
| Store | Resolve the handle to the trusted principal and current `Access`, then invoke the existing unchanged `Store::dispatch`; wire data is never the actor. Do not accept unverified forwarded principals. Use a bounded binary codec, at most 8192 bytes, for this local proof; it is not an XMPP protocol implementation. |

Allow at most eight live handles, eight operations per cycle and ten seconds per
cycle, a 5000 ms handshake deadline, five-second drain and at most three helpers.
Each operation rechecks the current generation and reset epoch; a change invalidates
old handles. Restart creates a new
instance and invalidates all previous handles. An in-memory generation is not
durable anti-rollback protection. Production enrollment, durable registry recovery,
anti-rollback and full provider/session integration need separate designs.

A successful loopback proof must demonstrate an approved certificate reaching the
existing ciphertext store with its exact mapped principal, current peer policy and
normal retry behavior. It must reject missing, wrong, expired or unapproved client
certificates including not-yet-valid certificates, bad CA, server name or pin,
copied certificate without private key, cross-connection handles, wrong
account/device/full identity, stale generation or handle after restart, and live
account/device/peer revocation before the next operation. Exercise replay, quota,
malformed input, timeout, cancellation and failure; denied or failed operations must
not mutate store state where applicable. Preserve the existing store bounds: eight
objects, 1,048,592 bytes each, 8,388,736 bytes total, 4096-byte chunks and seven
days retention. Verify bounded cleanup of all owned
fixtures and unchanged default-off, classic access and device-only decryption.
Provider outage fails closed without a plaintext fallback. These are specifications,
not executed results. The prototype cannot establish production enrollment,
hardware attestation, own-home delegation or independent-client interoperability.

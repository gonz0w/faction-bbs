# Native XMPP research transport contract

Issue [#208](https://github.com/gonz0w/faction-bbs/issues/208) qualifies one narrow
transport beneath a future mail adapter. The standalone
[experiment](../experiments/208-xmpp-transport/Cargo.toml) is not part of the product
workspace or a live mailbox. [#209](https://github.com/gonz0w/faction-bbs/issues/209)
owns deliberate product adoption and the first usable online exchange.
Existing [checkpoint](native-mail-checkpoints.md), local console, SSH and files
remain separate. The probe never opens a vault or accesses member keys.

## Qualified profile

Reuse tokio-xmpp 6.0.0 with defaults disabled and native-tls 0.2.18. Use its typed
XML stream and SASL, with explicit fixed resource-binding IQ glue through the
library serializer; an existing incremental rxml 0.14 parser
guards bytes before the downstream XML/DOM reader. Do not introduce a second
handwritten XMPP parser, new cryptography or an unreviewed OMEMO helper.

The research binary connects only to loopback. Its fixed synthetic identities are
`alice@home.test/faction208` and `bob@away.test/device1`. The sequence is:

1. Load the trusted generated regular CA PEM, open one TCP connection and require
   direct TLS with a minimum TLS 1.2 setting. NetBSD observes1.2/1.3; the macOS native backend observes
   1.2 only. No plaintext or STARTTLS fallback.
2. Verify the chain with explicit anchors, the `home.test` hostname/SNI, and the
   full SHA256 DER leaf-certificate pin. Built-in roots are disabled. Complete all
   three checks before XML or authentication credentials are sent.
3. Intersect offered mechanisms with PLAIN only, then use the library's SASL
   authentication and one trusted XML stream restart. PLAIN supplies no protection
   itself; verified TLS supplies channel confidentiality. Other SASL mechanisms
   are outside this qualification. The home server necessarily receives its own
   account credential; it receives no mail-device key or local vault passphrase.
4. Bind the exact local resource and exchange one matching addressed chat stanza
   containing synthetic base64 ciphertext bytes under `urn:xmpp:omemo:2`.
   The subtree is a transport fixture, not a complete OMEMO header/prekey/SCE codec
   or proof of a new E2EE implementation.
5. Close on success, error, cancellation or deadline. There is one attempt and no
   retry task, background Client, DNS discovery, component, browser or HTTP adapter.

The [protocol sources](research/0055-native-xmpp-transport.md) distinguish this
profile from complete RFC/XEP conformance. A TLS certificate pin is server trust;
it is not the peer's Ed25519 mail-device pin, home-account proof or BBS approval.

## Admission and lifecycle limits

| Item | Fixed research bound |
| --- | --- |
| Decrypted XML read from TLS | 65536 unique physical bytes across both stream epochs |
| Parser interval/stanza budget | 16384 bytes, conservatively including preceding root whitespace |
| Nested XML elements | 32 including the stream root |
| Guard parser events | 256 observed; the terminal rejected event is not forwarded |
| Encoded test payload | Canonical base64 of 1–1024 bytes |
| Trusted stdin config | 8192 bytes, exactly five newline-terminated fields |
| CA PEM read | 65536 bytes from a trusted generated regular file |
| Network operation | One attempt; cooperative ten-second lifetime and explicit cancellation |

The guard admits at most one byte per read. Before reading byte 65537 it refuses
further input. It checks nesting/event/interval limits before releasing a byte
that would complete an excessive structure to the DOM reader. Rejection latches;
unknown/malformed XML cannot turn into local authority.

The existing rxml reader can peek the next byte before draining an earlier end
event. On the authenticated SASL restart, a generation marker replays exactly one
already-read, unconsumed byte into the new guard parser without counting it twice.
Lifetime byte/event counters survive restart; a second restart is rejected.
Root Text events do not clear the interval counter, so whitespace cannot erase
the next stanza's opening delimiter. Exact-limit controls cover this boundary.

These are decrypted-XML limits. Native TLS/socket buffering and cryptographic
provider work are outside those counters. Deadlines are cooperative current-thread
Tokio cancellation, not OS preemption or a real-time guarantee. Trusted stdin
setup precedes the timer. The demonstrated CA inputs are generated regular files;
arbitrary local FIFOs/devices/blocking paths or hostile same-UID control are not
qualified. Product adoption must validate/preload local inputs before making a
broader deadline claim. No pending application queue exists in this one-stanza probe.

Protocol logging is disabled at compile time using `log` max-level-off features,
including when a caller installs a Trace logger. Both `SSLKEYLOGFILE` and
`TLSKEYLOGFILE` are refused before stdin/network work. Output is only a status enum,
one-attempt count and bounded counters; no error formatting, XML, credentials,
certificate paths or endpoint details are printed.

## Boundary to durable encrypted mail

The research probe has no checkpoint API or key access. The product adapter in
#209 must enforce these separate contracts:

- Unlock and verify device trust only through the member's trusted local path.
  A profile, remote ANSI string, XMPP stanza or server device list cannot enroll
  a peer, unlock a vault, import a BBS role or clear recovery quarantine.
- Publish public bundle material only when its private protocol state is durable.
  Validate the full existing typed device identity before starting a session.
- Release only ciphertext returned after a successful checkpoint. Reconnect/retry
  uses the exact persisted operation ID and bytes, never another seal operation.
- Accept an inbound Wire only through native authentication/SCE validation and the
  atomic history/ratchet commit. Parser success is not message authentication.
- A socket write, stream counter or stanza acknowledgement is not proof of peer
  acceptance, reading or exactly-once delivery. Keep ambiguous outbox items and
  require an honest explicit local retirement decision.
- Retain current one-peer, history/outbox/session limits and poisoned-handle rules.
  Full/expired/revoked/recovered stores require explicit handling. Offline archive
  synchronization needs its own durable cursor/backpressure contract under #90.

## Build and updates

```sh
cargo +1.96.0 test --manifest-path experiments/208-xmpp-transport/Cargo.toml \
  --all-targets --release --locked
python3 tools/check_xmpp_transport_dependencies.py
```

On NetBSD set `OPENSSL_INCLUDE_DIR=/usr/include` and `OPENSSL_LIB_DIR=/usr/lib`.
The [selected inventory](dependencies/xmpp-transport-research.json) pins versions,
checksums, licenses and features for Linux, ARM64 NetBSD and ARM64 macOS. The checker
also compares extracted source bytes to locked crate archives and rejects JS/TS
source/manifests. Upstream CI is not adopted: the local shell/Rust/C workflow is
explicit. Cargo lock/index metadata for unselected target packages does not select,
fetch or execute their JavaScript/WebAssembly source. The offline NetBSD fixture
used the verified source cache and sparse index; an incomplete directory-vendor
subset was rejected rather than replaced with unselected JS sources.

Keep third-party terms, including MPL-2.0 for the unmodified XMPP crates. Project
BSD-3-Clause does not relicense dependencies. A pin/feature/profile change needs a
finite issue, full graph/term/source review and preserved actual old evidence,
then host/NetBSD identity/auth/bounds/cancel/restart tests before adoption.
The complete fresh release build failed in a 1 GiB VM and succeeded on a 2 GiB builder;
final incremental rebuild and runtime tests passed at 1 GiB. See the
[actual observations](evidence/xmpp-transport/README.md). This is not an integrated
client/server memory or capacity promise.

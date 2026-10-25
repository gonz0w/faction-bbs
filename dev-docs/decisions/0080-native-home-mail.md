# ADR 0080: Default-off native home-mail adapter

Status: accepted for optional product implementation, 2026-10-25.
Authority: [#209](https://github.com/gonz0w/faction-bbs/issues/209), following the
[transport qualification decision](0079-native-xmpp-research-transport.md) and
[encrypted checkpoint decision](0078-encrypted-native-checkpoints.md).
This decision does not declare the issue's acceptance gates complete or authorize
a deployment, release, public listener or new external service.

## Decision

Adopt the qualified Rust direct-TLS XMPP profile beneath the member-device client
through the default-off `network-mail` feature. It includes `native-mail` and a
small optional transport crate. Use tokio-xmpp 6.0.0, native-tls 0.2.18 and the
existing rxml 0.14 admission guard with explicit route, CA, domain and leaf pin.
Keep the lower-level stream/authentication API, PLAIN only after verified TLS,
one connection attempt, bounded XML, cooperative cancellation and a finite manual
cycle. The [technical contract](../native-home-mail.md) fixes the details.

Expose a separate trusted-local `--mail-online` console. Publish public keys,
verify one peer, deliberately prepare the initial sender, compose/queue, synchronize
and retire pending ciphertext through explicit actions. Neither a remote BBS nor
home authentication can unlock local mail, verify a device or grant membership.
Ordinary board, SSH/files and local-vault operation need no home connection.

Reuse the reviewed native OMEMO/SCE implementation and atomic checkpoint/outbox.
There is no new cipher, ratchet or vendor-source modification. The narrow native
addition exports existing public prekeys read-only; the network receive entry
derives time from authenticated SCE after decryption. Preserve Wire encoding,
checkpoint schema, durable retry IDs, recovery quarantine and failure poisoning.
A successful write does not retire pending ciphertext or imply a peer receipt.

Standard PEP device/bundle publication and the bounded OMEMO-2 codec replace the
research fixture's synthetic payload shape. Full typed Ed25519 enrollment remains
the trust authority. Publish 32 already-durable prekeys and accept 25–32 in peer
bundles; no unbounded multi-device discovery or automatic session replacement is
introduced. The first sender starts actively and the recipient starts passively.

## Why this choice

The existing transport qualified verified TLS/SASL and bounded pre-DOM parsing
without building another protocol stack. The checkpoint already provides the
transaction boundary needed to keep ciphertext release and accepted plaintext
consistent with durable native state. Combining those boundaries is smaller and
more reviewable than adding a second vault, crypto engine or reconnect service.

The earlier smaller C candidate remains outside the selected profile because its
public stanza callbacks followed unbounded tree construction. A generic automatic
XMPP client would also introduce reconnect/lifecycle behavior not selected for
this first finite exchange. Neither alternative resolves durable outbox semantics
or device trust by itself.

## Dependency adoption and maintenance cost

The [product inventory](../dependencies/network-mail.json) records the actual
combined locked Linux x86_64, NetBSD ARM64 and macOS ARM64 graphs. Its checker
compares selected registry archives with root-lock checksums, verifies every
extracted source file and records licenses/features. Selected local and registry
source trees receive the no-JS filename/manifest checks. Upstream CI is not adopted
or executed; unselected target packages are not a runtime/source adoption.

The combined resolver retains product `cc 1.9.0` and selects the research graph's
`smallvec 1.16.4`. The maintained core/native inventories therefore intentionally
advance from `smallvec 1.16.3`; their prior executed bytes are preserved by the
[pinned dependency baseline](../evidence/native-network-dependency-baseline-manifest.json).
The additional `rustix` default feature aliases its already-selected `std` feature.
This is a reviewed combined graph, not a claim that every dependency byte equals
the standalone research graph. Preserve previous evidence before later changes.

The Rust source/build footprint is substantial. The prior research required a
2 GiB VM for a fresh build after retaining failed 1 GiB attempts; later runtime
observations at 1 GiB are a separate result. This adoption does not turn either
observation into a new integrated-client compiler, SBC or capacity guarantee.
Exact current source counts and pins belong in the generated inventory rather
than another manually maintained list.

## Consequences and acceptance boundary

The member guide is [Manual encrypted home mail](../../docs/online-mail.md).
Home credentials remain hidden local input and memory-only, separate from vault
secrets. The home receives its own account password inside verified TLS and sees
necessary public key/routing metadata. It receives no private device key or mail
plaintext. No server-readable fallback or remote private-mail rendering is added.

The manual cycle preserves pending work after ambiguous delivery and requires an
explicit local retirement decision. Full/expired/revoked/recovered state remains
visible and requires deliberate maintenance. New local devices also require
explicit handling of old public home entries and renewed peer verification.

The issue's native two-home encrypted subject/body/reply, failure, recovery, PTY,
source-matching and teardown checks remain acceptance requirements. Earlier
[research transport observations](../evidence/xmpp-transport/README.md) are not
fresh integrated-client results. No full XMPP/OMEMO conformance or security audit
is implied.

Parent [#90](https://github.com/gonz0w/faction-bbs/issues/90) stays open. Reliable
offline/archive synchronization needs its own finite work; background/private-chat
UX is [#99](https://github.com/gonz0w/faction-bbs/issues/99), roaming is
[#91](https://github.com/gonz0w/faction-bbs/issues/91), and attachments are
[#88](https://github.com/gonz0w/faction-bbs/issues/88). No existing server resource
limit, admission policy, owner playground state or deferred hardening scope changes.

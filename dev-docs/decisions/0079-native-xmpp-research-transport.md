# ADR 0079: Narrow native XMPP transport qualification

Status: accepted for research qualification only, 2026-10-25.
Authority: [#208](https://github.com/gonz0w/faction-bbs/issues/208).
[Product adoption #209](https://github.com/gonz0w/faction-bbs/issues/209) remains
blocked until this qualification merges. No product dependency, live account,
provider, deployed listener or mailbox is adopted by this decision.

## Choice

Carry tokio-xmpp 6.0.0, native-tls 0.2.18 and an incremental rxml 0.14 admission guard
into the first optional home-server adapter. Qualify verified direct TLS (1.2 on macOS;1.2/1.3 on NetBSD),
CA+hostname+DER leaf pin, SASL PLAIN only after channel verification, explicit
resource binding, one attempt, bounded XML and cooperative cancellation. Preserve
compile-disabled protocol logs and key-log environment refusal. Use the lower-level
stream/auth APIs; do not inherit the general Client's automatic reconnect policy.

This reuses an existing Rust protocol implementation rather than building another
XMPP/SASL stack. The required guard is small and uses the existing XML parser;
actual fragmented/coalesced restart and exact-limit tests qualify its boundary.
The experiment exchanges synthetic ciphertext bytes only. It does not replace the
existing native OMEMO/SCE implementation, typed device trust or encrypted vault.

The other candidate, stock libstrophe 0.14.0, is a smaller C source tree but exposes
stanza callbacks only after unbounded tree/text construction. Its read-buffer size
is not an input-size/depth limit. Adopting it would need a different parser hook or
maintained fork, not a documented callback check. No such change was qualified.
It remains an explicit rejection here, not a claim that the library is generally
unusable. Both candidates' source pins/terms are in the
[comparison](../research/0055-native-xmpp-transport.md).

## Costs and consequence

The selected NetBSD graph has 127 registry packages and a substantially larger
source/build footprint than the C candidate. Two-job and serial release builds
failed with rustc SIGKILL in the 1 GiB guest; kernel out-of-swap was captured for the
two-job phase. The same unmodified dependency code built on a 2 GiB compiler
VM. Runtime qualification then returned to 1 GiB. Compiler memory is a deployment
tradeoff; it is not hidden inside a small one-client RSS observation or erased by
changing optimization/runtime limits.

Keep the exact [contract](../native-xmpp-transport.md), locked selected graph and
third-party licenses. Upstream CI contains ineligible marketplace-action choices;
only the explicit local no-JS build/test workflow is selected. No product Cargo
or default service/resource setting changes. The ordinary BBS and offline vault
work without this experiment. Removing it requires no database/vault migration.

The follow-up must demonstrate real encrypted messages through two invited home
servers, deliberate device/bundle trust, atomic transport/checkpoint coupling and
local UI without a server plaintext fallback. Parent #90 stays open. Archive
reliability, visited-board notifications, roaming and attachments remain separately
scoped. This decision makes no audit, full-XMPP, physical SBC or capacity claim.

# Native home-mail software observations

Issue [#209](https://github.com/gonz0w/faction-bbs/issues/209), 2026-10-25.
The [immutable results](results.json) qualify the default-off
[member-local adapter](../../native-home-mail.md), with an explicit manual cycle
and one independently verified peer. They do not establish a reliable offline
inbox, full XMPP/OMEMO conformance, an audit, hardware support or capacity.

```sh
python3 tools/check_home_mail_evidence.py
```

This verifies saved result and source identities. It does not start a VM, contact
a home or repeat the recorded tests. The [member guide](../../../docs/online-mail.md)
explains the actual build, private configuration, first-sender preparation and
send/receive workflow. [ADR 0080](../../decisions/0080-native-home-mail.md) records
the exact dependency/profile adoption.

## Two independent native homes

The final complete native fixture used NetBSD 11 ARM64 with two cores and 1024 MiB
configured RAM, observing 1054990336 RAM bytes. Two distinct unprivileged home
identities ran Prosody 13.0.7 with separate private directories, certificates and
stores. [Runtime provenance](fixture-provenance.json) matches startup logs and
executed Lua source to the official 13.0.7 archive. The separately installed
13.0.3 package supplied prerequisites; its stock service was unused. A standalone
version command failed without a configuration; that failure is retained rather
than substituted for the startup observations. All selected C2S/S2S listeners were loopback-only. Only those invited test
homes could federate; no real provider or public listener was involved.

Fresh generated accounts/devices passed all eleven controls: deliberate full
Ed25519 enrollment, durable public-key publication, encrypted subject/body and
reply, exact persisted pending retries after reopen, bounded duplicate history,
and no known private content in either home's message storage or logs. Wrong
leaf pins, rejected account passwords, substituted device lists, changed full
identities and server-added devices preserved the existing binding and canonical
checkpoint. A nonce-bound test-only home hook recorded that the added device was
never addressed. It never became a product module or trust authority.

The final run made 27 native actions in 24.342 seconds. Four deliberate denials
returned the expected Pin/Auth/PeerUnavailable/Protocol errors. Pending ciphertext
remained pending after writes. The positive fixture is a coordinated online
exchange, not retained offline delivery or an exactly-once receipt.

## Compatibility, bounds and cost

The result records host and native Rust tests, real terminal controls and the
20-case TLS/parser matrix separately. The latter uses an independent minimal TLS
peer and synthetic payload; it tests transport boundaries, while the two-home
fixture uses actual native OMEMO/SCE encrypted subject/body. Darwin observed TLS
1.2; native NetBSD observed TLS 1.2 and 1.3. No unsupported host TLS version is
inferred. Default-off rejection and the older checkpoint/v1-vault/board-picker
terminal checks remain separate acceptance controls. Their older result booleans
refer to those offline paths, not the availability of the separately invoked
online feature. Both host and native passed 104 product Rust tests, 25 transport
Rust tests, nine online PTY controls and 23/13/8 checkpoint/vault/picker controls.
The single intentionally ignored crash helper is invoked by its parent test.

The complete builder used 2 GiB RAM and one compilation job. A standalone build
initially lacked the already-qualified `cc 1.10.0` cache; its failure is retained
in [compiler observations](compiler.json). Supplying that exact reviewed source
resolved the cache failure without changing locks or source. The root product
uses `cc 1.9.0`; the separate transport crate's release optimization is 3, while
the root product remains at 2. Later runtime and cached incremental compilation
at 1 GiB do not establish a fresh complete 1 GiB build.

The cost row measures one complete native `faction-client --mail-online` process:
checkpoint unlock, a verified TLS/account/publication/pending-write cycle, hidden
input and restored terminal. It excludes the compiler, generated-home fixture and
other processes. Wall time includes the scripted terminal interaction; rounded
user/system figures and process RSS do not predict total server RAM or capacity.
The old research transport's 9992 KiB figure is not this integrated-client cost.
The observed peak was **70676 KiB RSS**, with **3.22 seconds** wall time,
0.06 seconds user time and 0.02 seconds system time. The exact collection scope
is pinned in the result.

## Failures and executed sources

The [first native candidate](initial-native-failure.json) stopped at peer prepare:
the valid IQ had standard namespaced `xml:lang`, which the strict codec rejected.
Its source, original six-control harness and diagnostic source remain frozen.
Independent hand-XML tests failed before a bounded root-only allowance was added;
unknown, nested and wrong-namespace attributes remain rejected. Language metadata
cannot choose trust, time or terminal rendering. The
[successful continuation](native-continuation.json) is explicitly distinguished
from the later [fresh complete run](native-fresh-run.json).

The [nonsecret home configurations](fixture/homea.cfg.lua) and
[actual start-wrapper snippets](fixture/executed-start-wrapper-snippets.py.txt)
record loopback laboratory inputs, not an installation service. The invited-peer
policy is unchanged from the public [home template](../../../experiments/084-mail-proof/prepare_homes.py).
Generate your own certificates/accounts; private fixture profiles are deliberately
not published.

The [source stage ledger](../sources/209-home-mail/stages.json) maps actual
pre-UI/native candidates and red/refactor/green inputs to exact bytes. Unchanged
bytes resolve to the repository; changed versions resolve to digest-named frozen
objects. It includes the native read-only diagnostic, not its compiled binary or
private profile. Public test diagnostics are retained under `logs/`; private
terminal transcripts, passwords, keys, checkpoints and server state are excluded.

Actual red tests also exposed stale public-prekey publication after an interleaved
receive, expired state accepted after an await, absent early core-dump suppression,
and output serialization mismatch. The fixed tests keep their assertions and
resource limits. CI's cold-cache regression failed before selected locked graph
provisioning was added; separate review then verified both source guards from an
empty crate/source cache. No broad `cargo fetch` or unselected JS source was used.

The [combined dependency inventory](../../dependencies/network-mail.json) pins
selected Linux/NetBSD/macOS archives, extracted source, licenses and features.
[Fixture package metadata](fixture-packages.json) identifies the exact native
Prosody prerequisites; these test services are not bundled product dependencies.
The three pre-209 baseline manifests preserve all prior executed bytes/records.
Older evidence is neither refreshed nor reinterpreted as integrated acceptance.

## Teardown and remaining boundaries

The broad matrix records `homes_left_live: true` at the end of testing, before
the separate shutdown/capture phase. The final teardown proof records the later
state; it does not rewrite the earlier observation. Likewise, `native-teardown.json`
records private host roots retained for the lead after guest shutdown;
`teardown.json` records their subsequent verified removal.

The saved result requires verified removal of the owned VM/private inputs and
closure of its listeners/processes, plus before/after checks that the saved owner
playground remained stopped and intact. Public source/cache inputs may be retained
separately; generated credentials and private home/client state are excluded.

This adapter has no MAM/RSM cursor, background reconnect, automatic enrollment,
server-readable fallback, roaming or attachments. Durable bounded archive catch-up
is [#211](https://github.com/gonz0w/faction-bbs/issues/211); parent
[#90](https://github.com/gonz0w/faction-bbs/issues/90) remains open. Existing server
admission, memory/credential limits and deferred hardening are unchanged.

The [initial record](initial-record.json) is preserved byte-for-byte as a rejected
provenance candidate. Its first saved-check run failed because pinning the record
changed the checker after a placeholder-stage source capture. The correction
restores the exact earlier bytes by their already-recorded digest and points that
stage to a frozen object. Product bytes, measurements and test outcomes are
unchanged; the corrected complete record is independently pinned.

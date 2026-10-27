# Main-menu encrypted home mail software evidence

Issue [#230](https://github.com/gonz0w/faction-bbs/issues/230), 2026-10-26.
The [immutable record](results.json) covers the optional themed main-menu entry
in the [member guide](../../../docs/main-menu-mail.md) and its separate
[mail handoff](../../main-menu-mail.md). File areas and the transfer service are
disabled for the main-menu proof; mail neither requests a transfer grant nor sends
a file public key.

## Executed controls

| Layer | Observed scope |
| --- | --- |
| Host workspace | 473 Rust tests; formatting and strict Clippy pass. |
| Optional host client | Default42 and network137 tests pass; four pre-existing network tests remain ignored. |
| Python tooling | Full391-test suite before final evidence-checker additions; guard10 and corrected probe12 results are retained separately; two later publication-layout controls bring the final guard suite to12. |
| Main menu | Eleven controls each in host/NetBSD classic and Unicode PTYs: local choice, fresh confirmation/unlock, cancellation, outage, private view, lock and same SSH return. |
| Actual SSH negatives and compatibility | Per OS: twelve protocol/current-revocation/old-default-unconfigured controls, three hostile private-transition controls, three dumb/compact controls and four standalone/inherited explanation PTYs. |
| Existing Files | Per OS: fourteen existing Files/SFTP controls each in classic and Unicode, plus two old/default native-v1 Files roundtrips. |

The NetBSD11 ARM64 VM uses two cores. The earlier selected native service/terminal/client command passes267 tests
with four existing ignores at 2 GiB; final focused service/terminal tests pass126.
Complete native client/build tests use 2 GiB; the final service correction uses a cached incremental build and focused
service/terminal tests at 1 GiB. Runtime reports 1,054,990,336 bytes of physical
memory. This does not prove a fresh whole-product build fits in 1 GiB. The
preserved older client is identified by its saved binary hash; it is not a fresh
source-build qualification of all older versions.

## Source identity and test-first corrections

The record retains actual pre-edit RED and subsequent GREEN variants for theme,
shared service, native routing, bounded nonce history and evidence tooling.
Independent review exposed normal content being mistaken for a control,
incomplete trailing controls escaping validation, and dumb/plain menu routing and
escape output. Captured regression failures precede each correction. Final
source-matched tests retain the assertions and normal limits.

All source stages name their actual bytes. Early native contexts and later Python
orchestration overrides remain distinct. Host service validation observed a client
transport edit outside the service/CLI dependency graph; both source maps remain,
with a later stable final workspace run. Setup failures, intermediate fixture
errors, admission denials of unproven cause and the selected/unselected public
crate metadata/downloads remain visible. No old measurement is refreshed to new
source or represented as a fresh run.

The privacy review found printed trace scanning could miss a secret split between
I/O records. Original probe inputs/results remain preserved. Corrected decoding
and a separate retrospective verdict bind the retained actual private captures;
the original runs are not relabeled as executions of the corrected oracle.
Repeated-row decoding follows the observed forms explained by NetBSD’s
[hexdump implementation](https://raw.githubusercontent.com/NetBSD/src/trunk/usr.bin/kdump/kdump.c);
unsupported or inconsistent records still fail closed. Raw
private captures, generated secrets, keys, mail stores and fixtures stay out of Git.

Normal eight sessions/5000-ms admission, two credential permits/kernel slots,
three owner helpers, 32768-byte combined remote output and five-second exchange
bounds remain. The mail protocol adds at most64 consumed connection-local nonces,
not a persistent credential lease. Current120-second non-menu timeout and terminal
scrollback/backpressure limitations remain. A forced-client-exit subprocess
lifetime hazard is tracked separately in [#231](https://github.com/gonz0w/faction-bbs/issues/231);
fixture cleanup uses exact owned-process fingerprints rather than name-wide kills.

This is software-fixture evidence. It establishes no physical SBC, connected-user
capacity, production deployment, new cryptography, background polling, automatic
read acknowledgement, live private chat or encrypted attachment support.

The eight marked disposable roots were removed after preservation. Their VM and
controller stopped, four loopback forwards closed, and no owned SSH child remained.
The saved owner playground stayed stopped; its six saved files retained their
full hashes, sizes and modification times. The [publication layout](publication-layout.md)
explains the unchanged raw Markdown snapshots.

Validate the preserved observations offline:

```sh
python3 tools/check_menu_mail_evidence.py
python3 -m unittest discover -s tools -p test_menu_mail_evidence.py
python3 -m unittest discover -s tools -p test_probe_menu_mail.py
python3 tools/check_evidence_sources.py
python3 tools/check_repository.py
```

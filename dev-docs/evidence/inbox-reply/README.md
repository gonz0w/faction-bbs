# Selected-record encrypted replies

Issue [#233](https://github.com/gonz0w/faction-bbs/issues/233) adds deliberate local
reply composition to the manual native home-mail inbox. The [member guide](../../../docs/local-mail-inbox.md)
and [technical contract](../../local-mail-inbox.md) describe its use and limits.

[results.json](results.json) binds the selected observations, actual source stages,
raw test logs, capture associations, compatibility controls and teardown. Run
`python3 tools/check_reply_evidence.py` from the repository root. This verifies
stored bytes and assertions; it does not rerun a VM or prove production support.
The earlier [main-menu record](../main-menu-mail/results.json) remains unchanged.

## Observed software behavior

| Layer | Actual result |
| --- | --- |
| Host workspace | 473 passed, no ignored; release tests, formatting and strict Clippy |
| Optional client | Network-mail 139 passed / 4 existing ignored; native-mail 90 / 4 |
| NetBSD selected native tests | 272 passed / 4 existing ignored |
| Python tools | 423 passed, including 12 evidence-guard and 5 reply-oracle tests |
| Local reply PTYs | 11 controls each, host/NetBSD × classic/Unicode |
| Same-SSH main-menu reply | 6 controls each, host/NetBSD × classic/Unicode |
| Existing Files/SFTP | 14 controls on each OS |
| Actual older and default native-v1 | 2 controls on each OS |
| Existing configured inbox consumer | All 14 controls pass after updating its return-prompt selection |

NetBSD compilation used two cores and 2 GiB. Runtime observations used two cores
and 1,054,990,336 bytes of physical memory in a disposable VM. The final native
build map contains 699 public inputs; nine later Python runtime helper overrides
are preserved separately and match the guest. Production Rust inputs and pinned
binaries stayed unchanged during the final runs. The host workspace/default build
and feature-enabled client are separate observations, not interchangeable binaries.

The queue controls reopen canonical encrypted storage and check exactly one new
sent record and pending intent, while preserving the original received record,
read decision, archive cursor and unrelated pending intent. The existing encryption
ratchet advances normally. Reply requires a current prepared verified peer and a
fresh reviewed confirmation; it neither imports historical trust nor sends mail.

Four unique capture IDs bind the actual main-menu results to pre-capture marker
sets and per-PID/fd/direction trace inputs. The corrected stream oracle reconstructs
split board I/O and checks generated private reply markers against that I/O and
board database/store/log files. Private raw captures, vaults, credentials, SSH keys,
configuration and binaries are deliberately excluded from this publication.
The [publication scan](publication-scan.json) checks all three frozen candidate
manifests against 81 actual generated private values and private artifact signatures,
with zero matches. Older candidate summaries saying the scan is pending predate
this separate verdict and remain immutable.

## Executed versions and failures

The native, client and lead directories retain explicit candidate manifests,
content-addressed source objects, source-stage maps and selected actual logs.
Objects preserve original bytes, including line endings and source whitespace.
They are historical execution inputs, not current agent instructions or work
tracking. Package manifests omit their own hashes to avoid recursion; the root
record pins those manifest files independently.

Actual behavioral RED includes the preserved prior binary lacking a Reply action,
missing Rust subject/eligibility interfaces, missing reply-oracle module and
selection helper, evidence registration/capture-binding/live-CI controls, and the
old configured-inbox consumer waiting for its former return prompt. The latter
log deliberately suppresses private UI; it proves the failed expectation, not a
captured view of the new menu. GREEN retains all original read/checkpoint assertions.

Setup and rejected candidates stay distinct from these RED observations: sandbox
build/bind denials, an invalid repeated-ratchet test fixture, strict Clippy style
failure, an incorrect online lock-message expectation, a shared target binary
overwritten by a default build, missing package-index metadata, a fixture-local
Python alias and a disposable Files-settings ownership correction. Final binaries
were rebuilt, pinned and rerun. A metadata-writer variable-shadowing failure and an
incorrect inline assumption about a suppressed traceback were diagnostic errors,
not product regressions. One early sandbox-denied formatting/build scratch log
was reused before preservation; its pre-format source and tool-transcript diagnostic
survive, but this package does not claim to preserve that complete scratch log or
use it as behavioral RED. Later executed variants and meaningful results are retained.

Requested delegated settings were the adopted implementer/reviewer model at high
effort. Runtime model/effort metadata was unavailable; routing remains unverified.

## Limits

These are bounded software observations. They do not qualify physical SBCs,
250-member capacity, graphical terminal acceptance or production deployment.
No new crypto, schema, dependency, remote protocol, background polling, automatic
send/read acknowledgement or credential lease is introduced. Normal resource and
approval boundaries remain. The owner playground stayed stopped and intact;
owned disposable resources were removed after review, as the final cleanup record
reports. Forced-exit work remains in #231 and broader communications UX in #99.

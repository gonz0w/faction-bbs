# Member-device checkpoint validation

[#203](https://github.com/gonz0w/faction-bbs/issues/203) validates the default-off
[native checkpoint contract](../../native-mail-checkpoints.md), including encrypted
atomic state/history/outbox, process-death recovery and strict typed peer trust.
[Results](results.json) distinguish actual host and owned NetBSD execution, fresh
independent reference controls, prior TDD inputs and offline identity checks.
Original #200/#202 records stay immutable; no saved check is described as a fresh run.

```sh
python3 tools/check_checkpoint_evidence.py
python3 tools/check_omemo_dependencies.py
```

The saved-result guard checks identities, outcomes and explicit limits; it does
not launch a VM, rerun cryptography or certify an audit. Native feature builds/tests
are also run by required CI through the optional-native job dependency. The local
v1 UI remains a drafts/pins preview; no new network inbox, live XMPP, roaming,
attachments, physical SBC or 250-user capacity claim is made.

Actual final results on both host and NetBSD:

- Checkpoint/v1 library: 17 passed. One subprocess helper is marked ignored in the
  ordinary listing and is explicitly invoked by the process-death parent tests.
- Existing client binary: 21 passed; native adapter/build/core/SCE/example: 27
  passed; disabled native feature: one passed.
- Fresh unchanged independent reference: 92 controls per route. Private process
  refusal/bounds controls: eight host and seven NetBSD.
- Existing v1 local-vault and picker PTYs: 13 + eight controls per route.

The captured NetBSD11/aarch64 VM had two vCPUs and 1024MiB configured RAM. One
functional checkpoint/restart/receive/ack test process peaked at 73252KiB RSS,
including two synthetic vaults and the unchanged 64MiB Argon2 unlock KDF. This is
not whole-client/server cost, load capacity or a hardware/power-loss result.

The record pins 258 public build/harness/metadata capture inputs matched against
the supplied guest tree (plus the host reference driver), and 19 frozen prior
inputs. A capture does not claim every file was individually compiled. Frozen
review-red, refusing SCE/example, pre-core, pre-style and CI inputs preserve actual
failed candidates; the SCE/example stubs came from their exact executed heredoc
payloads. No rejected run is counted as a passing qualification.

The owned VM/overlay, reference runtime, synthetic keys and test fixtures were
removed after export. Its actual forwarded port is closed; the saved owner
playground remains stopped with retained files/disk unchanged. Public shared Cargo
cache/binaries are retained. [#205](https://github.com/gonz0w/faction-bbs/issues/205)
owns the next bounded trusted-local screens; parent #90 remains open.

The [original capture record](initial-capture-record.json) is retained byte-for-byte.
The corrected result excludes
seven non-build Finder metadata paths; the guard verifies the original digest,
that exact exclusion and unchanged runtime input hashes and measured outcomes.
No Finder metadata bytes are distributed.

The [post-proof Linux build follow-up](linux-build-followup.json) preserves the
exact measured build script/test inputs and records the later Linux-only
`_DEFAULT_SOURCE` compiler definition. Strict C11 had hidden glibc's endian macros
and caused the first Linux CI link failure. The guard resolves those two original
capture paths to their frozen bytes and separately checks the current build/test
hashes. Vendor/runtime code and NetBSD/Darwin definitions are unchanged. Captured
compiler-command regressions are not foreign compilation; genuine Linux linking
is validated by the required current-head CI. No new VM run is claimed after this
build-only follow-up.

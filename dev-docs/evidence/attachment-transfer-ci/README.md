# Parallel attachment recovery fixture correction

This is the follow-up observation for [#260](https://github.com/gonz0w/faction-bbs/issues/260)
in [PR #262](https://github.com/gonz0w/faction-bbs/pull/262). It preserves the
[original attachment record](../attachment-transfer/README.md) and its fixed
`6645e4752fd4e32b6e7b2e8b970858fdd90ff3aa` inputs without refreshing their hashes.
The new record supersedes only the two recovery-test input identities.

## Observed failures and correction

The first hosted run passed Optional native mail, then failed the default-parallel
workspace run at an immediate store reopen: the fault test received `Busy`.
[Results](results.json) identify the exact completed run/job and original log
hash. Unchanged-source local parallel runs reproduced that failure and also
showed timestamp-only fixture-name collisions. A new fixture-lifetime regression
ran RED before the guard: another fixture entered while the first was still live.

The correction holds a test-only guard from before private fixture creation until
after cleanup, including child launch/reap and immediate reopen. Atomic suffixes
make private and public test names unique within the process. Default Cargo
parallelism and public integration-test concurrency remain enabled. No production
store, transport, lock operation, dependency, workflow or resource limit changed.

The first corrected candidate exposed another real race: the parent observed a
readiness pathname and killed the child before its marker write completed. The
parent now waits for the exact complete marker bytes within the original five
seconds. It retains the post-kill exact-byte assertion, kill/reap, and every
old/new-snapshot expectation at all five publication points. The next candidate
also exposed the same name-collision defect in the public test fixture; its atomic
suffix corrects that observed case. All failed candidate sources/logs remain
retained, including cascading poisoned test-guard failures.

The lock explanation is consistent with [flock open-description semantics](https://man7.org/linux/man-pages/man2/flock.2.html),
[Rust 1.96 process launching](https://raw.githubusercontent.com/rust-lang/rust/1.96.0/library/std/src/sys/process/unix/unix.rs)
and [spawn descriptor inheritance](https://man7.org/linux/man-pages/man3/posix_spawn.3.html).
Close-on-exec does not close a descriptor at child creation. The exact failing
syscall schedule was not traced. A safe duplicate-descriptor control separately
proves `Busy` until the last shared description closes; it does not claim to trace
that schedule. Production locking continues to return legitimate contention.

## Actual verification and retention

The corrected all-targets store suite passed 18 tests. Twenty complete normal
parallel Cargo replays then passed, each including the controlled child helper at
five real process-death points. These are repeated stability observations, not 360
independent test cases or a capacity claim. Serial regression, scoped all-targets
Clippy with warnings denied, formatting and whitespace checks also passed.
The separate reviewer verified original log hashes, all repeated results, exact
corrected inputs and unchanged production bytes. [Review facts](review.json)
record this second examination; hosted gates still must pass on the new PR head.

`before/` retains the two exact committed tests from `6645e47`; `after/` retains
the corrected tests. Production inputs and the completed 1 MiB TLS journey are
unchanged, so that runtime observation is not presented as a new rerun.
[Private retention](private-retention.json) pins a separate owner-only ignored
archive with the raw CI/local failures and passing captures. Raw logs and private
paths are not published. Its unique regular members and original bytes were
verified. [Cleanup](cleanup.json) records owned-root/marker/UID/mode and empty
open-descriptor checks before removing the disposable source/build root.
The owner playground was not started or changed.

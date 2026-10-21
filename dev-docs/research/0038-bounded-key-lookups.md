# Bounded SSH key-lookup work on the experimental NetBSD profile

Status: bounded experiment complete; production adoption deferred.
Date: 2026-10-21.
Issue: [#66](https://github.com/gonz0w/faction-bbs/issues/66), related #55/#23/#28.
Decision: [ADR 0023](../decisions/0023-bounded-key-lookups.md).
Plan: [0024](../plans/0024-bounded-key-lookups.md).
Stopping condition: bounded helper/state implementation, native compatibility,
eight-adversary lookup accounting and two fresh whole-host 250-member controls.
Production adoption and global connection-rate policy remain separate gates.

## Scope and alternatives

An invitation holder can ask base SSH about a known key without a signature.
Result 038 received 64 PK_OK replies on one reservation in roughly two seconds;
no authentication or local approval followed. The former helper nevertheless
repeated all cohort liveness queries, key-file reads and persistent writes.

ADR 0023 compares the fixed allowance, newer base SSH/backport, a private-network
or perimeter gate, unchanged helper and a new dispatcher. Preserve Rust/base SSH,
current local authority/Argon2/two credentials, all frontend/owner ceilings and
normal defaults eight. No JS/TS, external service or physical SBC is required.
Narrow authority/whole-host evidence gates production; do not invent weighted
scores for unknown independent review, availability and complete capacity.

## Primary evidence and inference

Retrieved 2026-10-21:

- [OpenSSH 10.6 release notes](https://www.openssh.org/releasenotes.html): new
  PubkeyOptions max-pk-ok accounting defaults to six free PK_OK tests, then counts
  additional tests against MaxAuthTries. This documented candidate is **not** an
  installed or verified NetBSD-base upgrade/backport. It still needs actual helper,
  options, auth-failure, startup and whole-host comparison before replacing this
  profile; no OS update is installed by this change.
- [Pinned NetBSD auth2-pubkey.c](https://github.com/NetBSD/src/blob/36cf5108df513943b2914535a5e7c2f86fa35fd7/crypto/external/bsd/openssh/dist/auth2-pubkey.c)
  checks key authorization for unsigned and signed requests, postponing successful
  unsigned tests. [auth2.c](https://github.com/NetBSD/src/blob/36cf5108df513943b2914535a5e7c2f86fa35fd7/crypto/external/bsd/openssh/dist/auth2.c)
  returns early for postponed tests and counts ordinary denials toward MaxAuthTries.
  Source review is an inference about behavior; the recorded base runtime and
  native adversarial results verify the selected fixture, not source-build lineage.
- [NetBSD ktrace(1)](https://man.netbsd.org/ktrace.1) documents inherited syscall/namei
  tracing and descendant-scoped clearing. Native cn traces count actual helper
  executions, key-path opens, writes/fsyncs and sysctls. Raw traces remain private
  and are deleted; sanitized syscall classes carry no PIDs, addresses or keys.
- [NetBSD sshd_config(5)](https://man.netbsd.org/sshd_config.5) separates
  AuthorizedKeysCommand, authentication attempts, MaxStartups, sessions and source
  penalties. Connection concurrency is not a global TCP/fork rate bound.

## Declared work bound

One monitor may consume at most four durable allowances. Exhaustion returns no
keys and skips key-file IO, persistence and cohort liveness scanning. Its fixed
8104-byte state read/checksum and 250-entry identity comparison still happen,
as do process startup, NSS/runtime and parent/visibility checks. First-time callers
prune at most 250 entries. A reservation remains until its real-root monitor birth
exits; restarting the listener cannot refresh that monitor's lookup allowance.

For eight simultaneously offered preauth monitors, with MaxAuthTries 3 on the
verified base path, conservative **separate upper bounds** are:

| Work | Bound per finite eight-monitor cohort | Reason |
| --- | --- | --- |
| Helper executions | 56 | At most four allowed plus three denied requests per monitor. |
| Charged key-file reads/output attempts | 32, at most 2 MiB key bytes total | Four per admitted monitor; file <=64 KiB. |
| State writes/fsyncs | 32 each, <=259328 state bytes | Four per admitted monitor; unreserved full/rate denials at most three per monitor. |
| State bytes read | <=453824 | 56 fixed 8104-byte records; checksum scans at most 453376 additional bytes. |
| Slot-prune iterations / possible recorded-process liveness queries | <=6000 | At worst all eight unreserved full/rate-denied monitors scan 250 entries three times. |
| Parent process identity queries | <=56 | One each helper; libc/NSS/visibility sysctls are additional. |

These maxima need not occur together. With k admitted monitors, helper bound is
7k+3(8-k), commits 4k+3(8-k), and prune loops at most 250[k+3(8-k)]. At full 250,
eight rejected monitors need <=24 helpers/commits and <=6000 recorded-process
queries, with no invitation-key read. With room for all eight, there are <=56
helpers/32 commits and <=2000 prune loops. Repeated recorded callers take no
cohort sysctls; fixed in-memory validation/comparison loops remain bounded.

These are algorithmic/syscall-count bounds, **not** wall-clock deadlines, CPU
throughput or whole-host flood resistance. Short reads/writes and EINTR may need
more IO syscalls; measured pwrite counts are actual complete fixture writes.
The three-second alarm is best-effort termination, not a hard bound on blocked
kernel IO. New cohorts, KEX, SSH parser/crypto and external NSS configuration
need separate global availability policy. Source penalties can deny legitimate
same-NAT callers; they are preserved, not bypassed to manufacture a pass.

## Native test contract and initial outcomes

Use a freshly verified NetBSD 11 ARM64 release image, two QEMU/HVF vCPUs and
configured 1 GiB RAM, pinned Rust 1.96.0/Python 3.13 and optional Paramiko 4.0.0
research client. The server has no Paramiko dependency. Pin host keys via serial,
use synthetic members/posts only and retain the unchanged 128 MiB/zero-swap,
process/file reserves and all base services. New drivers keep every original
32/100/200/250 stage, cold/hot credentials, 0/19/38 MiB controls, limit readbacks,
UI, loaded stop and fresh-login recovery. No compiler runs in measured windows.

Native pre-load controls: 63 Rust tests; 14 original key/state/rate/config checks;
29 shared UI/approval/theme checks; six channel/restart checks; six explicit
legacy migration/birth checks; matching-key from-option denial and ordinary
signed Paramiko compatibility. Full/rate and failed-key paths remain distinct.
The PID-reuse control simulates a wrong birth for a real PID; it does not claim
observed natural kernel PID recycling.

A traced eight-monitor known-key cohort measured 32 PK_OK replies, then two denied
replies and a disconnect per monitor on the third denial: 56 helper executions,
32 key-path reads/outputs, 32 pwrite/ftruncate/fsync calls and 476 helper sysctls.
Of these, 24 exhausted helpers have no key lookup/write/fsync and only eight
runtime/NSS/visibility/parent sysctls each. With an initially empty registry, first
admissions scan 0..7 previous live entries; recorded repeats skip those scans.
Unknown-key and signed-with-the-wrong-key cohorts each measured 24 helpers,
24 key returns/commits, 220 sysctls, no PK_OK and no authentication. Root-managed
key options remain parsed by OpenSSH, verified with a matching invitation key
whose from restriction excludes the synthetic source, while the helper can read
the key file. Successful ordinary signed Paramiko/OpenSSH admission remains
transport-only; local approval is covered independently by the shared-UI oracles.

A test-fixture mistake initially restored the invitation file with root-only
0600 permissions. The helper failed closed and a valid signed client was denied.
The corrected option test preserves the original readable mode and exact bytes,
checks factionauth readability, and passes signed admission after restoration.
This failed fixture attempt is retained as a testing limitation/correction; its
initial denial alone was not accepted as proof of key-option enforcement.

## Whole-host outcomes and remaining gates

Both native whole-host matrices pass. Each new active interval is
three minutes: start with 250 distinct useful members; offer eight full-capacity
known-key clients; keep peer chat flowing; free one real member slot to test
same-source fresh admission without clearing listener/registry/penalties; restore
that real member after native source policy permits it. Record 249 during the
recovery period and 250 before/after; never label the entire interval 250 members.
Exclude sender echo from confirmed peer receipts. This is distinct from the two
historical 15-minute active intervals in result 038.

Do not infer combined maximum preauth+credential+blocked-output/38-MiB controls,
long soak, supported capacity or a complete CPU budget. #23/#18 retain global
accept/fork/source-fairness policy; #20 raw CPU/timekeeping and maintained host
policy; #28 broad overlap/long-soak/complete budgets; #55 independent authority
review and scoped adoption. Lua/doors still need a separate launcher and UID.

## Preservation and reversal

Third buildable snapshot sources/71491cc retains all 137 executed inputs of
037/038 and pins both unchanged result files. Older manifests, nine records,
two snapshots and rejected-candidate overlays remain unchanged. New native input
hashes verify current sources directly; never update historical digests to new
code. Migration is explicit/quiescent and inode-preserving; normal v1 use fails
closed. Rollback drains transports/selects ordinary SSH and leaves local content
and export compatibility unchanged. Review ADR 0023 before future base upgrades.

## Final recorded outcomes

[Result 039](results/039-netbsd-bounded-lookups.json) saves 145 current execution
inputs with matching transferred hashes, installed binary/library identities,
three traced adversarial cohorts, every original matrix/control/limit stream,
two active intervals, fixture correction and verified restoration/cleanup.

| Fresh boot | Confirmed peer receipts | Minimum active headroom | Minimum stress/control headroom | Fresh same-source admission recovery |
| --- | --- | --- | --- | --- |
| 1 | 44,279 | 155.14 MiB | 130.36 MiB | 50.48 s |
| 2 | 44,030 | 164.08 MiB | 132.83 MiB | 50.55 s |

All sampled guards pass with zero swap; 88,309 peer receipts exclude the
sender. The runs have 178/177 witness rounds. The full-capacity attack gets exactly
24 helper denials without another key reservation or key read; source penalties
then temporarily block a new legitimate transport. Record 249 approved members
during that recovery window, rather than claiming 250 throughout. No listener,
registry or penalty reset occurs during these intervals. Minimum stress margin
above the 128 MiB reserve is only 2.36 MiB; this is experimental headroom,
not broad workload or supported-capacity acceptance.

The fixture reuses one synthetic SSH invitation key across distinct local BBS
accounts. A 250-distinct-SSH-key list/rotation and maximum 64-KiB key-file
memory/IO overlap remain unmeasured; the declared byte ceiling is a code bound.

A separate exhausted-monitor control verifies four PK_OKs, listener restart
without changing the registry inode, then denial/disconnect with no refreshed
allowance. It is independent of the no-reset active/fairness intervals. Original
state/rate/restart checks, local approval/UI and ordinary OpenSSH/Paramiko sign-in
remain positive. The successful direct signed Paramiko client charges one lookup.
After both runs the original full base-service/allocator/NTP policy is restored
and rebooted/read back; all profile SUID/mount/NPF changes are removed. Ordinary
31 frontend, 20 SSH, 13 worker-lifecycle and six native PTY controls pass. The
synthetic fixture returns to integrity=ok, 1001 distinct account hashes, 50 rooms
and 100000 posts. Only this task's VM, keys, generated credentials, disks, raw
traces/logs and forwards are deleted. No other host or VM SSH service is changed.

The initial UI fixture has 1002 accounts/52 rooms/100004 non-load posts; each
matrix adds one UI-load post. Offline integrity checks retain 100005 then 100006
posts across the two boots, before the final bounded synthetic reset. This is
not an identical database snapshot comparison. Native Rust workspace has 63
passing tests. New/current historical evidence checks and 81 host Python tests
pass; repository policy checks retain all older failures/record hashes. CI runs
the new checker and all established dependency/build/test/terminal checks.

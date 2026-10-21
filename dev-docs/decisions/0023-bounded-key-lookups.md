# 0023: Fixed SSH helper lookup allowance

Status: selected for the experimental software profile; production adoption deferred.
Date: 2026-10-21.
Issue: [#66](https://github.com/gonz0w/faction-bbs/issues/66), related #55/#23/#28.
Research: [0038](../research/0038-bounded-key-lookups.md).
Amends [ADR 0022](0022-ssh-admission.md); ordinary SSH rollback remains independent.

## Question and mandatory gates

Bound repeated successful unsigned invitation-key offers on the measured NetBSD
11/base OpenSSH 10.2 path. Keep base SSH parser/options/signatures, local BBS
approval, unchanged credentials/ceilings, dedicated helper UID, eight defaults,
process birth/restart correctness and the 128 MiB/zero-swap sampled guard.
No automatic host upgrade, firewall replacement or public exposure is authorized.

## Alternatives

| Candidate | Gate/evidence | Disposition |
| --- | --- | --- |
| Existing unlimited reservation reuse | 64 unsigned PK_OK replies in result 038; repeated state and key IO | Reject unchanged lookup behavior. |
| Four durable helper allowances per monitor | No resident daemon or SSH parser; ordinary query/signature needs native proof; adds 2000 state bytes | Select bounded experiment. |
| OpenSSH 10.6 PK_OK accounting/backport | Upstream documents six free PK_OK tests then MaxAuthTries accounting; not installed/validated as NetBSD base here | Credible future simpler upstream boundary, not a verified substitute. |
| Private network / perimeter accept-rate control | Can reduce who reaches SSH; distinct identity/NAT/availability/update policy | Complementary later #23/#18 work, not a per-monitor lookup bound. |
| New proxy/parser/resident dispatcher | Adds trusted code, lifecycle and memory; previously rejected inetd path failed memory controls | Defer; no demonstrated advantage for this uncertainty. |

Qualitative comparison: current base compatibility and measured finite lookup
work outrank speculative savings. Unknown independent review/complete availability
gates block production adoption for every candidate; no numerical score masks them.

## Decision and tradeoffs

Persist at most four charged lookups per real-root monitor PID and kernel birth.
Charge before reading/emitting the key file, including failed file IO. Recorded
monitors skip cohort-wide liveness scans. Exhaustion emits no keys, performs no
state write/fsync or cohort liveness query, and leaves the slot occupied until the
monitor exits. Base MaxAuthTries 3 then bounds denied requests on the measured
release. A normal key query and signed request retain room for the previously
observed two/three helper calls. Clients that deliberately exhaust the allowance
must reconnect; arbitrary agent/query sequences are not a compatibility promise.

Registry version 2 adds 250 uint64 counters, increasing 6104 to 8104 bytes.
Version 1 is refused during normal lookup, inspect and reset. Conversion requires
explicit root preflight --migrate-v1 after stopping the member listener/draining
all member transports. It validates old metadata/size/checksum and dead births,
retains the inode under flock, then writes a fresh v2 budget. Missing/corrupt files
are never silently repaired. A crash during in-place migration can fail closed;
operator recovery is explicit, with no live authority to erase. Mixed old/new
helper binaries must not run against the same registry. Downgrade requires drain
and explicit quiescent reinitialization with the old tooling, never a live reset.

The bound is per finite cohort on the verified SSH path. It excludes repeated new
TCP cohorts, SSH KEX/parser work, global pre-fork rate, NSS/host IO latency and
shared-source fairness. Helper launches still occur for exhausted requests; only
the expensive repeated work is removed. Source penalties stay enabled. The helper
UID/state remain trusted and unavailable to Lua/doors. No application schema,
community content, SSH crypto or credential-cost change is made.

## Confidence, reversal and validation

Source review plus native negative/positive and whole-host controls are required.
See research 0038 for exact measured counts, identities, failures and limits.
Confidence in finite helper work is bounded by those software observations; no
production security, CPU budget, all-port or physical SBC acceptance is implied.
Rollback drains transports and chooses ordinary faction_sshd; local content and
exports are unaffected. Reopen if normal supported clients need more than four
lookups, a base SSH change breaks failure accounting, or memory/authority guards
fail. Upstream adoption needs a fresh native comparison, not a release-note claim.

# 0049: Historical backup-check receipts for the local operator

Date: 2026-10-22. Status: implemented and software-validated. #95/#18/#98.

## Context and decision

The dashboard has real host counters but no backup health source. It must not
claim success from a file's existence or run a privileged backup/restore on refresh.
Use the existing complete file-area backup helper's catalog/blob verifier, shared
by a new verify-only operation and restore. Default check publication stays off.
An explicitly authorized operator can request `--record-status --board PATH` for
backup/verify, producing one private numeric historical receipt. The dashboard
reads only that small receipt in its existing OS-owner/root context.

A background scanner would repeatedly traverse private DB/blob content and require
scheduling/resource/authority decisions. A dashboard action would mix the current
read-only UI with privileged service/restore control. Defer both. A historical
receipt makes this workflow useful without inventing adapters or host authority.

## Semantics and bounds

At most 512 bytes, exact version/order/fields, numeric counts/results/timestamps;
no paths/content/member identifiers/keys. Atomic owner-private publication keeps
latest attempt and last verified metadata separate. Failed checks preserve prior
last-success time/counts, not a current success badge. Missing/unsafe/unknown and
future-clock states remain explicit. Existing modest refresh/history budgets,
member/service defaults and no-JS policy stay unchanged.

The operator associates an artifact with the chosen board directory. Verification
is structural/content integrity of a selected bundle, not authenticated provenance,
automatic live-board identity/freshness, continued availability, off-host retention
or successful restore. Themes/OS/SSH and member-device keys are outside the bundle.
Full restore still validates every object before reserving any fresh destination.

## Evidence and reversal

The [native evidence](../evidence/operator-recovery-netbsd/README.md) and [GitHub issue](https://github.com/gonz0w/faction-bbs/issues/95) for actual tests
and [operator documentation](../../docs/backup-checks.md) for limitations. Validate
native root CLI with distinct owner/file UIDs separately from single-UID developer
API fixtures. Repeated checks must leave bundle bytes and restore destinations
unchanged; corruption must record failure and refuse fresh restore.

Low reversal cost: stop requesting publication, roll back the dashboard/helper and
discard telemetry after all writers stop. No DB/schema/backup format migration,
new listener or service privilege. Stronger containment/crypto and peer protocols
stay in their own later work. Do not turn these checks into supported capacity,
physical hardware or production-hardening claims.

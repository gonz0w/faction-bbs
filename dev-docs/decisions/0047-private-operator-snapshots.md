# 0047: Opt-in private operator snapshots

Date: 2026-10-22. Status: implemented first software slice, #95/#18/#98.
Authority: owner requested a local OS-console sysop view and grouped related work.

## Context and non-negotiables

The shared BBS owner holds local board/session authority. An OS operator needs
read-only aggregate status without a member SSH escape, additional BBS listener,
private content/keys or an unbounded collector. Keep normal limits and no JS/TS;
software NetBSD validation suffices. Host actions and later hardening stay separate.

## Options and decision

| Option | Evidence/gates | Consequence |
| --- | --- | --- |
| Read live SQLite from every dashboard refresh | Separate readers are feasible but require exposing DB/lock/schema handling to the UI. | Reject for this slice; no dashboard DB opens/migrations. |
| New privileged administrative socket/API | Would require another listener and authorization/lifecycle contract. | Defer until actual actions need a reviewed narrow operation boundary. |
| Owner-produced bounded private aggregate file | Existing owner state directory and OS permissions; no new helper/listener. | Adopt explicitly opt-in, read-only telemetry only. |

Every five seconds an enabled publisher takes aggregate counts under a bounded
SQLite progress budget and zero busy wait, restores prior connection settings,
and atomically replaces an at-most-2-KiB private snapshot. An explicitly launched
`faction-sysop` reads it as its Unix owner/root. It contains numeric metadata only,
not authority, names or content. Snapshot age/running markers are observations,
not proof that a PID is live. Refuse untrusted files and distinguish unavailable,
stale, stopped and clock-unknown data.

The separate console uses existing terminal styles/decoder, bounded 60-sample
history, delta ANSI and plain fallback. Local host collection is fixed/read-only,
outside the owner; no root command field or implicit BBS role elevation. No getty,
login profile, network/firewall or clock changes are installed automatically.

## Validation and confidence

Prototype required for authority, live-owner interaction, telemetry costs and TTY
restoration. [GitHub issue](https://github.com/gonz0w/faction-bbs/issues/95) and
[native results](../evidence/operator-netbsd/README.md) record actual Rust/UI,
private-file/budget/privacy/export, member-denial, restricted SSH, resize and
service lifecycle observations on a two-core/1-GiB NetBSD VM. Confidence is scoped
to this read-only software slice. Source snapshots preserve prior measurements.

## Reversal and open work

Low reversal cost: disable the optional key under stopped restart, use the earlier
binary and discard telemetry. No database/schema/account/key migration. Matching
old config must omit the new optional key; never unlink live lock files. Existing
BBS features retain their authority/defaults. No capacity/hardware/sandbox claim.

#95 retains used/free RAM, CPU utilization, network/peer/backup health and fuller
console coverage. Host write/apply/rollback #98 and abuse policy #97 need separate
narrow designs. Reopen this boundary if data stops being aggregate telemetry or
member/host actions are introduced; do not add commands to the snapshot protocol.

# 0064: Device-local interactive travel preferences

Status: accepted for the hobby preview, 2026-10-23. Issue: #159 (partial increment).

## Context and decision

A member should choose a lighter visit or decline large file payloads before
connecting. Add a small in-memory Travel value and local picker panel with draft/
apply/cancel, plus fixed CLI flags. Keep ordinary/16-MiB defaults. No persistence,
remote setters, account replication, route change or new registry/helper/service.

Compact requests `TERM=faction-compact` and the updated service uses its classic path;
optional board graphics stays disabled. Remote text still passes the existing
filter. File ceilings gate known payload sizes before upload reservation/download
grant/SFTP; allowed files retain current confirmation/size/checksum/authorization.
Zero deliberately defers all files. No new schema; the service recognizes the explicit compact-native hint.

Choose explicit per-file limits rather than an inaccurate encrypted-wire/session
budget. Protocol overhead and retries differ; a future measured wire budget would
need separate accounting/cancellation semantics. Offline exchange/reply sync has
no verified packet format yet and remains #145/#159. Route settings remain separate.

[The guide](../../docs/travel-modes.md) defines usage, classic/local UI distinction,
current costs/limits and recovery. Rollback simply uses the old client and removes
new flags; device data and board schema 14 are unchanged. [Evidence](../evidence/travel-netbsd/README.md)
uses owned software fixtures only. No new throughput/capacity/physical-radio claim.

A native test showed that `TERM=dumb` alone intentionally suppresses the Files
advertisement. Use the explicit compact-native hint to advertise the existing
bounded Files protocol without enabling ANSI display. Ordinary dumb/manual SSH
keeps escape-free Files; the hint grants no admission/file authority and every
native operation still checks current local approval/grants. The worker/local
console also treats the reserved hint as plain presentation. Matching updated
service/client versions are required for this negotiation.

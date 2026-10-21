# 0029: Fixed classic-server door before local legacy execution

Date: 2026-10-21. Status: implemented optional preview candidate.
Issues: [#77](https://github.com/gonz0w/faction-bbs/issues/77),
[#34](https://github.com/gonz0w/faction-bbs/issues/34).

## Question and mandatory gates

Provide a useful path toward original games without rewriting them or expanding
Lua into a network/host account. Owner accepted both server reuse and isolated
DOS hosting; preserve local approval/data, no JS/TS, low bounded memory, terminal
safety, default-off operation and original backend/credential/process limits.
Original title/runtime/license acceptance is separate from connector acceptance.

## Alternatives and decision

- Rewriting titles fails the reuse-first priority before original reuse is tried.
- A local DOS/native launcher needs its own execution/UID/filesystem/node boundary;
  it cannot inherit the backend's zero-process budget. Accepted follow-up #78.
- A separate TCP proxy adds another service/identity/IPC protocol and a deployment
  cost for the first simple client. It could narrow parser blast radius later.
- Choose direct bounded safe-Rust TCP/Telnet parsing in the shared owner, pointing
  only to trusted-config literal endpoints. The original game executes on the
  separate host. No script/member destination resolution or local execution.

This deliberately expands the shared owner's protocol parser surface. Memory-safe
bounded parsing and tests reduce specific risks; they do not create a parser
process sandbox. The runtime/OS remain trusted, and a compromised game server is
untrusted terminal input. Unknown controls are discarded; malformed over-budget
streams close the door. No arbitrary endpoint, terminal clipboard/title/device
query, BBS identity/session export or score-submission action is exposed.

At most six configured destinations and eight active remote sockets fit the
ordinary service's existing descriptor/process/thread envelope without raising
limits. Per-connection reads/output/pending writes, connection/write/idle deadlines
and total/window byte limits bound work; normal eight sessions and two credential
permits/locks stay unchanged. This is not a measured 250-user memory result.

Legacy Telnet must use a sysop-managed protected/loopback path; the BBS's SSH leg
alone does not encrypt it. Member confirmation shows destination and explains
separate game credentials before any socket opens. Ctrl-] returns locally;
EOF/failure/resize/idle returns to the BBS, while revoked local membership closes
the member connection. Remote game scores/saves stay with their own host.

## Sources, evidence and reversal

Primary protocol and title sources, retrieved 2026-10-21, are linked in the
[reuse inventory](../research/0039-classic-door-reuse.md).
[Connection/codec](../../crates/faction-service/src/classic.rs),
[UI](../../crates/faction-service/src/classic_ui.rs) and
[configuration](../../crates/faction-service/src/config.rs) define actual behavior.
Unit tests exercise fragmentation, options, CP437, unsafe controls, limits, real
TCP and eight-slot release. A marked NetBSD VM/owned server exercises actual SSH
menu/confirm/input/return/revocation and Lua score coexistence; see
[plan 0030](../plans/0030-classic-server-doors.md) for actual results.

Reversal cost is low: remove config entries and restart, leaving local schema-3
state unchanged. Existing old custom Lua lobbies need an explicit classics target;
updated examples hide it when no destinations exist. The Lua installer prepends
its root key before array tables. No existing remote-server game state is migrated.

Confidence is limited to the recorded connector/fixture scope. Original TWGS
and DOS game artifacts/manuals/licenses, real title play and compatibility-host
costs remain gates in #79/#78/#34. Broader Telnet dialects, Unicode server streams,
rlogin/dropfiles and stronger parser isolation are follow-ups, not hidden support
claims. Independent review remains part of production readiness.

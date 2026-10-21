# Build file areas and restricted SFTP

Issue #83. Owner authorized merging #82/#85 and building this increment.
Status: implemented and validated; PR review pending. Ordinary eight-session BBS remains the baseline.

## Outcome

A member browses/searches local areas, requests a key-bound upload/download and
uses a separate restricted SSH/SFTP service. Private uploads reserve quotas and
need sysop review before publication. Every transfer rechecks current local
approval/epoch/area permission. No BBS shell/SFTP escape, file federation, private
mail/auth migration or backend helper/credential-limit expansion.

## Architecture

Core schema 4 owns catalog, area grants, quota reservation and short-lived one-use
transfer grants. Native SFTP v3 handler owns bounded byte I/O under factionfiles;
base OpenSSH handles transport/signatures, with a separate factionfileauth key
lookup helper. A bounded UID-checked owner policy socket serves no file bytes or
member sessions and adds no owner process/thread. Other future adapters use the
same semantic policy methods. Store paths are opaque generated object IDs.

Compare base internal-sftp (chroot/request filters, no per-request BBS policy)
with a small virtual single-object handler. Select the latter for this increment;
actual client, permission, path, interruption, restart and memory tests gate claims.
Initial streams: two; file 16 MiB; board 256 MiB; grant five minutes; chunk 32 KiB.
These are conservative preview caps, not capacity results. Resume is not provided
in v1: interrupted files remain private/charged until review/remove and GC.

## Work and checks

1. Merge #82/#85 after required checks and preserve all earlier measured source
   bytes/record identities before edits. Historical JSON remains unchanged.
2. Implement catalog/policy and schema-2/3 export compatibility, bounded IPC,
   separate SFTP handler/key lookup and local file-area UI.
3. Add opt-in NetBSD setup with separate UIDs/listener and no automatic service
   start/network policy changes. Implement stopped/drained file-data backup/restore.
4. Verify real SFTP upload/review/download/hash, key replay/wrong area/pending/
   active revocation, traversal/symlink/hardlink/oversize/quota/interruption and
   service restart. Preserve prior BBS denial/credential/lifecycle checks.
5. Run native NetBSD VM software tests and relevant host tests/dependency/policy
   checks. Collect sanitized results/footprint and clean private VM inputs.
6. Update README/guides/AGENTS/changelog/backlog and open a focused PR. Do not
   claim physical SBCs, arbitrary SFTP operations or 250-user capacity.


## Actual outcome

Merged #82 (`2ff768e`) and #85 (`daae757`) after checks; preserved pre-change
execution bytes in sources/pre83 before runtime edits. Schema 4, shared Files UI,
separate lookup/transfer identities, scoped SFTP v3, disabled installer, explicit
GC and complete file-data bundle/fresh restore are implemented. Sysops can inspect
completed pending uploads without exposing them to members. The handler refuses
root/mismatched real/effective identities; no owner thread/process/credential or
normal session/admission ceilings were relaxed.

[Native evidence](../evidence/file-areas-netbsd/README.md) records 13 real SSH/SFTP/
recovery checks, 90 native and 90 host Rust tests, 101 Python tests, existing PTY
flows and all historical evidence checks passing. One RSS observation is recorded
separately from configured quotas; no peak/whole-host/capacity claim. Teardown is
verified. Physical hardware, 250 users, arbitrary SFTP operations and resume were
not tested/adopted. The first preview deliberately uses conservative fixed caps.

Follow-ups remain separately scoped: self-service cancellation/partial retention,
configurable policy caps, more client interoperability, classic in-session transfer
adapters and native-client/home identity/encrypted mail (#84). Files remain local.

The first PR CI run exposed stale standalone experiment lockfiles after the core's
base64 addition. Refresh only those four current lockfiles; leave the historical
measured versions in the frozen snapshots. The research UI explicitly keeps Files
unavailable. Dependency checks allow a standalone feature subset of the reviewed
workspace graph while retaining exact package/version/license and no-JS checks;
unknown features/versions/licenses are rejected. All four Rust-1.96 research builds/
lints, both applicable research test suites, dependency checks and unchanged saved
evidence checks pass. This compatibility work is not a new native capacity trial.

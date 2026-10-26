# Encrypted device-local read state and parked Files entry

Implementation authority is [#216](https://github.com/gonz0w/faction-bbs/issues/216)
and [#217](https://github.com/gonz0w/faction-bbs/issues/217), finite children of
[#99](https://github.com/gonz0w/faction-bbs/issues/99). Member instructions are in
[local unread mail](../docs/local-unread-mail.md). This extends the existing
[native checkpoint](native-mail-checkpoints.md), [manual home adapter](native-home-mail.md)
and [archive contract](native-mail-archive.md), not their cryptographic protocols.

## Encrypted representation and local authority

The `FACTVLT2` envelope, KDF, password/recovery wrappers, AEAD associated data and
native protocol snapshot are unchanged. `FACTCP02` and `FACTCP03` retain their exact
encoding when tracking is disabled. Explicit `enable_read_tracking()` publishes
`FACTCP04` through the existing atomic canonical commit. It retains common fields,
a strict archive-present boolean and optional original archive payload, then a
bounded count and strict boolean per history record. Exact EOF, length equality
and received-only true bits are validated. No plaintext sidecar exists.

Read bits align with append-only history, at most 64 entries. Selection checks both
index and the retained authenticated received-frame digest. A stale index, sent
record or wrong digest cannot acknowledge another message. Idempotent repeated
acknowledgement performs no publication. Newly accepted records append false;
duplicates and known own echoes preserve existing bits. No timestamp ordering or
server archive-ID comparison supplies read authority.

`ReadSummary` exposes only local counts and history-only status. Recovery, expiry
and local peer revocation do not prevent reading retained history or making local
read decisions, and those decisions cannot reactivate protocol state. Backup,
reset and fresh-device paths preserve matching history/read metadata. Publication
failure retains the existing poison/drop/reopen rules; do not unlink a live lock.

The trusted console acknowledges only a locally selected received message after
its subject/body writes succeed, input is flushed, and the member types `read`.
Body display or a skipped/cancelled prompt does not itself acknowledge. This is an
explicit device-local decision, not proof of comprehension or remote delivery.

## Parked session and generic notice boundary

A trusted local catalog supplies the optional home paths. Only the completed local
native Files desk offers E/N. Existing native RPC completion supplies the parked
boundary; no remote content decides that a draft or game is safe to interrupt.
The same `Remote` and SSH child survive local mail. A fresh local confirmation and
input flush precede hidden unlock; trailing menu input cannot supply credentials.

A bounded ordinary `areas 0` RPC checks current board access before the private
console and after it returns. No pump/render/send of remote data occurs during the
private call. Existing bounded PTY backpressure may make a peer disconnect. On
return the vault/secrets/inner terminal guard have dropped; viewport clear and
input flush precede remote handling. Unsolicited text is discarded through the
existing bounded RPC parser; unexpected frames, EOF or revoked access stop the
connection. No new worker/thread, unbounded capture or reconnect loop exists.

Notices start off per visit. Explicit N opts in only to generic presentation at
the safe Files menu. A successful local lock supplies a counts-only snapshot;
errors/cancellation invalidate it, and opt-out discards it. Locked/unknown,
disabled tracking and stale retained-device counts remain distinct. No home
credentials are retained to refresh them. The server can observe timing and a pause
in member activity, but receives no notice count, private content or read receipt.

The BBS schema, approval/epoch/grant checks, admission/credential/helper limits,
Argon2, bounded output, five-second drain and file bounds are unchanged. This is
a finite manual in-session slice. Background polling, general draft/game pause,
private live chat, roaming and attachments need their own executable issues.

# 0042: Unicode subject composing

Issue #114. Status: implemented and validated, ready for review, 2026-10-22.

- [x] Fix optional INN Linux CI (explicit disabled sendmail path), exact-head green,
  and merge #115 as a5c04654a632c13a490195c362484978293141f2.
- [x] Freeze all 100 original #115 build/run inputs and unchanged result before edits.
- [x] Add schema-7 metadata, strict old catalog/ASCII validation, atomic rebuild,
  stopped upgrade/complete backup/rollback and read-only versioned snapshots.
- [x] Integrate one bounded UTF-8 subject editor into local/worker/shared terminals;
  safe classic echo, preview clipping, scalar erase and reply/default/retry behavior.
- [x] Run 140 Rust tests on host/native NetBSD, five real local/worker PTY controls
  per host and twelve shared native UID/PTY controls, including revocation.
- [x] Finish evidence cleanup/guards, docs and focused PR.

## Scope and evidence limits

160 bytes is distinct from display cells. Compatibility rendering remains escaped
ASCII, not native Unicode glyphs/graphics (#96). No new package/version/feature,
owner helper/thread, session default or credential/resource ceiling. No physical
hardware, 250-user load, feed/provider, public listener or hardening activation.
The saved owner playground stays off and unchanged. Preserve eleven snapshots and
twenty-three old result identities; new observations receive a separate record.

## Discoveries

INN's Linux configure requires a sendmail path even for its offline reference;
/usr/bin/false avoids adding/contacting a mail service. Malformed local subject
input originally returned a recoverable action error; use a fatal IO error so its
closure behavior matches the shared protocol and never interprets leftover input
as menu commands. Tests retain the unchanged StaleSession revocation contract.

Final review also caught an overbroad textual range edit in the request-ID check;
restore the original 16–64-byte bound and explicitly assert rejection at 65/74
bytes. Repeat host/native tests on those final bytes before recording/publishing.
The fresh record supersedes only this uncommitted increment's initial trials;
all previously published evidence stays byte-for-byte unchanged.

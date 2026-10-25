# ADR 0081: Explicit encrypted archive checkpoints

Status: accepted for optional product implementation, 2026-10-25.
Authority: [#211](https://github.com/gonz0w/faction-bbs/issues/211), following
[ADR 0080](0080-native-home-mail.md) and
[ADR 0078](0078-encrypted-native-checkpoints.md).
This decision defines behavior; it does not declare native acceptance complete or
authorize a deployment, release, public listener or new external service.

## Decision

Add an explicit manual own-home MAM2 page to the default-off `network-mail` client.
Fetch at most four entries for the already independently verified peer. Require
the matching own archive, query IDs and finishing response before applying any
entry. Keep the existing TLS configuration, one attempt, cooperative deadline and
bounded XML transport. There is no automatic polling or page loop.

Store the archive position inside the encrypted checkpoint using a deliberate
`FACTCP02` to `FACTCP03` inner-format transition. Preserve the outer `FACTVLT2`
encryption/wrappers and unchanged schema-02 encoding. A scoped random epoch and
exact compare-and-swap token bind the saved opaque position to this home and local
device. Unlock accepts both versions without silently upgrading either.

Reuse the native authentication and atomic checkpoint publication boundary. Each
accepted message, native key transition and archive position publish together.
Duplicates advance the position without decrypting again; a known own sent copy
must match retained immutable sent history and never becomes a delivery receipt.
There is no second cursor database or plaintext history file.

Expose local **G** confirmation for the format change and **A** for one page.
In archive-enabled stores, publication, preparation and sending omit presence;
**N** sends pending ciphertext without live receive. **A** alone accepts retained
archive messages. This prevents unsolicited offline pushes from changing native
state outside the saved position. Stores still using schema 02 retain the previous
live synchronization path.

Serialize one standard empty storage hint on every outgoing encrypted message,
including existing schema-02 sends and retries. This requests retention of
bodyless ciphertext while preserving Wire, operation IDs and ciphertext. The home
can still refuse, omit or expire storage. The hint creates no receipt or guarantee.

## Why this boundary

The existing checkpoint already owns private native state, accepted history,
durable ciphertext, recovery quarantine and atomic publication. Extending that
transaction boundary avoids independently advancing a cursor past an uncommitted
message. A plaintext side file or a second database would add coordination and
recovery failure cases without solving authentication.

An explicit inner-format migration makes the new receive semantics and old-client
incompatibility visible. Automatic migration would make merely opening a store
change its compatibility and behavior. Keeping both receive paths active after
migration would permit a live frame to advance native state before the archive
position that later describes it.

One bounded page is sufficient for deliberate offline catch-up within the existing
small retained store. A generic archive synchronizer would require additional
retention, pagination, cancellation, multi-device and background policy. This
change does not adopt those policies or a new protocol/crypto dependency graph.

## Consequences

The [member guide](../../docs/archive-mail.md) explains backups, explicit upgrade,
send/receive separation and unavailable positions. The
[technical contract](../native-mail-archive.md) fixes supported protocol shapes,
metadata handling, limits, recovery and per-entry atomicity.

Server delays, archive IDs, counts, ordering and completion flags are untrusted
metadata. They cannot supply authenticated SCE time, enroll an identity or retire
pending work. The client rejects observable query/scope/duplicate mismatches but
cannot prove that opaque server history is complete or globally ordered. A bad
later ciphertext may leave an earlier authenticated entry durably accepted; a
malformed page before fin leaves all entries unapplied.

Existing full-history/outbox, expiry, revocation, history-only recovery and poisoned
handle restrictions apply before cursor advancement, including duplicates. A
backup does not resume active native keys. A fresh device retains history but
requires renewed peer verification and a new archive scope. There is no automatic
cursor reset, skip, history eviction or downgrade.

Acceptance requires separate source-matched host and NetBSD two-home, process-death,
terminal, compatibility and teardown observations. Historical source/record pins
remain unchanged; synthetic validator tests do not constitute runtime evidence.
Complete-build memory and runtime memory are separate observations. No physical
hardware, supported capacity, full protocol conformance or security-audit claim
follows from this decision.

Parent [#90](https://github.com/gonz0w/faction-bbs/issues/90) remains open. Background
private-chat UX is [#99](https://github.com/gonz0w/faction-bbs/issues/99), roaming is
[#91](https://github.com/gonz0w/faction-bbs/issues/91), and attachments are
[#88](https://github.com/gonz0w/faction-bbs/issues/88). Ordinary board operation,
classic access, the stopped owner playground and existing server limits are
unchanged.

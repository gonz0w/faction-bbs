# 0053: Bounded explicit multiline room writing

Status: accepted implementation direction for the first #141 slice.
Date: 2026-10-22. Related: #133/#141/#21/#12; ADR 0052.

The room browser now gives members a destination but the body prompt accepts
only one line. Thoughtful conversation needs paragraphs, a chance to review and
a clear send action. Add `/edit` at that prompt; retain the one-line shortcut.

Use one shared in-memory draft model and the existing inert paged room presenter.
Enter adds a line; exact `.preview`/`.send`/`.cancel` commands separate writing
from publication. Add undo/clear and all-or-nothing freshly authorized reply
quoting. Only send creates a request ID; existing core posting and busy retry
remain authoritative. Preview/quote never alter durable unread state. Store at
most 1024 body bytes / 32 lines, with current approval/room/parent checks on each
action. No per-member history, new helper, schema or dependency.

A line desk works with plain SSH, worker rollback and shared-owner backpressure.
A full-screen editor adds caret/viewport/resize work; native-only editing would
exclude ordinary clients. Both remain possible later. This slice deliberately
leaves #141 open. Device-side versus opt-in server-side room-draft persistence
must pass the ownership/quota/retention/export/recovery gate in
[the product guide](../../docs/writing-desk.md) before adoption. No server-readable
private mail, implicit draft persistence or external editor launch is introduced.

Validation requires host/native unit suites, private local/worker PTYs, actual
restricted SSH and native-device journeys, explicit no-submit/read-ledger checks,
reply/Unicode/limits/cancel/disconnect/resize/revocation and duplicate-free busy
retry. Preserve old executed bytes/results before edits and pin new actual inputs.
No fresh capacity, physical-device or broader hardening claim is intended.

Reversal cost is low: remove the optional branch/desk modules without changing
stored posts, existing one-line commands, schema or deployment policy.

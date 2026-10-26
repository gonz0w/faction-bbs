# Bounded device-local inbox

[Issue #219](https://github.com/gonz0w/faction-bbs/issues/219) adds selection and
filtering to the existing unlocked checkpoint console. It builds on the
[local read-state contract](local-mail-read-state.md); [member instructions](../docs/local-mail-inbox.md)
describe the keys and status labels. It adds no checkpoint schema or wire format.

The list model retains only original history indices and presentation status,
with at most 64 entries and six rows per page. Reversing local append order gives
the newest retained record first; server timestamps do not establish ordering or
read authority. Navigation clamps at boundaries. Filtering uses only received
UNREAD status; disabled tracking remains unknown rather than a synthetic zero.
After acknowledgement the list refreshes from the canonical handle and keeps the
selected original index where it remains eligible.

Received status comes from the backend's index-and-authenticated-frame lookup.
Sent status comes from operation-ID membership in the durable outbox; retirement
is a local decision. The model holds no body, subject, key, store or transport.
Preview text is borrowed from the checkpoint, rendered as inert scalars and
truncated by terminal cells into zeroized scratch. Only trusted fixed text supplies
navigation and selection markers. The existing decoder handles arrow sequences
and plain hotkeys; untrusted text cannot become input or terminal instructions.

Opening uses the original history index, renders that record only, and retains
the existing fresh post-output typed acknowledgement. Both index and received
frame identity reach the existing atomic mark-read API. Failed output, skipped
prompts, cancellation and sent records cannot acknowledge a different message.
The original H reader and its prompts remain available.

The shared local action works in offline native-mail and manual network-mail
consoles. It never opens a network connection, unlocks implicitly or alters archive
position, pending IDs, trust or keys. The existing parked Files boundary retains
SSH while the local console owns input/output. Lock/drop precedes its current
bounded Files access recheck. Ordinary approval, helper and transfer limits remain
unchanged; this adds no arbitrary board overlay or worker.

Verification distinguishes actual old-client absence, model red/green, real local
PTYs and owned NetBSD software execution. Earlier source/result hashes remain
historical observations. Source preservation, current required CI, separate review
and owned-fixture teardown follow [the project workflow](workflow.md).

[Source-bound software observations](evidence/mail-inbox/README.md) retain the actual
red/green candidates, final host/NetBSD controls and cleanup.

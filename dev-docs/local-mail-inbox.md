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

The common browsing action works in offline native-mail and manual network-mail
consoles. Browsing never opens a network connection, unlocks implicitly or alters
archive position, pending IDs, trust or keys. The existing parked Files boundary retains
SSH while the local console owns input/output. Lock/drop precedes its current
bounded Files access recheck. Ordinary approval, helper and transfer limits remain
unchanged; this adds no arbitrary board overlay or worker.

Verification distinguishes actual old-client absence, model red/green, real local
PTYs and owned NetBSD software execution. Earlier source/result hashes remain
historical observations. Source preservation, current required CI, separate review
and owned-fixture teardown follow [the project workflow](workflow.md).

[Source-bound software observations](evidence/mail-inbox/README.md) retain the actual
red/green candidates, final host/NetBSD controls and cleanup.

## Deliberate online reply

Issue [#233](https://github.com/gonz0w/faction-bbs/issues/233) adds a separate local
reply action to a selected received record in the manual network-mail console.
The offline reader does not expose online composition. Selection remains bound
to the original index and authenticated received-frame identity; a sent record
cannot become a reply source. Read acknowledgement remains an independent fresh
decision, never a consequence of reply, cancellation or queueing.

Reply eligibility uses the canonical healthy store, current prepared peer, exact
account/device match, revocation/expiry/recovery state and existing capacity limits.
The UI displays the current verified identity; historical messages do not prove
that identity still owns an earlier key. Context is checked before composition
and commit, with the existing atomic queue retaining final authority. No history
field imports trust, starts a session or selects another recipient.

The visible subject suggestion preserves an ASCII case-insensitive Re: prefix,
otherwise adds Re: and a space. Scalar-boundary truncation stays within128UTF-8
bytes and is disclosed. Enter accepts; fresh text replaces. The body starts empty.
All displayed values use inert classic/Unicode rendering and private scratch is
zeroized. Full recipient/content output must succeed before fresh queue
confirmation. Ordinary cancellation returns locally; control cancellation and
real I/O failure retain lock/drop semantics.

A confirmed reply uses the existing encrypted atomic outbox and changes no source
read decision, archive cursor, peer trust/identity or wire schema. Encryption
advances the existing ratchet normally. Queueing does not connect; pending is
not delivery. Backend refusal creates no partial reply and an ambiguous write
keeps the existing poisoned-store drop/reopen requirement. No plaintext sidecar,
quoted body, threading header, background credential or BBS metadata is added.

[Reply software observations](evidence/inbox-reply/README.md) preserve the executed
source stages, actual failures, host/NetBSD controls and compatibility limits.

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


## Retained correspondent navigation

Issue [#235](https://github.com/gonz0w/faction-bbs/issues/235) adds C inside the local
inbox in both offline and manual network-mail consoles. A derived index list
identifies the newest retained record for each exact account/device tuple. At
most 64 groups share the six-row, clamped navigation model. Peer strings remain
borrowed while grouping; only original indices survive in selection metadata.
No subject/body copy, persisted index, contact record or new checkpoint field is
introduced. Label scratch is zeroized and uses inert terminal-cell truncation;
the full device number appears before the account preview.

A selected group scopes the input entries before the existing inbox model applies
All/Unread. Original canonical indices survive filtering and refresh. Therefore
All cannot escape that group, an empty unread result cannot hop to another peer,
and explicit read decisions still bind the original received frame. Append-only
reply refresh retains the selected source where eligible. Nested Back returns to
the picker, then the original global inbox, then the mail menu.

Grouping is not threading, historical-key verification or peer enrollment. The
existing record viewer and online reply checks remain authoritative, including
current verified-identity review and mismatched/history-only refusal. Offline
inspection gains no Reply. Navigation changes no canonical ciphertext, cursor,
pending intent, read decision or trust, and performs no network work. Existing
private-console routing, locked return and normal approval/resource limits apply.

[Correspondent software observations](evidence/mail-correspondents/README.md)
retain executed variants, actual failures and host/NetBSD controls separately
from the earlier inbox and reply records.

## Local search

Issue [#242](https://github.com/gonz0w/faction-bbs/issues/242) adds S to the
unlocked offline and manual home-mail inboxes. A validated query is a literal,
case-sensitive Unicode substring of at most 64 UTF-8 bytes. Match each borrowed
peer account, subject and body independently; never combine fields or copy
bodies into an index. Blank Enter clears the query. Invalid or oversized input
is rejected without replacing the active query. The list uses a generic active
indicator, not the query, and reports an empty result truthfully.

Search intersects the current account/device group and All/Unread filter. The
model retains original canonical indices, newest-first order and six-row pages.
A new or cleared search selects the newest eligible record; normal refresh
retains the selected index where eligible. A nested group starts with an empty
query, Back restores the outer inbox query, and leaving its owning inbox drops
it. Query text and private scratch are transient and zeroized. Classic and
Unicode output remains inert and bounded to the existing 39-cell list width.

Search does not alter ciphertext, read state, pending intents, archive position
or trust; it does not unlock a store or contact home. Opening and explicit
acknowledgement still target the original record. Online reply retains its
existing authority checks; offline has no Reply. Control interruption,
inactivity and I/O failure retain the existing lock/drop behavior. No persistent
query, plaintext index, schema, dependency, crypto or wire change is introduced.

## Local conversation preview

Issue [#339](https://github.com/gonz0w/faction-bbs/issues/339) adds a deliberate
T action within an unlocked correspondent inbox in the offline and manual
home-mail consoles. The eligible records are the intersection of the exact
account/device group, current All/Unread filter and literal search. Original
canonical indices identify selections; local append order presents oldest first,
independent of sender UTC values. Empty results cannot escape the group.

Each page contains at most two previews with direction, status, UTC, subject and
at most three body rows per message. Each preview indicates continuation when
needed and offers the existing full-record route. Output fits 39 terminal cells per row
and a classic 24-row page, using inert classic/Unicode scalar rendering and
bounded zeroized text scratch. Navigation metadata contains no copied body or
key. Arrows/J/K select and N/P page with clamped boundaries. Enter/O opens the
selected original index in the existing reader; B returns to the scoped inbox.
After record return, refresh the same group, filter and query, preserving the
original selection where eligible and clamping otherwise.

Preview and navigation do not acknowledge, queue, send or perform network work,
or change archive/read state, ciphertext or trust. They add no schema, persisted
index, credential lease or background polling. Existing post-output typed read
decisions and current online peer/reply checks remain authoritative; offline has
no Reply. Existing lock/drop and parked-SSH return behavior remains in force,
and private text stays out of board streams, logs and diagnostics. Existing N
notice consent is unchanged. This retained-device presentation is not live chat,
protocol threading, current presence, a complete home inbox or key continuity.

## Multiline writing and paged bodies

Issues [#341](https://github.com/gonz0w/faction-bbs/issues/341) and
[#342](https://github.com/gonz0w/faction-bbs/issues/342) add a foreground private
writer and a bounded selected-record body view. W in manual home mail starts a
new message; W after an eligible received record starts a reply. Existing C/R
one-line paths remain available. The offline inbox supports selected-record
body paging but gains no compose or reply action.

The transient device-local body holds at most 1024 UTF-8 bytes, including LF
separators, and 32 logical lines. Both modes accept UTF-8 input; classic echo uses inert
escapes. Accepted text and discarded edits use bounded zeroized storage. Enter adds a line; .done, .preview, .undo, .clear, .cancel and
.help control the writer, and two leading dots enter one literal dot. Unknown
commands and oversized input do not append a partial body. Empty or whitespace-only
completion cannot queue. No plaintext draft is persisted.

Selected I-record bodies and W preview/review use body rows of at most 39 cells
including the trusted prefix, and pages of at most ten body rows. Short-line
payload fits 33 cells after the six-cell Body label; paged payload fits 37 cells
after the two-cell bar prefix. These bounds do not cover the complete metadata
layout. LF becomes a logical line break; other unsafe
scalars are escaped under the existing classic or explicit Unicode rules. Bounded
zeroized scratch and offsets support paging without copying bodies into navigation
metadata. A short single-line record keeps its direct post-output prompts. While
more pages remain, Enter advances and B returns without acknowledgement. In
.preview, B returns to the writer; during final review, B or a non-queue response
cancels the intent and discards the unsaved body. Only
complete body output permits the existing fresh typed read decision, bound to the
original index and authenticated frame. The H history loop retains its escaped
view. Group, filter, query and eligible original selection survive record return.

The current prepared verified peer and, for replies, the original received frame
remain authoritative before editing and at queue. Complete inert recipient,
subject and body review precedes fresh typed queue confirmation. Preview, paging,
cancellation and rejected input do not acknowledge or queue; neither writing nor
review connects to home or acquires credentials. Explicit send remains separate.
Control interruption, EOF, output failure and inactivity retain lock/drop and TTY
restoration behavior. This adds no persistent draft, schema, dependency, crypto,
protocol, notice, background lease or attachment action. Private text stays out
of board streams and logs; terminal scrollback is not erased.

## Foreground home exchange and scoped return

Issue [#344](https://github.com/gonz0w/faction-bbs/issues/344) adds an explicit
online-only X choice in the global/group inbox and conversation. Navigation
retains no password. The selected local entry carries the existing home settings;
no remote board frame or retained record grants that capability. The submenu
flushes queued navigation input before accepting N/A/B. Opening or canceling
has no network or checkpoint effect. Offline/native-mail views carry no exchange
capability and expose no exchange action.

N reuses Sync with its actual home-wide pending-send behavior; archive-enabled
Sync does not receive. A reuses a single CatchUp page when already enabled;
otherwise it explains the existing G prerequisite without upgrading or connecting.
The choice discloses those differences and does not chain actions, upgrade the
schema or enroll a recipient. The existing hidden password-per-cycle, deadline,
cancel/poison/drop and result paths stay authoritative. No new dependency,
transport behavior, background task or credential lease is introduced.

After a completed or recoverably failed action, recompute eligible canonical
indices and literal search matches from the actual vault, retaining the exact
account/device group, filter and query. Keep the original selected index when
eligible and otherwise clamp within the same scope. Rebuild even on recoverable
archive failure because earlier entries may already be saved. Conversation and
list navigation never acknowledge these new entries or retire sent operations.
The home-wide report is not a scoped conversation delivery receipt. Private
content and local controls remain outside the parked board stream.

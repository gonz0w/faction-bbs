# Select retained encrypted mail

The optional native mail client has an **I: Local inbox** action in both its
unlocked offline checkpoint console and its manual home-mail console. Open home
mail from the local picker or [parked Files desk](local-unread-mail.md), or use
the [offline checkpoint console](checkpoint-console.md). Unlock remains local.
Ordinary SSH and the default client build do not decrypt member mail.

The inbox lists retained records newest first, in local retention order. It shows
six short entries per page, a `>` selection marker, the original record number and
an inert subject preview. Long previews end in `~`; controls remain visible text.
Classic display escapes Unicode; the explicit Unicode override retains safe
Unicode. Each list line fits within 39 terminal cells.

| Status | Meaning on this device |
| --- | --- |
| UNREAD | Received and retained; no explicit local acknowledgement. |
| READ | Received and explicitly acknowledged here. |
| UNKNOWN | Local read tracking is disabled. |
| PENDING | Sent ciphertext is still in the durable local outbox. |
| RETIRED | Sent ciphertext was retired by a local decision. |

Pending and retired states prove neither remote delivery nor reading. These are
retained-device records, not a complete or current home-server inbox.

## Find and open a message

- **Up/Down** or **K/J** moves the selection. Movement stops at either end.
- **N/P** moves to the next or previous page.
- **U** lists unread received messages; **A** returns to all retained records.
  If tracking is disabled, U reports unknown and keeps the all-records view.
- **S** searches the current inbox. See [Search retained mail](#search-retained-mail).
- **Enter/O** opens only the selected record, with peer, device, UTC time,
  subject and body. An empty list has nothing to open.
- **B** returns from the list to the local mail menu.

Browsing, filtering and opening do not mark mail read. Long or multiline bodies
open in pages of up to ten rows. When more pages remain, press Enter for the next
page or B to return without acknowledging. After the complete received body is
shown, enabled tracking offers the existing prompt to type `read`. Only that
fresh decision acknowledges the selected record; Enter leaves it alone. Then
Enter returns to the inbox with updated status. An acknowledged record leaves
an unread-only list. Sent records have no read-acknowledgement prompt.

Enable [local read tracking](local-unread-mail.md) deliberately with E at the mail
menu before using read decisions. Merely opening the inbox does not upgrade an
older encrypted checkpoint. The **I** selected-record reader supports body paging
offline as well as online. **H** retains its sequential history reader and escaped
view.

## Search retained mail

In an unlocked inbox, press **S**, enter text and press Enter. Your input is
hidden as you type. Search finds a
literal, case-sensitive Unicode substring in the peer account, subject or body
of each retained record. The text must match within one field; it cannot span
fields. The query can use up to 64 UTF-8 bytes. Invalid or longer input is
rejected and leaves the previous search active. Press S and then Enter with an
empty prompt to clear it.

Search works with the current All or Unread filter and, inside a correspondent
inbox, only that account and device. Results remain newest first, with the
original record numbers and six entries per page. A new or cleared search
selects the newest eligible record. The list shows a generic search indicator,
not the query. If no records match, the list is empty.

A newly opened correspondent inbox starts without a search. Returning through
the picker restores the original inbox's search; leaving an inbox discards its
search. Search uses the unlocked records on this device, performs no home
connection and saves no query or plaintext index. Opening, acknowledging and
replying retain their existing rules; the offline inbox has no Reply action.

## Browse one correspondent

From the inbox, choose **C: Correspondents**. The picker groups retained sent and
received records by their exact stored peer account and device. The same account
on two devices appears separately. Groups with the newest retained activity come
first; this is your device snapshot, not live presence or a complete home inbox.
A group does not verify a contact or establish continuity with an older key.

Use arrows or J/K to select, N/P to page, and Enter/O to open. Each page has up to
six entries. The complete device number precedes an inert account preview; `~`
indicates shortened text. Open a record to inspect its existing peer details.

Inside a correspondent inbox, **U** shows that group's unread received messages;
**A** shows all its retained records. Neither key switches to another correspondent.
Disabled read tracking still shows unknown. Acknowledging the last unread record
leaves an empty unread view for that group. Browsing and filtering acknowledge
nothing and do not connect to your home.

**B** returns from the grouped inbox to the correspondent picker. Another B
returns to the original inbox, and B there returns to the local mail menu.
The picker also works offline. Existing explicit read decisions remain available;
Reply is still limited to the manual home-mail console and its current verified
peer checks. Grouping never enrolls or retargets a recipient.

## Preview a conversation

Inside a correspondent inbox, press **T: Conversation** to preview the current
account and device. The preview uses the current All/Unread filter and search;
it shows only matching retained records, in local append order, oldest first.
Sender timestamps do not determine this order. Each page shows up to two messages,
with direction, local status, UTC time, subject and up to three body rows. A
truncation indicator shows when text continues; open the full record to read it.
Classic and Unicode displays render message text inertly.

Use **Up/Down** or **K/J** to select a message and **N/P** to change pages;
movement stops at the ends. **Enter/O** opens the selected full record, where
existing explicit read and eligible online reply choices apply. On return, the
same correspondent, filter and search remain active, with the original selection
kept if still eligible. **B** returns to that correspondent inbox. An empty
result cannot select a different correspondent.

In an online conversation matching the current independently verified peer, **W**
opens a fresh multiline message to that peer. Review the recipient and complete
content and confirm **queue** to save encrypted ciphertext locally. Composing
does not contact home or acknowledge mail. Cancellation or queueing returns to
the same group, filter, search and eligible selection; a new sent record may be
hidden by the current filter or search. See the
[multiline writer](online-mail.md#write-a-multiline-message) for limits and commands.
Offline, old history or a different or no-longer-current peer cannot authorize
writing. Full, cold, expired, revoked and history-only states retain their existing
refusals.

Previewing alone does not acknowledge, reply, queue or send mail, or contact home.
It is a view of retained records on this device, not live chat, protocol threading,
current presence or proof of contact or key continuity. The offline reader has no
Reply action. Existing local notice settings are unchanged.

## Reply from the manual home-mail console

After opening a received record in the manual home-mail console, press Enter to
skip the read-acknowledgement prompt if tracking is enabled, then choose **R** to
prepare a one-line local reply, or **W** to write a multiline reply. Review the
current verified recipient, device and full
identity. The received account/device must match that already prepared peer;
history never enrolls a recipient or proves continuity with a changed key.
Sent, unprepared, revoked, expired or history-only state cannot authorize a reply.

The suggested subject keeps an existing `Re:` prefix or adds one. It stays within
128 UTF-8 bytes, with a notice if shortened. Press Enter to keep it, or enter a
replacement. The one-line body starts empty and retains its 1024-byte limit. For the multiline
writer, see [Write a multiline message](online-mail.md#write-a-multiline-message).
Review the recipient and complete content, then type a fresh **queue** confirmation
to persist encrypted ciphertext. Enter or other confirmation text cancels and
returns to the inbox. Queued confirmation input is discarded.

Reply does not mark the source read, connect to your home or ask for a home
password. Sending still requires a separate deliberate N action or an eligible
brief E exchange. A pending reply proves neither delivery nor reading. Normal
cancel returns to the same inbox; Ctrl-C/D, input/output failure and inactivity
retain their locking behavior.
Failed writes retain the canonical-store close/reopen rule.

Reply is available only in the manual network-mail console, including its picker,
Files and main-menu routes. The offline inbox and ordinary SSH keep their existing
behavior. No reply body, subject, count or key is sent to the BBS, and no plaintext
reply draft or index is saved.

## Exchange home mail without leaving your place

In the manual online console, **X: Home exchange** is available in the global
inbox, a correspondent inbox and its conversation preview. It opens a local
choice; entering it or choosing **B** does not connect or change the vault.
The offline console has no Home exchange action.

In an eligible online conversation, **E: Brief exchange** also offers a separate
receive or one-text send. Its **Q** choice selects queued ordinary text even when
Unread or search hides sent records. S and Q both review the complete selected
subject, then page the complete body before fresh confirmation. B in the queued
picker returns to the exchange menu; B during body review cancels the send and
returns to the conversation. After E, **R: Review newly received
text** appears when actual new ordinary text from the current verified peer,
device and identity was retained, including text hidden by search. Enter/O opens
the full record, which offers a fresh read confirmation after the complete body
if local read tracking is enabled. Press **W** in the arrivals list to open the
local multiline reply writer. Returning keeps the original filter, search and
eligible selection. See the
[brief exchange guide](online-mail.md#exchange-from-an-inbox-or-conversation) for
selection, confirmation and limits.
Classic and Unicode production host TLS/PTY checks cover the queued-body and
arrival-reader reviews from #363 and #364. Source-matched NetBSD software VM
evidence covers those reviews and same-SSH/SFTP return at 1 GiB runtime. The
[technical contract](../dev-docs/native-home-mail.md#bounded-foreground-contract)
describes the classic evidence qualification. See
[#363](https://github.com/gonz0w/faction-bbs/issues/363) and
[#364](https://github.com/gonz0w/faction-bbs/issues/364).

Choose **N** for the existing home-wide Send pending / Connect-sync action.
It may send **all pending ciphertext**, regardless of the correspondent or search
you are viewing. With archive checkpoints enabled, N only sends; without them,
it also briefly receives. Choose **A** to fetch one archive page of at most four
entries. If archive checkpoints are disabled, A explains that they must first
be enabled with G at the mail menu; it does not upgrade or connect. A does not
send pending mail. Neither choice automatically runs the other.

If **A** saves new ordinary text, **R: Review newly received text** opens the
arrivals list, including messages hidden by your search. Press **W** to reply to
the selected text. Confirming queues an encrypted reply locally and returns to
the list. Press **B** in the list, then **B** or Enter at the result to return to
your view.

Each connection asks for a fresh hidden home account password. Press Enter with
the password empty to cancel and return without connecting. On completion or a
recoverable failure, the same local view returns with its correspondent,
All/Unread filter, search and eligible selected record kept. Records are rebuilt
from the actual vault, so newly saved messages appear where they match. A failed
archive cycle may have saved earlier entries; it does not imply an unchanged vault.
An unavailable selected record clamps within the same view.

The exchange actions do not themselves acknowledge messages or retire pending
ciphertext. A send write is not proof of delivery or reading. The result covers
the home exchange,
not only the displayed correspondent. Ctrl-C/D and terminal or store failures
retain the existing locking rules. No background polling or password retention
is added; the board receives no private mail or local counts.

## Lock and return

Use B to return to the mail menu and **L** to lock. From parked Files, this returns
to the same SSH visit if its bounded access check still succeeds. Ctrl-C/D, input
failure or the existing inactivity timeout lock the console. A failed checkpoint
write keeps the existing poison/drop/reopen rule; reopen the canonical store
before trying again.

Ordinary inbox navigation and composing do not connect; only a deliberate
exchange action contacts home. There is no background refresh. Keys, message
text and read decisions stay on the member device. It creates no plaintext index
or sidecar, sends no receipts and retains no new background credentials. A local
view can remain usable during a home outage. Terminal scrollback is not erased.

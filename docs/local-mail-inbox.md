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

Browsing, filtering and opening do not mark mail read. After a received body is
shown, enabled tracking offers the existing prompt to type `read`. Only that
fresh decision acknowledges the selected record; Enter leaves it alone. Then
Enter returns to the inbox with updated status. An acknowledged record leaves
an unread-only list. Sent records have no read-acknowledgement prompt.

Enable [local read tracking](local-unread-mail.md) deliberately with E at the mail
menu before using read decisions. Merely opening the inbox does not upgrade an
older encrypted checkpoint. **H** retains its sequential history reader.

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

## Reply from the manual home-mail console

After opening a received record in the manual home-mail console, press Enter to
skip the read-acknowledgement prompt if tracking is enabled, then choose **R** to
prepare a local reply. Review the current verified recipient, device and full
identity. The received account/device must match that already prepared peer;
history never enrolls a recipient or proves continuity with a changed key.
Sent, unprepared, revoked, expired or history-only state cannot authorize a reply.

The suggested subject keeps an existing `Re:` prefix or adds one. It stays within
128 UTF-8 bytes, with a notice if shortened. Press Enter to keep it, or enter a
replacement. The one-line body starts empty and retains its 1024-byte limit.
Review the recipient and complete content, then type a fresh **queue** confirmation
to persist encrypted ciphertext. Enter or other confirmation text cancels and
returns to the inbox. Queued confirmation input is discarded.

Reply does not mark the source read, connect to your home or ask for a home
password. Sending still requires a separate deliberate N action. A pending reply
proves neither delivery nor reading. Normal cancel returns to the same inbox;
Ctrl-C/D, input/output failure and inactivity retain their locking behavior.
Failed writes retain the canonical-store close/reopen rule.

Reply is available only in the manual network-mail console, including its picker,
Files and main-menu routes. The offline inbox and ordinary SSH keep their existing
behavior. No reply body, subject, count or key is sent to the BBS, and no plaintext
reply draft or index is saved.

## Lock and return

Use B to return to the mail menu and **L** to lock. From parked Files, this returns
to the same SSH visit if its bounded access check still succeeds. Ctrl-C/D, input
failure or the existing inactivity timeout lock the console. A failed checkpoint
write keeps the existing poison/drop/reopen rule; reopen the canonical store
before trying again.

Inbox navigation performs no home connection or background refresh. Keys, message
text and read decisions stay on the member device. It creates no plaintext index
or sidecar, sends no receipts and retains no new background credentials. A local
view can remain usable during a home outage. Terminal scrollback is not erased.

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

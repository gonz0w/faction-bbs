# Local unread mail and a parked board visit

The optional native mail client can remember which retained received messages you
explicitly acknowledged on this device. This state is encrypted with your existing
checkpoint. It sends no read receipt to a home server or peer.

Build and configure [manual encrypted home mail](online-mail.md), then add the
private [picker home-mail paths](picker-home-mail.md). The default build and
ordinary SSH do not decrypt member mail.

A separate optional [main-menu mail entry](main-menu-mail.md) opens the same local
console without entering Files. The Files workflow below remains available.

## Enable local read tracking

Open home mail and unlock your checkpoint. Its **E: Enable local read tracking**
action asks you to type `enable`. Keep a [history-only backup](checkpoint-console.md)
before upgrading. Existing received history starts unread because the client has
no reliable record that you read it. Sent messages and your known sent echoes do
not count as unread.

This deliberate transition writes the encrypted inner `FACTCP04` format. Older
clients cannot open it. Merely unlocking an older checkpoint leaves its format
unchanged; `FACTCP02`, `FACTCP03` and the v1 vault remain separate supported paths.
Tracking can be enabled independently of archive mode. It does not change keys,
peer verification, pending ciphertext or the archive position.

**I: Local inbox** offers [selection, pages and unread filtering](local-mail-inbox.md).
**H: History** shows each retained record. After a received body has been output,
type `read` at its local acknowledgement prompt to mark that one message read.
Enter leaves its current state alone. Cancellation locks without acknowledging
the current message. Listing counts, viewing a body, sending mail or fetching an
archive page does not acknowledge received mail automatically.

Read decisions survive lock, restart, encrypted backup and history-only recovery.
They remain local: they do not retire pending sends, prove delivery, synchronize
other devices or tell a peer you read something. A duplicate does not reset an
existing decision. New authenticated received messages start unread. History stays
bounded at 64 records with the existing explicit full-store stop.

## Open mail while a board stays connected

Connect through the private board catalog and enter the BBS's **Files** screen.
After the native Files handshake completes, the local menu offers:

- **E: Encrypted home mail** asks for a fresh local confirmation and checkpoint
  unlock. The existing SSH visit stays parked while the member-device console runs.
- **N: Local mail notices** enables generic local status for this visit. It starts
  off. Enabling it does not unlock a checkpoint or contact your home.

Inside home mail, **E** means enable read tracking; in the Files desk and board
picker, **E** means open home mail. Each screen labels its action.

**L** locks mail and returns to the local Files desk. Cancellation also locks.
A failed local operation may ask for Enter before returning. The client clears the
visible viewport and discards queued input at the transition; it does not erase
terminal scrollback. Unlock is required each time you enter mail.

While the private view is open, the client does not render board output or forward
mail keyboard bytes to SSH. The existing PTY supplies bounded backpressure. A peer
can disconnect while parked; this is not an indefinite pause guarantee. A bounded,
authenticated Files request checks the board before unlock and again after lock.
A lost or rejected connection is reported rather than automatically reconnected.
Normal local approval, file permissions and transfer grants still apply.

This entry exists only between completed Files operations. It does not intercept
a game, room draft, transfer, form or probe, or create a mail overlay over arbitrary
remote output. A catalog without home-mail paths and a direct `--profile` visit
retain the existing Files menu.

## What a notice means

With notices enabled, a graceful mail lock can leave a counts-only snapshot at the
local Files desk: unread, retained history and pending ciphertext. It is labelled
**LOCKED** and **stale local snapshot**. It contains no sender, subject or body.
A locked checkpoint with no snapshot says **unread unknown**; disabled tracking
also remains unknown, not zero. Opting out clears the snapshot. Notices reset off
for each new board visit.

These are retained-device counts, not a complete or fresh home-inbox total. No
background timer, polling, bell, desktop notification or shared BBS badge is added.
To fetch retained mail, explicitly unlock and use the existing bounded manual
archive action. Broader unread push, live private chat, roaming and encrypted
attachments remain separate capabilities.

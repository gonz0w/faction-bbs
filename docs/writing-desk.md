# Room writing desk

Status: optional full-screen/line room writer and explicit saved checkpoints;
software preview, no supported release yet. This is for **local discussion-room
posts**, separate from private mail.

Choose Post from a room card or main menu, select a reply and author a subject
(blank keeps its generated/reply default). At the one-line Message prompt, enter
`/edit`. The one-line shortcut still sends immediately. Every writing action
checks current local approval and room/reply context; a denied draft is never
redirected to another room.

## Line editing and preview

Enter adds a line, including blank paragraphs. Backspace edits the current line.
Exact lowercase dot commands work on ordinary SSH and plain terminals:

| Input | Effect |
| --- | --- |
| `.screen` | Full-screen editing when ANSI and at least 40x18 are available. |
| `.line` | Return to the classic desk with the body intact. |
| `.preview` | Page through the inert room presentation; nothing sent or marked read. |
| `.send` | Explicitly post; a busy database offers the same-submission retry. |
| `.undo` / `.clear` | Remove the last completed line / clear the body. |
| `.quote` | Append the freshly authorized reply parent's body with `> ` prefixes. |
| `.save` | Save a plaintext checkpoint on this board, readable by its sysop. |
| `.leave` | Close writing; keep the last saved checkpoint, discard unsaved edits. |
| `.cancel` | Discard writing and delete this saved revision, if any. |
| `.help` / `..text` | Show help / add a literal line starting with `.text`. |

The complete body stays within **1024 UTF-8 bytes**, including newlines, and
**32 lines**. A rejected line/quote keeps existing text intact. Whole quotes must
fit; otherwise type a shorter excerpt. Preview, quoting, resume and save never
mark a message read. Already copied quotations are ordinary local plaintext;
permission changes cannot retract text a member has already seen.

## Full-screen editing

Arrows move the caret across scalar boundaries and displayed rows. Enter splits a
paragraph; Backspace joins across a newline. Ctrl-D deletes the next scalar,
Ctrl-A/Ctrl-E move to logical line start/end, Ctrl-K deletes to line end, and
Ctrl-U removes the current logical line. The body viewport scrolls with the
caret; Unicode cell widths follow the selected Terminal settings. Classic mode
escapes non-ASCII/invisible formatting rather than assuming a font.

| Key | Action |
| --- | --- |
| Ctrl-P | Preview; return to editing afterwards. |
| Ctrl-G | Quote the selected reply. |
| Ctrl-S | Explicitly save a board checkpoint. |
| Ctrl-T | Ask for `SEND` confirmation before posting. |
| Ctrl-Q | Return to line editing. |
| Ctrl-C | Cancel and remove this saved revision. |

Fitting resizes preserve the body, caret and partial-key decoding. A too-small
terminal or unsupported input returns to line editing with an **Enter recovery
barrier** so delayed key fragments cannot become draft text. Shared line-input
resize keeps completed lines and clears unfinished input; local line input keeps
its current line until completion. One-line posts and plain terminals remain
available. Sysops may set `writing_title` (up to 24 printable ASCII bytes) in a
base/seasonal theme; the existing palette, board name, subject and room destination
remain visible. Themes cannot supply executable editing behavior or local paths.

## Saved room checkpoints

Save is deliberate; **there is no automatic keystroke persistence**. Up to four
rooms per member have one checkpoint each on that local board. Returning to Post
in the same room offers **Resume**, **Delete and start new**, or **Quit** before
asking new reply/subject questions. Resume restores the saved subject, reply and
body. Subsequent edits need Save again. Disconnect, timeout or a process failure
retains only the last successful checkpoint; unsaved changes may be lost.

A checkpoint is **plaintext, sysop-readable board data**, not end-to-end encrypted
mail. Scope is the local board/member/approval epoch/room/reply. Save and delete
check the current saved revision: another connection's newer checkpoint cannot be
silently overwritten or removed. An older editor may still explicitly post its own
version; that cannot delete a newer saved revision. Send uses the checkpoint's
request ID and removes that matching checkpoint in the **same transaction** as
the post. Lost acknowledgments cannot leave that revision available for a duplicate
resume/send. Pending/revoked members cannot recover/send; status changes and local
sysop password recovery clear affected checkpoints. A missing reply is refused;
the checkpoint can still be deliberately deleted instead of redirected.

Recovery expires after 30 days by the board's clock; per-member reads/saves lazily
remove expired rows. Expiry/deletion is logical, **not secure erasure**: inactive
rows, WAL/journals and old backups may retain bytes. Each save replaces its prior
revision, not a retained edit history. No remote board synchronization or arbitrary
file attachment/path is implied.

## Upgrade, export and recovery

Schema **8** adds room checkpoints. Stop/drain the owner and transfers and verify
a complete private DB/blob/config/key backup before the first new owner open.
Old schema-2..7 exports remain read-only and verifiable; restored/opened older
boards upgrade through the existing migrations without inventing drafts. New
owner exports and complete file-area bundles include checkpoints as private board
data. Restore retains checkpoint contents under restored membership, while rotating
old session authority as before. Review restored approvals/passwords. Roll back
using matching old binaries and the complete older backup in fresh paths; never
relabel `user_version`, downgrade in place or unlink live locks.

The selected server checkpoint supports ordinary SSH. A native-only alternative
would keep room drafts on member devices but needs its own scoped format, local
ownership/unlock, quota/retention and deliberate recovery/import design. It remains
possible later. **Every private-mail draft, key and decrypted message stays on
member devices under #90/#99**; this room feature adds no server mail plaintext or
vault unlock. Member-scoped draft export, autosave and richer editor features
remain follow-ons; owner backup/export is the current recovery path.

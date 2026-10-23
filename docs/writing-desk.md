# Room writing desk

Status: optional first writing-desk slice for #141; no supported release yet.

Choose Post from a room card or the main menu, select a reply if wanted and enter
an authored subject (blank keeps the generated/reply default). At the ordinary
one-line Message prompt, enter `/edit` to open the multiline desk. The existing
one-line shortcut still sends immediately. The desk is for **local room posts**;
it is separate from private mail.

Each Enter adds a line, including blank lines between paragraphs. The desk shows
its room destination and uses the sysop's existing section palette/board identity.
Current approval and the selected room/reply are checked on each action. It never
moves a denied draft to another room.

| Input | Effect |
| --- | --- |
| `.preview` | Page through the draft with the ordinary inert room formatting. Nothing is sent or marked read. |
| `.send` | Submit explicitly; a busy database offers the existing same-submission retry. |
| `.undo` | Remove the last completed line. |
| `.clear` | Remove all draft lines. |
| `.quote` | Append the current reply parent's permission-filtered body with `> ` prefixes. Select a reply before entering the desk. |
| `.cancel` | Discard the draft and return through the normal menu pause. |
| `.help` | Show the commands. |
| `..text` | Add a literal line starting with `.text`. |

Commands are exact lowercase lines. Unknown dot commands report an error without
altering the draft; they never send. Quotes are all-or-nothing: if the complete
quote does not fit, type a shorter excerpt. A quote does not mark the parent read.
An already viewed/copied room quotation is ordinary local plaintext, like text
a member typed; later file-permission changes cannot retract that prose.

The complete body remains limited to **1024 UTF-8 bytes**, including newline
separators, and at most **32 lines**. A rejected additional line leaves completed
lines intact. Backspace edits the current line with whole-scalar erase; `.undo`
changes completed lines. Preview uses the current terminal geometry and selected
Unicode/classic rendering. Controls and invisible formatting are inert. Native
Unicode remains opt-in under Terminal settings. Plain terminals need no ANSI.

Shared-service resize keeps completed draft lines, clears unfinished input and
returns to the writing prompt; restart preview with `.preview`. Local/worker
sessions keep their current input until the line completes and calculate geometry
again for each preview page. Disconnect/timeout/invalid input/revocation discards
the connection's draft. **There is no saved-draft or reconnect recovery yet.**
The current desk is a line-oriented step toward #141; full-screen caret movement,
paragraph editing, configurable desk labels and recoverable drafts remain open.

## Recoverable-draft design gate

No draft persistence is adopted by this slice. The follow-up must compare:

| Candidate | Benefit | Required decisions/evidence |
| --- | --- | --- |
| Native member-side room drafts | Device ownership; works across an interrupted visit without adding board storage. | Explicit board/member/room binding, local unlock/path ownership, quota/retention, delete/export/recovery, changed destination/approval checks and deliberate resume confirmation. Ordinary SSH still needs a usable fallback. |
| Bounded server-side room drafts | Reconnect recovery through ordinary SSH. | Room-post plaintext readable by the sysop, explicit opt-in, per-member quotas/retention, epoch/approval/room checks, stopped backup/export/deletion semantics and clear discard/recovery UI. No hidden indefinite history. |

Neither option may hold private-mail plaintext or member decryption keys on a
server. All private-mail drafts remain device-side under #90/#99. A remote frame
or theme cannot choose a local path, invoke an editor or unlock a keystore.
Keep this choice separate from the working in-memory room desk. No new schema,
external service, listener, dependency or capacity promise follows.

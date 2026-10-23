# BBS-local command chat

Updated 2026-10-20. Implemented in the local console and [restricted SSH member worker](restricted-ssh.md);
there is no IRC wire server or federation. [Build/setup](local-development.md) and
[ANSI/theme options](theme-packs.md) still apply. This owner-capable executable
must never be exposed as an SSH forced command or member OS shell.

Log in with an approved account and choose `[C] Local chat`. For simultaneous
testing, open another terminal on the same private state directory and use a
second approved account. These are operator-run local consoles, not isolated OS
accounts for untrusted members. Each chat entry automatically joins `#lobby`.

```sh
./target/release/faction --state-dir "$HOME/.faction-dev" login --ansi --theme themes/base
```

In `--ansi` on a capable terminal, new messages appear while you type and the
draft is redrawn in a bounded one-row viewport. Enter submits; Backspace edits
the tail. Arrows/control sequences are consumed rather than inserted into text;
menu hotkeys are not active in chat. The live editor accepts ASCII, at most 768
bytes; overflow discards the whole line. Bodies are limited to 512 UTF-8 bytes
in the core, with controls/newlines rejected. Plain cooked input can carry UTF-8;
all received non-ASCII/control text is displayed with escapes. Full Unicode/caret
editing remains work.

Plain or unsupported terminals use Enter-driven commands. `/poll` or a blank line
reads new messages; they are not printed asynchronously over a cooked-input draft.
Both modes heartbeat their presence while idle. Ctrl-C/D leaves live chat; `/quit`
leaves either mode. Normal exit restores terminal state and removes chat presence.
A live resize outside supported geometry closes chat and displays, without sending,
the unsent draft; it does not move that draft to another channel. Fatal process
termination may still require local `stty sane` recovery.

## Commands

| Input | Behavior |
| --- | --- |
| `/help` | Show the implemented command vocabulary. |
| `/list` | List board channels and private channels you may access, all explicitly local. |
| `/join #channel` | Join/select a channel; case-insensitive ASCII name, `#` plus 1–31 letters/digits/underscore/hyphen. |
| `/part` | Leave the selected channel. Other joins remain until parted or chat exit. |
| `/names [after-handle]` | List up to 32 distinct joined, approved, live account handles; use the returned cursor for more. |
| Ordinary text | Send to the selected channel. No selection -> no message is sent. |
| `/me text` | Send a simple emote using your account handle. |
| `//text` | Send literal `/text`, without treating it as a command. |
| `/poll` or blank | Read the next bounded page of new messages. |
| `/history [after-sequence]` | Replay eligible retained history, with an explicit next-page cursor when needed. |
| `/quit` | Remove this chat entry's joins/presence and return to the BBS menu. |

Handles are local account identities; no `/nick`, direct messages, IRC handshake,
numeric replies, existing IRC-client compatibility or remote server connections.
External IRC-client interoperability remains an open product question in #26;
these slash commands do not resolve it by implementing a wire protocol.

## Channels, grants and moderation

All approved members can join board channels. Private channels are hidden from
outsiders and require a sysop grant; the sysop has operator access to every local
channel. Private does not mean encrypted or secret from the filesystem owner.
This increment has sysop/member authority, without channel-moderator roles yet.

| Sysop command | Behavior |
| --- | --- |
| `/create #channel board` | Create an approved-member channel. |
| `/create #channel private` | Create a grant-controlled private channel. |
| `/allow #channel handle` | Grant an already-approved member access to a private channel. |
| `/deny #channel handle` | Remove its private grant and all current joins to that channel. |
| `/kick #channel handle` | Part all of that member's current joins; it can rejoin if still allowed. A kick is not a ban. |

Every grant update, including a repeated allow, forces a new join. Deny/regrant
never silently revives old joins; rejoin starts a new history floor. Global account
revocation invalidates the chat lease and future room/chat actions. Reapproval
requires fresh authentication. The next checked action enforces the change;
already-authorized output may finish. When an action is denied, the console clears
its selected channel/draft and requires an explicit join before sending again.
It never redirects a private draft to `#lobby`.

Every channel is local. No shared-channel toggle, network subscription or outbound
adapter exists. The original local-chat increment keeps shared-channel federation explicit and
separate from local accounts, room data and private channels. Private chat grants
do not add private-room ACLs to the message boards.

## History, leases and bounds

The database retains the latest 64 messages per channel, not a permanent archive.
Retention is count-based: an idle channel's retained messages can remain at rest
indefinitely. Reads only return messages after this chat entry's join floor, so
new grants/rejoins/reconnects cannot retrieve earlier private history through the
API. Selecting an already-joined channel goes to its current tail; `/history`
can replay eligible retained messages since the original join. Slow readers get
an explicit history-gap notice when older messages have been removed.

| Resource | Initial cap |
| --- | --- |
| Channels | 32, including `#lobby`; no channel delete/rename/conversion command yet. |
| Live chat entries | 256 leases, not a supported member-capacity claim. |
| Joins per chat entry | 8. |
| Retained messages | 64/channel, bodies <=512 UTF-8 bytes. |
| Read page | <=16 rows and 2 KiB charged body/author/metadata; escaped TTY bytes can be larger. |
| Presence page | 32 distinct account handles with a cursor. |
| Retry receipt | One latest submission per lease; no unbounded deduplication log. |

A chat lease expires after 60 seconds without heartbeat; consoles heartbeat every
10 seconds. Expired/revoked entries disappear from roster queries, and their rows
are reclaimed on the next chat entry. There is no background daemon. System clock
changes can invalidate a lease; timestamps more than five seconds in the future
fail closed rather than reviving indefinitely after rollback.

The core assigns channel sequences and checks current approval/role/grant/join in
each transaction. Submission numbers increase per lease. An exact retry of its
latest request in the same join context returns the original sequence; changed,
older or skipped requests fail. On a failed/uncertain send, retry the exact text;
the console keeps its submission number until success. No draft/retry recovery
after process exit is promised.

There is no per-recipient queued message copy: local consoles pull bounded pages
from the shared channel history, roughly every 250 ms in live mode. SQLite writes
remain serialized; active history reads are read-only. TTY writes occur after the
database transaction has ended. This is a local adapter, not a measured 250-member
network server or a substitute for future SSH output timeouts/backpressure.

History and latest retry bodies are plaintext application data in the private
SQLite store, not telemetry or operational body logs. Row retention/expiry does
not guarantee secure deletion from SQLite pages, WAL, snapshots or backups. Total
database size, account/grant quotas, retention administration and full server
accounting remain separate work. [Private snapshots](export-restore.md) include retained
chat and grants, remove live leases/joins/receipts and require fresh joins after restore.

## Schema upgrade and checks

This increment transactionally migrates schema 1 to 2 on open, adding chat tables
without replacing board identity, account hashes, room messages or unread cursors.
Before upgrading, stop every old console and make a complete private copy of the
state directory. Older binaries reject schema 2. There is no downgrade migration;
rollback requires the pre-upgrade copy, not editing the version pragma. Keep test
data until broader lifecycle/deployment gates are met; schema-2
[export and restore](export-restore.md) is now available for local recovery rehearsal.

```sh
cargo test --workspace --locked --release
cargo build --workspace --locked --release
python3 tools/smoke_local_console.py target/release/faction
python3 tools/smoke_local_chat.py target/release/faction
```

The chat smoke uses simultaneous real PTYs and generated disposable credentials;
it checks draft preservation, private grants/history floors, deny/regrant/kick,
revocation, control-tail isolation, resize/plain fallback and presence cleanup.
It prints neither credentials nor transcripts. See [ADR 0010](../dev-docs/decisions/0010-bounded-local-chat.md)
for tradeoffs, sources and unperformed native NetBSD/server validation.

The [modern communications stories](modern-sysop-and-communications.md) now make
actual IRC/IRCv3/server linking a specific comparison (#100), with native XMPP
private chat/mail/attachments UX (#99/#90/#88). Current slash commands still do
not provide an IRC wire server. #101 separately researches explicitly shared
persistent rooms; local-only channels and existing rooms are never auto-relayed.

## Optional social lounge

The [opt-in social lounge](social-lounge.md) adds availability and bodyless polite
page/accept/decline signals across live local chat channels. Sysops enable/name it
with /social; members opt in for each chat entry and explicitly check /pages.
Mute, blocks and UTC quiet-hour intent persist. This is separate from existing
channel /names: /hide removes lounge advertising, not explicit channel membership.
No custom private-message body, forced join, automatic page popup or federation.
/notify on|off controls optional idle-menu indications for this visit; it never
adds page output over chat input.

In the primary shared service, a connection's explicit social availability now
survives leaving chat. Exit still parts every channel and clears its send context;
re-entry gets a new join/history floor. A separate main-menu Lounge action offers
current bodyless pages without joining chat. See [the scope and safe notifications](social-lounge.md).
Standalone local login and inherited workers now use the same visit scope,
including classic/plain menus. Their synchronous prompts and full-screen editor
silently heartbeat the existing lease; clean exit closes it.

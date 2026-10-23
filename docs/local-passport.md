# Your local board passport

Status: opt-in native-client notebook preview; validated on macOS and NetBSD 11
ARM64 software VM. No supported release or verified-board identity claim.

Keep a private note about a board, favorite-room reminders and an original ASCII
calling card on your own device. Nothing is collected automatically when you
connect, open a board or use the notebook. There are no visit timestamps, travel
notifications, public passport profiles or cross-board presence.

**Notes are plaintext on your device**, protected by private Unix permissions.
This is separate from the encrypted private-mail vault. Someone who controls your
OS account or backups can read the notebook. Use it for board notes, not keys or
passwords. It sends no data to any board.

## Enable and open

Create a new private store explicitly, with existing private parent folders:

```sh
faction-client --passport-store /home/member/.faction-passport init
```

Add its absolute path to your existing private Boards catalog **before** the
`[[boards]]` entries:

```toml
version = 1
passport_store = "/home/member/.faction-passport"

[[boards]]
id = "nightwire"
label = "Nightwire"
profile = "/home/member/.config/faction/nightwire.toml"
```

The [local Boards picker](board-launcher.md) shows **J:passport**. Highlight a board
and press J between visits. Q or Ctrl-C returns with the selection retained.
Opening an uninitialized store reports that it is unavailable; it does not create
one. Ordinary SSH/plain BBS sessions remain available and gain no passport state.

## Notes, seals and sharing

Commands use Enter. N/P page through bounded screens, including 40x18 terminals.
A too-small screen asks you to resize; it cannot confirm unseen review content.

| Command | Action |
| --- | --- |
| E | Edit a note (512 ASCII bytes) and favorite-room text (96). Empty keeps; `-` clears. Review, then type `SAVE`. |
| I | Choose an existing local seal file. Review, then `COLLECT`, or `REPLACE` for a different seal. An identical seal is unchanged. |
| S | Preview selected share fields. A toggles the seal, E the note, F favorite rooms. Personal fields default off each time. |
| L | Browse saved entries, including removed contacts. C chooses a saved local ID. |
| B | After checking a changed local profile/pins, explicitly `REBIND` this notebook association. |
| D | Review and type `DELETE` to remove the local note/rooms/seal. |
| Q | Back to Boards. |

Nothing is saved until its explicit confirmation. Cancel/disconnect preserves
only completed saves. Share is a **local copy preview**, not a network send or
file write. It always omits local IDs/labels, profile bindings, endpoints, keys,
paths and timestamps; selecting personal fields deliberately exposes their text.

A snapshot hash detects changes to the local profile or known-host file. This is
**not verified remote board identity**, a verified visit, membership or admission.
Changed/unavailable snapshots and removed catalog contacts are labeled. Import
requires a readable current snapshot and explicit rebind after a change. Existing
notes remain readable/editable/deletable when a contact disappears. No deleted
contact restores a route or authorizes a connection.

## Original calling cards

Use the included [original Nightwire seal](../deploy/examples/passport/nightwire.toml)
as a starting point. A sysop can deliberately offer the same generic reviewed pack
through existing approved [Files/SFTP](file-areas.md), with ordinary download review,
or hand it over locally. Import requires a member-chosen local path. No remote
screen/frame can choose a file or collect a stamp for you.

```toml
version = 1
title = "My board"
artist = "Your name"
rights = "Original artwork; your license"
art = ''' / MY / BOARD /
   tune in, stay a while'''
```

Packs are at most 4 KiB and strictly parsed. Titles are 26 printable ASCII bytes,
artist 48, rights 96; art has at most eight 38-column rows (311 bytes including
newlines). No terminal controls, Unicode, raw ANSI, scripts or executable links.
Include accurate rights/provenance. A generic souvenir has no signature, verified
identity, timestamp, access-key or home-identity authority. Never encode a unique
visitor beacon. Verified contacts and integrated generic board offers remain
#161 follow-ons; this feature does not claim those capabilities.

## Storage, retention and plain commands

The private 0700 member-owned directory contains a 0600 `passport.toml` and a
fixed `passport.lock`. No symlink directory traversal or shared writable files.
Up to 32 entries fit a 64-KiB notebook. One lifetime lock prevents concurrent
clients from overwriting each other; updates replace atomically. A corrupt or
unsupported store is refused, never silently recreated. Init never overwrites an
existing directory/store. Keep parent folders private and reopen after another
client closes if busy.

Notes stay until you delete them. No automatic expiry or timestamp is stored.
Close the notebook before copying its directory to your own private backup;
restoring that copy restores its notes. No account/key material is included.
Deletion is logical: backups, filesystem history or storage recovery may retain
copies. There is no encryption or secure-erasure claim.

Plain/non-TTY commands also work:

```sh
faction-client --passport-store /home/member/.faction-passport list
faction-client --passport-store /home/member/.faction-passport export nightwire
```

List prints saved IDs/labels without notes. Export previews only one souvenir's
public art/artist/rights fields on stdout; it omits personal notes and favorite
rooms. You choose whether to copy/redirect it. No automatic file or network
publication occurs. Native note/share editing remains a local ANSI/TTY surface.
Server schema, helper/credential/session limits and encrypted mail are unchanged.

## Software evidence

Twenty-five client Rust tests and sixteen real local PTY controls pass per OS,
including the original Nightwire pack, malformed art, selective sharing, profile
changes, removed-contact management and resize confirmation barriers. Host strict
workspace clippy/365 Rust/130 Python tests and local client/vault/graphics
regressions pass. [Exact evidence](../dev-docs/evidence/passport-netbsd/results.json)
pins 72 matching native source/harness identities and 317 unchanged non-client
inputs. This is member-device software validation, not new server capacity or
physical SBC testing. Hardening remains later.

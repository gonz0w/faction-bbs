# Contacts / Other BBSes

Updated 2026-10-22. The optional **[O]ther BBSes** menu lets approved members browse
a sysop-curated local directory and read SSH connection cards. It adds another
place to discover private communities while keeping each board independent.

## Enable a reviewed local pack

The normal installer leaves the directory off. Copy the symbolic example
[`boards.toml`](../deploy/examples/boards/boards.toml) into a dedicated pack folder,
review every field and give the service owner read/traverse access. A typical
owner-managed folder is `/var/db/faction/boards`, owned by factiond/factionipc,
mode 700, with `boards.toml` mode 600. Root-owned read-only packs are also accepted
when the service owner can read them. Symlinks, linked/nonregular files, writable
packs and unknown fields are rejected. Metadata is loaded at startup, not per key.

Validate as the BBS Unix owner before enabling:

```sh
faction directory-check /var/db/faction/boards
```

For the shared service, use the normal stopped/drained restart workflow and add
this root-managed setting to `/etc/faction/service.toml`:

```toml
directory = "/var/db/faction/boards"
```

The existing `faction-service --check-config` checks the pack too. An invalid pack
refuses startup; it does not create a listener or silently enable an empty/public
directory. Remove the setting and restart to disable it.

Local console and member-worker development routes accept `--directory PACK`:

```sh
faction --state-dir /private/board login --ansi --directory /private/boards
faction-member --state-dir /private/board --directory /private/boards
```

Worker rollback uses the corresponding root-managed `faction_broker_flags`
`--directory` option; `service.toml` is the shared-backend configuration. Never
make a member terminal supply its own service config, trusted pack or worker flags.
No shell/account/getty or ordinary installation defaults change.

## Pack fields and publication scope

```toml
version = 1

[[boards]]
slug = "night-radio"
name = "Night Radio BBS"
summary = "Radio talk, local rooms and ANSI games."
host = "radio.example"
port = 2222
login = "bbs"
reviewed = "2026-10-22"
valid_until = "2027-10-22"
# fingerprint = "SHA256:...43 base64 characters..."
```

The example endpoint is illustrative. Use the destination's actual transport
login (often a shared terminal account, distinct from the member's BBS handle).
The pack holds at most 32 listings / 32 KiB. Slugs are unique lower-case ASCII
letters/digits/hyphens, up to 32 bytes; names 26, summaries 80, DNS/IP hosts 96 and
login tokens 32. Ports are 1–65535; raw URLs, SSH options, shell syntax, control
characters and unknown/private-credential fields are rejected. IPv4/IPv6 and DNS
names are metadata; private-network availability is the member's responsibility.
Optional fingerprints are public `SHA256:` hints with 43 base64 characters.

All approved local members can see the entire configured catalog. Keep passwords,
invite tokens, SSH private keys, private-network secrets and information the other
operator has not agreed to share out of it. The directory is not a public registry,
peer invitation or contact with the listed operator. Local members cannot edit
its source through the BBS. Pending/revoked accounts get no directory access.

## Browse and visit

Arrows/Tab select a board; Enter or A–D opens a card on the current four-entry
page. N/P or Left/Right changes list pages. Cards paginate longer metadata with
N/P; L returns to the list and Q returns directly to the main menu. Plain terminals
use the same letter commands followed by Enter. Resize returns to a working main
menu; reopen the directory for the new geometry. Themes supply the colors.

O is added only when the host enables the directory. A pack can explicitly define
a `contacts` menu action with another free hotkey/label. An O conflict refuses
configuration until the sysop resolves it. If Files plus Contacts exceed a 5x2
custom grid, the main menu uses the classic ordered layout and keeps all eleven
possible actions, including on 40x18 terminals. The directory's own pages remain
bounded. Disabled directory defaults and existing artwork/previews stay unchanged.

Cards show host, port, transport login, review dates and an optional host-key hint.
A complete SSH command is shown when it fits; otherwise use the displayed fields.
The server never clips a command into a different endpoint, resolves a hostname,
connects/proxies, launches SSH/browser tools, creates a profile or imports a key.

Open another local terminal and deliberately connect with your own SSH client, or
log out first if preferred. Check your SSH settings and verify the destination's
host key independently. A directory hint is supplied by this sysop and grants no
trust. Your local client may have its own identity/configuration; Faction does not
copy or forward credentials, device keys, agent access or membership to a listed
board. The destination handles SSH admission, registration and local BBS approval.
Native profile handoff remains future work; it must retain member-held keys and
independent local approval. No remote login or federation is implemented here.

## Submissions, review and listing lifecycle

For this first slice, another operator sends proposed public connection details
to the local sysop through an agreed out-of-band channel. The sysop obtains consent,
checks the endpoint/login/host-key hint independently, chooses a local slug,
records the review window and edits the pack. A listing is local editorial data;
there is no claim of remote ownership proof or automatic verification. In-BBS
pending submissions/review tools remain follow-on work under #27.

Past `valid_until` listings show **OLD**. A future review date or unavailable clock
shows **DATE?**. These are editorial dates, not online/availability tests. Expired
listings remain visible with their dates until the sysop reviews or removes them.
Removal/change takes effect on a stopped restart; active sessions do not silently
switch to another endpoint. An empty `boards=[]` pack is a usable empty directory.

Keep the pack with other reviewed installation configuration. Database exports
and complete file-area bundles do not include it, just as they exclude themes and
host configuration. Back it up separately; never put private host credentials in
it. A listed destination never receives rooms, files, chat or account data from
this directory. Optional browser handoff remains separate no-JS/private-portal
work, not a requirement for discovering or visiting another board.

## Validation

[Plan 0055](../dev-docs/plans/0055-local-other-board-directory.md) records software
checks. Local/worker PTYs cover paging, arrows/hotkeys, card return, compact/plain,
revocation, configuration and a loopback no-connect trap. Native shared SSH/client
checks exercise the actual maintained service separately. No new physical-SBC,
250-user, public-listener, game, host-action or hardening claim.

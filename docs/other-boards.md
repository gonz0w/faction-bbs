# Contacts / Other BBSes

Updated 2026-10-26. The optional **[O]ther BBSes** menu lets approved members browse
a locally moderated directory and read SSH connection cards. It adds another
place to discover private communities while keeping each board independent.

## Enable a reviewed local pack

The normal installer leaves the directory off. Copy the symbolic example
[`boards.toml`](../deploy/examples/boards/boards.toml) into a dedicated pack folder,
review every field and give the service owner read/traverse access. A typical
owner-managed folder is `/var/db/faction/boards`, owned by factiond/factionipc,
mode 700, with `boards.toml` mode 600. Root-owned read-only packs are also accepted
when the service owner can read them. Symlinks, linked/nonregular files, writable
packs and unknown fields are rejected. The curator pack is loaded at startup; published submission state is checked
again on directory actions.

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
The optional [native board launcher](board-launcher.md) can visit profiles you
configure and verify on your own device, with a local confirmation and fresh BBS
login. Directory-to-profile import remains future work. The directory itself
performs no remote login and implements no federation.

## Submissions, review and listing lifecycle

The owner-managed pack and member submissions are separate sources. The BBS
never rewrites `boards.toml`. With an enabled pack, including an empty
`version=1` / `boards=[]` pack, **S** from Other BBSes opens the submission desk. A listing never
verifies its operator or grants membership, host trust or federation rights.

Approved members can prepare a private draft, preview it and explicitly submit
it for review. Canceling unsaved input publishes and saves nothing. A saved draft
survives logout and restart. Pending content cannot change underneath a review;
withdraw it and create a new draft to revise it. Members can inspect and withdraw
their own records, including published ones. Other members cannot see unpublished
submissions. Do not enter passwords, invite secrets, private keys or information
the destination operator has not agreed to share locally.

Use **N** to enter a new listing. **S** on its preview saves a private draft;
**S** on the saved draft explicitly submits it. Use **1–4** to open your records,
**>** for the next page, **B** for the first page, **W** to withdraw an open record
and **Q** to return to the directory. Desk commands take Enter in both ANSI and
plain modes. **C** cancels unsaved input. Preview/card **N/P** pages longer text.

The current local sysop uses **M** for review and **L** for their own list. Open
a pending record and use **A**, enter review/valid-until dates, then type exact
uppercase `APPROVE`. **R** and `REJECT` reject it. Published records use **X** and
`REMOVE`, or **E** and `EXPIRE`. Any other confirmation cancels the operation.
The sysop reviews pending records, checks the endpoint and consent
independently, then approves or rejects the exact revision. Approval publishes one
listing with the sysop's review dates. Rejection, withdrawal and removal delete the
submission. Explicit expiry marks a published listing **OLD**; a past valid-until
date also shows OLD. Expiry is a warning, not removal. Current approval and role
are checked again on every action. Listings submitted by an unapproved author
are hidden until the author's local approval is restored or the sysop removes them.

There are at most 32 private draft/pending records across the board, four per
member, and 32 combined curator plus retained published/expired listings. Expired
or hidden published records still reserve a listing slot. Slug or canonical
host/port/login conflicts are refused at submission and approval. Conflicts reveal
no other member's private pending text. Records persist until explicitly removed;
there is no automatic purge, rejection log or submission history. When a queue is
full, reject or withdraw unwanted records to free space.

Changing the published directory returns an open card to the list for a new
selection. It cannot silently swap the selected endpoint. Curator entries retain
the pack edit/restart workflow. An incompatible pack/published-overlay union
refuses service startup before listening: reconcile the pack, or restart with the
prior compatible pack and remove the conflicting submission. Pending conflicts
can stay in the queue for rejection; they do not block startup.

Schema-19 owner database exports and complete file-area bundles include private
submission records. Back up the curator pack separately with installation
configuration; it remains outside those exports. Restore validates submission
bounds and content, discards live session authority and checks the pack union
when the feature is enabled again. Deleted records may still exist in older
backups or storage remnants. See [export and restore](export-restore.md).

## Validation

[Submission software evidence](../dev-docs/evidence/directory-submissions/README.md)
records #184 checks. The original
[design issue](https://github.com/gonz0w/faction-bbs/issues/27) retains the earlier
curated-directory checks. Local/worker PTYs cover paging, arrows/hotkeys, card return, compact/plain,
revocation, configuration and a loopback no-connect trap. Native shared SSH/client
checks exercise the actual maintained service separately. No new physical-SBC,
250-user, public-listener, game, host-action or hardening claim.

# Curated community library shelves

The optional **K:Library shelves** gives approved members a browsable collection
of existing local catalog files: named shelves, curator notes, declared rights and
provenance, copyable file references and deliberate discussion/Files jumps. The optional
[file request desk](file-requests.md) adds member create/fulfill/close and moderation
alongside these shelves, including requests-only packs. Optional [local magazines](board-magazine.md) also
share the library, with owner-reviewed editions and existing file/discussion links.

## Prepare a collection

Copy the [disabled template](../library/example/library.toml) into an owned pack
directory. Publish permitted files through the existing upload/review workflow,
then use their current `faction-file:BOARD_ID/OBJECT_ID` links in the manifest.
Replace the zero-ID template, note, rights and provenance with actual reviewed
values. This template supplies no file bytes or rights to a third-party work.

```sh
cargo build --workspace --release --locked
./target/release/faction library-check /owned/library
./target/release/faction --state-dir /owned/board login --ansi --library /owned/library
```

Local/worker startup accepts a fixed `--library PACK`. The shared-service owner
TOML accepts `library = "/owned/library"`. Use an absolute installed path, owned
accessible parents, directory mode 0750/0755 and owner/root-managed regular manifest
mode 0640/0644; members must not be able to edit it. Run the offline check as the
intended service owner. It checks syntax/metadata, not a member's live file grants
or legal permission. Set `enabled = true` only after review.

The version-1 manifest defines `name` (24 ASCII bytes), `compact`, up to eight
`shelf` tables (`id`, `title`) and up to 32 `book` tables (`id`, `shelf`, `reference`,
`note`, `rights`, `provenance`, optional positive local `room`). IDs use lowercase
letters/digits/hyphens. Notes are at most 240 printable ASCII bytes with optional
newlines; rights 96 and provenance 160 single-line bytes. Shelf titles are at most
32 bytes. Unknown/duplicate fields and IDs, unknown shelves, malformed references,
controls, non-ASCII pack metadata and oversized inputs are rejected. There are no
member-supplied host paths, URLs, raw art assets or network locators.

A trusted theme may place/rename an `action = "library"` entry with another key.
A theme cannot enable the host capability. Otherwise K is reserved while enabled,
with a clear startup error on a conflicting custom key. Preloaded seasonal themes
receive the same capability; crowded lightbars and direct/classic hotkeys remain.
No enabled shelf entries, request desk or published magazines means no automatic
library menu action.

## Browse and discuss

E selects an eligible shelf (`-` means all); V selects a stable entry ID. N/P pages
lists or notes. L shows a current copyable local file reference; I returns to notes.
D deliberately opens the linked discussion room through the existing New reader.
Q returns to the shelves/main menu. Plain/narrow sessions use the same commands
followed by Enter. ANSI section styling follows the sysop's current theme.

The current filename, description, size and checksum come from the existing file
catalog. Shelf notes are supplementary owner-curated metadata. A shelf appears
only when it has a currently eligible published file. Private-only shelves expose
no title or count to an unauthorized viewer. Pending, removed, unavailable and
foreign-namespace references expose no entry captions/notes/tokens. All selection,
render/link/transfer/discussion actions recheck current local approval and source
access. A withdrawn selected source returns to a generic catalog, never opens a
successor. Previously seen/downloaded text cannot be recalled.

Viewing shelves/notes/links leaves unread and download state unchanged. D uses the
normal room reader and its existing complete-message/drained-output acknowledgment;
this deliberate read can advance unread. Missing discussion rooms are unavailable;
the file can remain readable independently. No discussion body is copied into the
pack, and a room link does not grant access or authorize republication.

## Existing transfers, permissions and rights

On the shared service, F is offered only when Files is configured. It rechecks the
selected object, shows its current reference and opens the existing Files lobby.
The native client collects it in its existing recent-link list; choose D and #1
(or the shown recent index), then review the local destination/transfer confirmation.
Compact-native visits and chosen payload ceilings still apply. Ordinary SSH keeps
the manual file workflow. Local/worker consoles expose metadata/links and room
reading; they do not gain a transfer adapter from this pack.

There are no new file bytes, grants, quotas, publication/review paths, network
fetches, archive extraction or automatic transfers. SFTP still needs the current
key-bound grant and separate restricted service. Putting a private/unpublished
file in a shelf never makes it public. License/provenance text is a curator's
record, not automatic proof of distribution rights: review the actual file and
permission before publishing it. Requests do not authorize pirated uploads. Local
approved-member display/download does not authorize public/Gopher/NNTP publication,
resale or permanent-chain publishing.

The owner or an appointed offline curator installs/revises/removes the reviewed
pack during a stopped/drained restart. Active sessions retain an immutable catalog
snapshot; changing the pack alone does not hot-reload them. File withdrawal and
grant changes take effect on the next current-authority frame/action without a
pack restart. Set `enabled = false` and restart to withdraw the whole capability.
For urgent withdrawal, stop the service or withdraw the catalog object.

## Limits and recovery

The manifest is at most 32 KiB; 32 entries/eight shelves bound each visibility scan.
Only four entry cards/eight note rows render per frame, with 32 prior text offsets.
Frames stay within 8 KiB. The immutable shared catalog is loaded once; each member
gets a small optional desk, not copies of all files or stored file metadata.
Existing session/credential/helper, 32-KiB output and five-second drain limits stay.
There is no new registry dependency, listener/helper or capacity measurement.

Packs/curation permission records are external host configuration, preserved and
checked separately alongside existing complete database/blob backups, like themes
and galleries. Shelf metadata stays external; the optional [request desk](file-requests.md)
adds schema-15 request data. Restore the intended owner's pack/config and recheck current local file
references before restart. File catalog/object/grant recovery stays in the existing
backup workflow. Private mail/attachments remain device-encrypted and separate.

See [file areas](file-areas.md), [ADR 0065](../dev-docs/decisions/0065-curated-library-shelves.md),
[plan 0070](../dev-docs/plans/0070-library-shelves.md) and
[software validation](../dev-docs/evidence/library-netbsd/README.md).

# Curated terminal art gallery

The optional A:Art gallery gives a board its own small creative scene: original
or licensed ANSI/ASCII works, themed exhibits, artist notes, paged art and an
explicit readable text edition. Main menus retain arrows/Enter and direct hotkeys;
inside the gallery use the displayed commands followed by Enter. Ordinary
approved local membership is required. No public publication or federation occurs.

## Try and configure

The [Nightwire pack](../gallery/nightwire/gallery.toml) contains three original
three-page city/season works and one ASCII sketch. Its included BSD-3-Clause
license covers those originals. It makes no claim to archival ACiD/iCE/Razor art.
Existing [Nightwire reference previews](artwork-nightwire.md) show the same city
cells; actual terminal fonts/pixels and owner artistic acceptance remain open.

```sh
cargo build --workspace --release --locked
./target/release/faction gallery-check gallery/nightwire
./target/release/faction --state-dir /owned/board login --ansi --gallery gallery/nightwire
```

Local/worker startup accepts a fixed `--gallery PACK`; shared-service owner TOML
accepts `gallery = "/owned/gallery"`. Use an absolute path with accessible, owned
parents in installed deployments. Copy the pack and all its assets/license to an
owner/root-managed directory, files mode 0640/0644, directory 0750/0755 as needed.
Run the offline check as the same owner that will load it. Never give members
write access. Changing the pack requires a reviewed stopped/drained restart;
active sessions use an immutable startup snapshot. Remove a work or set
`enabled = false` and restart to withdraw it. There is no member path selection
or hot reload. No configured/enabled works means no automatic A action.

Trusted themes may declare `action = "gallery"` with a different key/position;
a theme alone cannot enable it. Otherwise A is reserved when enabling the pack,
with a clear startup error on a conflicting custom key. Seasonal theme copies
receive the same startup capability. Main menus preserve crowded/narrow fallback.

## Curation, rights and text editions

Use the example manifest's exact version-1 fields. Set a gallery heading,
`compact` card layout, `formats = ["ansi", "ascii"]` (or a subset), up to four
named exhibits and up to eight stable-slug works. Each work needs its exhibit,
title, artist, description, license, provenance, `source = "original"` or
`"licensed"`, an asset/fallback basename and `display_permission = true`.
Metadata must be bounded inert UTF-8; classic presentation escapes non-ASCII.
The ASCII fallback is mandatory, meaningful and readable, not an empty placeholder.
ASCII art itself cannot contain escape sequences.

A sysop or an owner-appointed offline curator reviews provenance and actual
permission before installing an entry. Declaration validation cannot establish
legal ownership. Keep permission records privately when appropriate. Members can
submit through an existing pending file area or another agreed offline intake;
this adds no upload endpoint or automatic promotion from Files to the gallery.
Review content, license, credit, fallback and importer output, then explicitly
copy a permitted artifact into the owned pack. Original/licensed local display
permission does not imply download, resale, public syndication or permanent chain
publication. Remove contested works through the same owner-controlled restart.

Optional `download_ref = "faction-file:BOARD_ID/OBJECT_ID"` requires a separate
`download_permission = true`; otherwise omit the reference and set false. Publish
an independently permitted copy through normal Files review. The gallery resolves
the token against the current local namespace/approval/file grants on every link
frame. An unavailable/private/removed file exposes neither filename nor token.
Use the existing Files/native-client transfer workflow for the displayed token;
viewing art does not automatically transfer it or grant access. Curators must avoid
placing private filenames/tokens in public descriptive text themselves.

## Navigation and bounded presentation

E selects an exhibit (`-` means all); V selects a stable work ID. N/P pages lists
or artist notes. A opens art; N/P pages its canonical sixteen-row sections, T
switches art/text, I returns to notes, Q returns to the catalog/main menu. At least
80x24 ANSI cells display art; narrower/plain terminals use the explicit text
edition. Resizing resets page offsets for the new presentation while retaining
selection. Text remains available even on a capable terminal.

Raw art bytes never reach members: the existing bounded CP437/ANSI/SAUCE/iCE
importer accepts only canonical cells and rejects active OSC/DCS/APC/query/control
sequences. This grants no bitmap/Kitty authority. There is no new network/exec I/O
in the gallery workflow. Approval/epoch checks apply to every frame and command.
No room unread cursor, private-mail content/count or decrypt state changes.

Limits: 32 KiB manifest, 64 KiB/art asset, 256 KiB aggregate art input, 64 total
80x16 cell pages, 7168 encoded bytes/art page, 8 KiB/fallback (32 KiB aggregate),
128 lines/fallback, 72 columns/line. At most four cards/frame, eight text rows and
32 prior text offsets. All frames stay within 8 KiB and the service's existing
32 KiB output/five-second drain limits. Catalog cells are shared once in the shared
owner; no new helper/listener or session/credential defaults. These bounds are
not a fresh 250-member capacity measurement.

The pack is external host configuration, like themes: database exports/bundles do
not contain it. Preserve the exact pack, rights records and configuration separately;
restore/check them as the intended owner before restart. Schema remains 14; no
new database migration or gallery state is added.

See [ADR 0063](../dev-docs/decisions/0063-curated-terminal-gallery.md),
[plan 0068](../dev-docs/plans/0068-scene-gallery.md) and the
[software validation](../dev-docs/evidence/gallery-netbsd/README.md).

# The local board magazine

The optional **K:Library shelves → M:magazine** adds owner-reviewed local editions:
a themed table of contents, paged essays/tutorials/ASCII interludes, contributor
credits, revision/edition notes and optional current-permission text downloads.
Try the [original five-article Nightwire issue](../library/nightwire/README.md),
including an editorial, file-request field notes, original ASCII, arcade ideas and
a colophon. No third-party art or game bytes are bundled.

This is the reader/editor-pack increment of #143. The sysop assembles a reviewed
pack offline. Member submission queues, in-board editing, authenticated contributor
approval and automated release schedules remain follow-on work. A pack's consent
field records the editor's declaration; it cannot authenticate an author or prove
distribution rights. The reader deliberately keeps that distinction.

## Try and configure

```sh
cargo build --workspace --release --locked
./target/release/faction library-check library/nightwire
./target/release/faction library-preview library/nightwire signal-001
./target/release/faction --state-dir /owned/board login --ansi --library library/nightwire
```

For shared SSH, install an owner/root-managed copy and set the existing service
`library = "/owned/nightwire"`. Keep the [library pack permissions](library-shelves.md)
and optional capability/theme rules. A magazine-only pack uses empty `shelf = []`
and `book = []`; requests and ordinary shelves can share the same pack. Older
manifests remain valid. No main-menu action appears when there are no published
issues, shelf entries or enabled requests. Enable this feature by deliberately
selecting the pack; the ordinary board still starts without it.

`magazine_name` sets the desk label (24 printable ASCII bytes), and existing `name`,
`compact` and theme `action = "library"` control the parent title, card density and
main-menu layout. The same active/seasonal theme styles the reader. Classic/plain
sessions use the same commands followed by Enter. Narrow reading wraps text;
ASCII interludes remain accessible as printable text rather than raw ANSI controls.

## Assemble, review and release

The version-1 library manifest accepts up to eight `[[magazine]]` tables. Each has
unique `id` (32-character slug), `number` and `revision` (1–9999), `title` (64),
`state` (`draft`, `published`, `withdrawn`), `editor` (64), `released` (32) and
`schedule` (64). Text lengths are ASCII bytes. `released` and `schedule` are editor
labels, not machine-enforced dates or timers. Optional `download` is a canonical
local file reference and optional `room` a positive local discussion-room ID.

Each issue contains one to eight `[[magazine.article]]` tables, at most 32 articles
across the pack. Article fields are `id` (unique within its issue), `title` (64),
`section` (24), `author` (64), `rights` (96), `provenance` (160), `consent` and `body`.
The existing parser also caps opening bracket bytes at 256 across the combined
manifest, including text. The full [starter manifest](../library/nightwire/library.toml) is a working example.
Bodies allow printable ASCII and LF only: at most 8192 bytes, 128 lines and 120
bytes per line. There are no image/clipboard controls, member-selected paths or
unreviewed network assets. Place file references only in the dedicated optional
issue-download field so they receive current file checks.

Keep submissions and exact contributor approvals in private editor storage. Use
`state = "draft"` and `consent = "pending"` while reviewing. Invite original work;
ask before using a room conversation, private material or another author's art.
Have each contributor review the exact text, attribution, license, intended local
audience and proposed revision. Record permission provenance separately. Do not
copy private-room material automatically. This pack loader verifies declarations,
not who supplied or approved them.

The offline check lists all issue IDs/states/revisions. `library-preview PACK ID`
prints a bounded inert text edition, including draft/withdrawn content, for the
host-authorized editor; it is unavailable through the member frontend. After
review, change each contribution to `consent = "board-local"`, set the issue state
to `published`, assign the reviewed revision/edition label, check again and install
during a **stopped/drained restart**. A published issue containing pending consent
rejects the whole pack before login. Draft/withdrawn issue titles, bodies and counts
are absent from member lists and cannot be opened by guessed ID.

The member reader shows four issue/article cards per page. V selects an issue ID,
R selects an article ID, N/P pages. Articles and full credits read in eight-row
pages; C opens credits, R returns to the article, T returns to contents and Q goes
back through editions/library/main. I on contents shows full issue/editor/schedule
notes. Resize restarts the selected text at its beginning; it never publishes,
downloads or acknowledges a discussion. Library/magazine previews leave unread
unchanged. D deliberately opens an existing linked room through the normal New
reader and its usual complete-message/drained-output acknowledgment.

## Text editions and current file rights

```sh
./target/release/faction library-edition /owned/nightwire signal-001 /private/issue-001-r1.txt
```

Only a published issue can be exported this way. The owner tool creates a new
mode-0600 file, refuses existing files/symlinks, writes bounded plain text and
performs no network call or catalog publication. Keep the destination parent private.
Upload the permitted edition through the existing [file areas](file-areas.md),
complete sysop review, then add its current reference as `download`. This is an
optional file offer; changing the manifest does not regenerate a previously
published object. Issue revisions need their own newly reviewed immutable objects.

L exposes an edition reference only when the viewer currently has access to that
published local object. Shared F uses existing Files/native recent-link discovery;
the member still chooses and confirms the download. Native compact mode/payload
ceilings, restricted SFTP grants, quotas and review remain authoritative. Pending,
removed, deleted, private-ungranted and foreign references expose no token or file
caption. A grant withdrawal rechecks on the next frame/action. Local/worker
consoles provide copyable metadata without a transfer adapter; plain SSH retains
manual transfers. Deliberately published magazine text remains independently
readable even when its optional file offer is unavailable.

## Revision, withdrawal and recovery

Preserve a private immutable copy of every released pack revision and its exact
permission records; IDs/numbers/revisions are editor-managed labels, not signatures
or a verified monotonically increasing ledger. Live replacement loads only on a
stopped/drained restart. To withdraw, stop the service, set `state = "withdrawn"`
and restart, or remove/disable the pack. Active sessions otherwise retain their
immutable startup snapshot. For urgent removal, stop the service first; merely
editing the disk manifest cannot recall already loaded text. Withdraw the separate
file object through Files too if its download must stop. Previous exports,
member-downloaded copies and backups cannot be recalled or securely erased.

Local release/author intent does not automatically authorize public HTTP/Gopher,
NNTP, cross-board federation or permanent publication. Those remain separate
explicit audience/rights decisions. The pack adds no member tracking, submission
store or replication. These are ordinary local community publications, separate
from member-device encrypted private mail and attachments.

Magazine packs and permission records remain external configuration: preserve them
alongside complete database/blob/config/key backups and restore the intended pack
before restarting. **Schema stays 15**; database snapshots do not contain these
editions. Recheck download namespaces/grants and room IDs after restore. The entire
combined manifest remains 32 KiB; all magazines together share that bound. Eight
text rows/four cards, 32 recent previous offsets (forward reading remains unlimited within the article)
and an 8-KiB frame cap bound each optional
desk. One immutable shared Catalog Arc holds content, with no per-client article
copies/cache, new registry dependency, listener/helper or changed ordinary budgets.
There is no new physical-hardware, capacity or hardening claim.

See [ADR 0067](../dev-docs/decisions/0067-local-magazine-reader.md),
[plan 0072](../dev-docs/plans/0072-local-magazine-reader.md) and
[software evidence](../dev-docs/evidence/magazine-netbsd/README.md).

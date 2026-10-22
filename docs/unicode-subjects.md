# Unicode room subjects

Status: implemented software preview, 2026-10-22, #114. Rust/NetBSD; no new runtime
dependency, news transport, hardware or capacity promise.

## Composing and reading

Local consoles, inherited member workers and shared sessions accept valid UTF-8
subjects up to **160 bytes**, independently of screen width. Empty input retains
the generated/default behavior. Reject malformed/incomplete UTF-8, controls,
surrounding whitespace and encoded-word syntax. No Unicode normalization changes
the authored bytes. Busy retries keep the original subject/body/parent/request ID;
changed content under that ID conflicts. Current approval/epoch still controls
posting, reply lookup and reading.

The common input editor handles characters split across reads. Backspace removes
one complete scalar; deleting an incomplete pending scalar leaves the preceding
character intact. Malformed/control/oversized input ends that connection without
posting and restores the local/worker terminal. Subject input has a 120-second
local deadline and the existing shared prompt deadline/output/session bounds.

The default **compatibility display is ASCII**: typing `café` stores those exact
UTF-8 bytes while echo/headers show `caf\u{e9}`. This also makes combining marks,
bidi controls and terminal-control text inert. Paged headers and reply previews
clip whole escape tokens to available columns, at most 119 cells; byte length is
not used as a claim about native glyph width. Plain and ANSI sessions use the same
text policy. [Terminal settings](terminal-capabilities.md) now opt into native width-one/two
Unicode rendering and an isolated sync probe under #96. Invisible/cluster-dependent
forms remain escaped; bitmap graphics are not enabled. Neither TERM nor a brand
enables richer rendering automatically.

Blank replies inherit one `Re: ` prefix. Defaults are clipped only at complete
UTF-8 boundaries within 160 bytes, then trailing whitespace is removed. User
subjects are immutable after posting. The v2 wire projection already emits UTF-8
encoded Subject headers; this increment changes stored/composed metadata only.

## Upgrade, backup and rollback

Schema 6 keeps its original **ASCII** contract. Schema 7 uses a distinct checked-in
catalog with a 1–160-byte BLOB-length constraint plus Rust UTF-8/control validation.
An older binary rejects version 7 rather than reading wider metadata as schema 6.
Follow the [complete stopped-service procedure](post-metadata.md):

1. Stop member/file admission and drain sessions, transfers, doors and backend.
2. With the matching old tool, save/verify a complete private backup including
   SQLite, file blobs, keys, themes and configuration; keep old binaries.
3. Upgrade binaries together. Before the first new owner open, keep admission
   stopped. The 6-to-7 transaction checks the exact metadata catalog and old ASCII
   rows under a five-second work deadline, copies existing rows, replaces the
   byte constraint and advances the version atomically. A failure rolls back this
   stage; older migrations retain their existing staged behavior.
4. Verify a fresh export and inspect subjects, IDs, dates/provenance, replies,
   unread and file references before reopening admission.

No old label, public article ID, original date or legacy provenance is rewritten.
Schema-2 through schema-7 exports are checked against their exact catalogs and
version-specific row validation without migrating the source. A restored old
store upgrades on first owner open. Version-7 restore retains Unicode bytes and
public IDs while rotating private session authority. No live/mixed-version upgrade.

Rollback means restoring the matching verified pre-upgrade **complete** backup
with its older binaries/configuration into fresh state/store paths. Never relabel
or downgrade an in-place DB. Changes after that backup are absent from it. Keep
all original private locks/WAL/SHM and the owner's saved playground intact.

## Software validation

140 Rust tests pass on macOS ARM64 and native NetBSD 11 ARM64. Five actual PTY
controls on each host cover plain/ANSI/local/inherited-worker entry, split/deleted
scalars, exact 160-byte subjects and malformed/incomplete/control/overflow denial.
Twelve native shared-service/frontend controls include Unicode defaults, clipped
headers and active revocation during composing/reading. Existing paging, unread,
approval and snapshot regressions pass. See [recorded evidence](../dev-docs/evidence/unicode-subjects-netbsd/README.md).
These are software fixtures, not physical SBC or 250-member measurements. Keep
[later hardening](later-phase-hardening.md) separate from this product work.

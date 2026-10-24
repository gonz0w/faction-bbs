# 0080: Private local board passport

Issue: [#161](https://github.com/gonz0w/faction-bbs/issues/161)
Status: implementation and software validation complete

## Outcome

Batch a device-local notebook, explicit original/licensed ASCII-seal import,
private notes/favorite-room text, selective share preview and removed-contact
management. The optional native Boards J action runs only between SSH visits.
CLI init/list/public-seal export is available without ANSI/TTY. Ordinary SSH/plain
BBS fallback remains; no server/schema/dependency/default/resource change.

Keep no automatic timestamps, visit log, remote beacon or cross-board presence.
A seal is unverified generic artwork: no visit, membership, verified board identity,
admission, signature, home identity or private-mail authority. Existing profile
and host-pin bytes form a local configuration snapshot only. Changed/unavailable
snapshots and removed catalog contacts stay explicit; no silent rebind/import.

Store up to 32 entries in a private member-owned 0700 directory, one 0600 plaintext
TOML notebook capped at 64 KiB, a fixed lifetime lock and atomic descriptor-relative
updates. Notes 512 printable ASCII bytes, favorite-room text 96, seal up to eight
38-column rows plus required rights/artist metadata. No controls or raw ANSI.
No notes/private fields share by default. No endpoint/key/path/binding/timestamp
in exported souvenir fields. No password store or server mail plaintext; existing
device-encrypted private mail stays independent. Explain plaintext storage and
private backup/deletion expectations rather than implying encryption.

Sysops can offer reviewed generic seal TOML through existing approved Files/SFTP
or another deliberate local handoff; no new remote import frame. Include an
original Nightwire example. Owner-managed automatic distribution and verified
contact binding remain #161 follow-ons; do not close the full story.

## Progress

- [x] Preserve snapshot 47: 330 prior connection-desk inputs and its whole result.
- [x] Build bounded inert model/private atomic store and opt-in notebook interface.
- [x] Add explicit review/collect/replace, note edits, selective sharing and deletion.
- [x] Add original calling-card pack and plain local commands.
- [x] Complete host/native checks and exact source evidence.
- [x] Merge checked #178 (1c78fa3); join its reviewed main baseline before publication.
- [x] Finalize docs/evidence and remove owned fixtures; run final archive checks before publishing the focused PR.

## Validation

Validate actual local PTYs for explicit initialization, 40x18, notes across restart,
seal deduplication/replacement, default-redacted sharing, changed snapshots,
removed contacts, malformed controls, cancellation/concurrent locks and no network.
Run unit bounds/format/storage tests, strict workspace clippy, host Rust/Python and
maintained local client/graphics/vault regressions. Native NetBSD software uses a
separately owned clone; no physical hardware or new server capacity run. Keep
historical evidence exact and saved owner playground stopped/intact.

## Results and corrections

Strict workspace clippy, 365 host Rust tests and 130 Python tests pass. Both OSes
pass 25 client tests and sixteen real passport PTY controls, plus fourteen
connection-desk, seven launcher, six travel, eight vault-picker and thirteen
local-vault controls. Host native graphics additionally passes its three controls.
The native record matches all 65 selected client/dependency/example inputs and
seven harnesses, with 317 non-client execution identities unchanged. No server
capacity or physical hardware run is claimed. Snapshot 47 freezes 330 prior
inputs and the entire connection-desk record; guard 61 covers this bounded result.

Compilation caught parse-only TOML/no serializer assumptions and a test-call edit;
resolved using a strict bounded canonical writer without adding features/packages.
A macOS storage fixture initially used a symlinked temporary parent; canonicalized
the fixture, preserving the product no-follow rule. PTYs caught long-art notices
below the first page; moved notices first. Review refuses confirmation at small
sizes until a real content page is visible, tested across resize. Malformed-control
and uninitialized-store harness expectations were corrected before final runs.
Final tests are clean; explicit save failures avoid claiming unchanged disk state.

Complete all 71 offline commands in the checkout and committed source archive,
remove owned fixtures with saved-playground disk/login/key comparison, then open
a focused PR referencing #161. Keep the story open for verified contacts and
integrated generic offers; no automatic travel tracking or public registry.

Owned clone/vendor/private fixtures are removed. Saved playground disk metadata
and login/key hashes remain unchanged and its status is stopped. Final publication
requires all 71 offline checks in the checkout and committed source archive.

All 71 final checkout commands pass. The current main squash tree exactly matches
starting c6be0f6; join that reviewed baseline without rewriting history. Run the
same 71 commands in the committed source archive before PR publication.

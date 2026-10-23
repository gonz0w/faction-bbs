# 0061: Local member pages and dated .plan updates

Issue #135. Status: implemented and software validated; ready for review.

The owner requested merging checked #150 and the next useful product batch,
classic fallback and later hardening. Select #135 as the local social continuation
of the wall: member-owned bio/interests/ASCII signature/project and dated updates.
#150 first exposed a stale exact sysop-prompt assertion in the older custom-menu
PTY. The assertion now checks the existing stable Create-room suffix; runtime
wall sources and all old measured fixtures/results remain unchanged. Merge only
after the current-head complete workflow passes.

## Scope

Optional approved-local-member pages. Four optional ASCII fields (240/120/240/120
bytes; 4/2/4/2 lines, <=72 columns), explicit field visibility, whole-page publish/
hide, preview/send/removal and bounded immutable dated .plan history. No mandatory
real name/location/external identity, home directory/files, Finger service, home
proof, mail/decrypt/notifications or federation. Sysop policy: label, card/compact
presentation, allowed fields, retained 1–16 updates/7–365 days and hide-only versus
explicitly enabled removal moderation. Original authorship stays intact.

Use shared faction-pages workflow, with existing core/terminal dependencies only;
local/worker and shared service routes, inert themed/plain frames <=8 KiB and
existing defaults/credential/helper/output limits. New optional action uses M or
a free sysop-defined hotkey. Default disabled. Pending/revoked author pages do
not appear in view/list/count paths; returning approval permits existing pages.

Schema 10 owner exports include policy, pages and retained plans. Older schema-2..9
exports remain verifiable/read-only; open upgrades. Stop/drain and make a verified
complete pre-upgrade backup. Fresh-path rollback, logical delete versus older WAL/
backups and plaintext/sysop-readable scope stay explicit. Hidden social fields
are local visibility controls, never encrypted private mail.

## Checklist

- [x] Merge passing #150; preserve actual wall inputs/result before edits.
- [x] Snapshot 28 freezes 202 prior inputs and whole community-wall record.
- [x] Core schema/API, own authoring, visibility/history/retention, moderation.
- [x] Shared reusable UI, sysop settings and classic ANSI/plain/compact routes.
- [x] Meaningful core/UI/PTY/SSH/native/recovery tests and clean native build.
- [x] Pin actual evidence, remove disposable VM/cache, keep playground off,
  update docs/status, full policy checks and focused PR.

## Results

Checked #150 merged a12eb87 after corrected workflow 39885668295 passed. The
member-page branch was rebased onto that integration head. The correction changed
only a stale custom-menu prompt assertion; historical measured bytes were already
preserved and all hashes remain fixed.

Rust 1.96 workspace: 235 per OS. Host Clippy/format and reviewed registry inventories
pass. Python 118. Eight member-page local/worker PTYs per OS, fourteen actual
SSH/host-native/NetBSD-native/plain controls with two members. Thirteen complete
member-page/wall/file recovery checks per OS; native actual CLI with distinct
non-root owner/file UIDs. Older schema-9 export/upgrade and schema-10 exports pass.

Wall seven per OS/eleven actual shared, full-screen checkpoints seven per OS/eleven
actual shared, unsaved line desk thirteen per OS, browser ten per OS, shared reader
22, local menu/reader, custom grids four per OS, native directory eleven, launcher
seven, operator eight and artwork eighteen previews/seven local/seven actual
shared pass. No new load/capacity, pixel/font or hardened-host claim.

The member-pages-netbsd record pins 203 actual native source matches and 22
selected host harness inputs. All twelve workspace packages were cleaned before
native build; installed binaries/literal match. Snapshot 28 freezes 202 prior
inputs and original whole result; keep 42 saved-evidence guards. Owned VM/cache
removed and saved playground off. Visibility is tested with another member, not
just own views. Hidden profile text stays plaintext owner data, never private mail.

Final review found that physical row deletion would also discard the sysop hide.
Own removal now clears all authored content/history but preserves a minimal empty
hide record, with a fresh revision and unpublished visibility. Core and an actual
SSH delete/recreate negative case prove the author cannot clear that gate. The
actual member-page suite is fourteen controls. Member field choices use human
field numbers/all/none, and revision conflicts explain cancel/reopen recovery.

A repeated native fixture inherited plain/theme settings from the prior art suite;
its next suite expected ANSI. Final fixture preparation now explicitly restores
the ordinary eight-session ANSI configuration before initializing synthetic data.
No production startup/config behavior changed to work around that harness state.

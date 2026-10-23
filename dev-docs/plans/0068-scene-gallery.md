# 0068: Curated ANSI/ASCII scene gallery

Issue: [#144](https://github.com/gonz0w/faction-bbs/issues/144)
Status: implemented and validated; ready for focused PR review. Owner: merge #165 and continue useful product work.

## Goal and scope

Opt-in owner-curated exhibits with original/licensed ANSI/ASCII work, artist notes,
explicit display/download rights and provenance. Immutable startup-loaded catalog,
canonical-cell art, bounded paged notes/art, text edition for narrow/plain devices,
optional current-permission local file reference. Reuse original Nightwire assets;
no archival downloads, raw control passthrough or bitmap/Kitty authority.

Host configuration selects a validated pack, its heading/layout/formats/exhibits;
no member-selected path. Curators review originals/licensed submissions offline or
through the existing pending file area, then install a reviewed owned pack while
stopped. Removal/format disable requires reviewed restart; no hidden hot reload.
Current approval/epochs apply every member frame/action, download references resolve
under current file permissions each time. Gallery display grants no download access.

At most eight works/four exhibits, 32-KiB catalog, 64-KiB input per work/256-KiB
aggregate, <=64 canonical sixteen-row pages, <=7168 encoded art bytes/page, 8-KiB
fallback each/32-KiB aggregate, 128 <=72-column ASCII lines each. Four catalog cards,
<=8-KiB frames, <=32 prior text offsets. One shared catalog, no per-member art clone,
new helper/listener or registry dependency. Schema remains 14; catalog is external
host configuration to preserve separately, not a DB owner snapshot or member upload.

## Progress

- [x] Merge passing #165, read #144/#25 and preserve previous measured inputs.
- [x] Add bounded owned pack loader, rights/format validation and original samples.
- [x] Add common approved local/worker/shared gallery and safe classic rendering.
- [x] Validate rights/paths/caps/approval/resize/file grants and native NetBSD paths.
- [x] Update docs/README/ADR/evidence/CI and prepare focused PR handoff.

## Gates

Snapshot 35 preserves 258 preceding actual inputs and the whole discussion result.
Keep all 48 historical identities/guards fixed. Existing eight-session/5000-ms,
two credential permits, three helpers, queue/drain limits unchanged. Use only an
owned marked software clone; saved playground off/preserved and hardening later.
No new capacity, physical hardware, pixel-quality or broad asset-sandbox claim.

## Validation outcome

Host/native NetBSD: 302 Rust tests per OS, eight local/worker gallery PTYs per
OS; 129 Python tests. Eleven actual restricted SSH/host-native/NetBSD-native/plain
controls cover exhibits/art/notes, text toggle/resize, current file grants and
revocation, format withdrawal, disable and rights preflight. Seventeen existing
complete-recovery controls per OS pass, with actual native distinct-UID CLI.
Existing discussion/catch-up/polls/social/editor/browser/menu/operator/art/vault
regressions and full host terminal CI pass. Registry inventory remains unchanged.

New gallery-netbsd evidence pins 269 matched native inputs, 25 host harness
inputs and thirteen unchanged access/credential/dependency inputs. Broker fixed
--gallery argv and service config are intentionally changed and excluded from
that unchanged group. Installed current native service literal/binary hashes
are checked. Owned VM/cache/test credentials removed; saved owner playground
stays stopped with disk/accounts/keys intact. No new capacity or pixel claim.

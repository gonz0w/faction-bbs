# Local member-page software evidence

2026-10-22, macOS ARM64 and an owned disposable NetBSD 11 ARM64 VM, two cores/
1 GiB, Rust 1.96.0. This is functional software evidence, not new supported
capacity, incremental-memory, physical SBC or hardened-host evidence.

The [result](results.json) checks 203 selected native execution inputs against
actual guest source bytes, 22 selected host harness inputs and the unchanged
reviewed dependency/access/credential inputs. Four installed binaries match the
clean native build, which cleaned all twelve workspace release packages first.
The installed service has the actual member-page success literal; real restricted
SSH/native sessions exercise that image, rather than relying on copied mtimes.

## Validation

- Rust workspace: 235 tests per OS. Host Rust 1.96 Clippy `-D warnings`, format and
  standalone UI compatibility compile pass. Registry inventory/features and no-JS
  policy remain unchanged; faction-pages is a core/terminal-only workspace crate.
- Member pages: eight local/worker PTY controls per OS, including ASCII preview/
  Save, visibility, hidden dated history, own deletion, plain no-escape output,
  compact layout, inherited worker and independent board policy.
- Actual shared SSH/host-native/NetBSD-native: fourteen controls, including two
  different members, hidden fields/plan refusal, active page hiding, stale edit
  conflict, single Busy retry with history pruning, resize, sysop hide preserving
  authorship, own delete/recreate retaining sysop hide, active revocation,
  escape-input refusal and confirmed deletion.
- Core tests: board/epoch/approval/role boundaries, no foreign hidden bodies/counts,
  field availability, own revision conflicts, persistent moderation, retention,
  immutable UTC dates, same-key retry, overflow/control limits, owner recovery,
  schema-9 read-only export/upgrade and schema-10 export/restore.
- Complete member-page/wall/file bundle recovery: thirteen per OS. Native actual
  backup CLI runs with distinct non-root board/file UIDs. Hidden plan, page fields,
  policy, wall note/receipt and file bytes survive restore. Ordinary owner export/
  verify/restore per OS pass. Logical deletion remains distinct from old backups.
- Regressions: local wall seven per OS and eleven actual shared; editor/checkpoints
  seven per OS and eleven actual shared; line desk thirteen per OS, browser ten
  per OS, shared reader 22, ordinary local menus/reader per OS; custom grids four
  per OS, native directory eleven, launcher seven, operator eight. Artwork has
  eighteen previews, seven local/worker and seven actual restricted SSH/native
  routes. No new pixel/font/owner artwork-acceptance claim follows.
- Python tests: 118. New saved-evidence guard 42 protects this batch. Snapshot 28
  preserves 202 previous wall execution/harness/compatibility inputs and the whole
  original result, with all forty-one historical record identities unchanged.

New tests intentionally cover visibility as seen by a different approved member,
not just the page's author. The visibility masks are plaintext application policy,
not end-to-end encryption. New native mail/keys/decrypt stay member-side under their
separate work. No profile data is a host `.plan` file, public Finger service, home
identity proof, room article, private message or federated object.

Profile/update payload maxima exclude SQLite/catalog/index/receipt overhead. No
whole-process/database or 250-member claim follows. Existing eight-session,
5000-ms admission, two credential permits, three owner helpers, <=8-KiB frames and
32-KiB/five-second output bounds remain. No new listener or host action.

All fixtures use synthetic private data and owned marked software state. Retention
checks alter only synthetic timestamps, never host/guest time, NTP or sysctls.
The owned VM/cache are removed after capture; saved playground remains off.
Private argv/keys/profiles/credentials/transcripts are not included here.

## Reproduction

```sh
cargo +1.96.0 test --workspace --release --locked
cargo +1.96.0 clippy --workspace --all-targets --release --locked -- -D warnings
cargo +1.96.0 build --workspace --release --locked
python3 tools/smoke_member_pages.py target/release/faction target/release/faction-member
python3 tools/smoke_backup_recovery.py target/release/faction target/release/faction-sysop
python3 tools/check_member_pages_evidence.py
```

Actual SSH checks additionally require an owned marked disposable NetBSD fixture,
clean installed native build and private argv/profile arguments to
`tools/smoke_shared_member_pages.py`. It refuses ordinary live boards. Keep
profiles/private trust outside Git; do not start the saved playground for this.

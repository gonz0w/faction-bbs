# Local community-wall software evidence

Private software tests, 2026-10-22, macOS ARM64 and a disposable NetBSD 11 ARM64
VM (two cores/1 GiB), Rust 1.96.0. No physical SBC, new capacity, incremental
memory or hardened-host claim. The saved playground remains off.

The [result](results.json) pins 193 selected native execution inputs checked
against the actual guest source bytes, nineteen selected host harness inputs,
unchanged dependency/access/credential inputs and four final installed binaries
matched to the clean native build. All eleven workspace release packages were
cleaned before final compilation, avoiding stale artifacts from copied mtimes.
The installed service also contains the actual new wall success literal; real
restricted SSH/native sessions exercise that installed image.

## Checks

- Rust workspace: 224 tests per OS. Host Rust 1.96 Clippy `-D warnings` and format
  pass. Reviewed registry inventory/features unchanged; new faction-wall is a
  workspace crate using only the existing core/terminal crates. No JS/TS.
- Local/worker wall PTYs: seven per OS, including explicit ASCII review/send,
  sysop naming/limits, cooldown, plain own edit/delete without escapes, compact
  paging, weekly clear confirmation and independent disabled board configuration.
- Actual shared restricted SSH and host/native NetBSD clients: eleven controls,
  including three terminal routes, chronological plain paging, cooldown/Busy
  retry, edit/resize, archives/read-only edit/delete, approval/revocation and
  escape-input rejection. Eight sessions / 5000-ms admission stay unchanged.
- Schema tests cover control/overflow rejection, board/epoch/role boundaries,
  expected-body edit conflict, stable-ID moderation, retention/count limits,
  explicit archives, clock rollback, deleted-request refusal, same-key retry,
  schema-8 read-only export/upgrade and schema-9 wall export/restore.
- Complete wall/file bundle recovery: twelve per OS. Native test uses the actual
  backup CLI with distinct non-root owner/file UIDs. Policy, note and cooldown
  receipt survive complete restoration; ordinary export/verify/restore also pass.
- Existing full-screen/checkpoint PTYs: seven per OS and eleven actual shared
  SSH/native/plain controls. Legacy line desk thirteen per OS, browser ten per
  OS, ordinary local room/menu checks per OS, shared reader 22, native directory
  eleven, launcher seven and operator eight. Artwork: eighteen native previews,
  seven local/worker logins and seven actual SSH/native controls.
- Python tests: 118. Saved-evidence guard 41 protects this new record; all forty
  historical result identities remain unchanged, with 27 exact-source snapshots.

This is bounded functional software validation, not a load benchmark. Wall
payload <=399,360 bytes across current plus twelve retained editions excludes
SQLite/index/catalog/author/receipt overhead. No whole-process/database/250-member
memory claim follows. No public feed, new listener/helper, private-mail plaintext,
autosave, secure erasure or host hardening is introduced.

## Discoveries

An initial local PTY run caught first-keystroke loss: showing a prompt before
entering raw input allowed a later flush to discard quick input. The final local
adapter enters input mode before showing the complete frame/prompt. Another real
SSH run caught an archive navigation error: deleting the final archived note
succeeded but redrawing that absent edition reported not found. Explicit archived
note deletion now returns to current with its success message. Both final flows
were rerun, without retaining failed candidates as passing evidence.

Weekly tests move only synthetic note edition/timestamp rows. They do not alter
VM/host time, NTP, sysctls, penalties or security settings. Native runs use only
owned marked disposable state and loopback-forwarded test SSH, with synthetic
credentials and member-owned native profiles. Private transcripts/keys/paths do
not enter this record. After evidence capture the owned VM/cache are stopped and
removed, and no saved-playground image handle remains open.

## Reproduction

On a prepared Unix developer host:

```sh
cargo +1.96.0 test --workspace --release --locked
cargo +1.96.0 clippy --workspace --all-targets --release --locked -- -D warnings
cargo +1.96.0 build --workspace --release --locked
python3 tools/smoke_wall.py target/release/faction target/release/faction-member
python3 tools/smoke_backup_recovery.py target/release/faction target/release/faction-sysop
python3 tools/check_wall_evidence.py
```

Native reproduction requires an owned, explicitly marked disposable NetBSD VM,
clean native build, restricted member gateway and privately supplied arguments to
`tools/smoke_shared_wall.py`. Its marker/target checks refuse ordinary live boards.
Do not restart the preserved playground or paste private profiles into Git.

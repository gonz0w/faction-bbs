# 0011: Private local export and non-overwriting restore

Issues: [#18](https://github.com/gonz0w/faction-bbs/issues/18), [#19](https://github.com/gonz0w/faction-bbs/issues/19), [#35](https://github.com/gonz0w/faction-bbs/issues/35)
Status: published for review in [PR #51](https://github.com/gonz0w/faction-bbs/pull/51), 2026-10-20.

## Goal and constraints

Owner requested merge #50 and continue with export and restore. #50 merged as
642d88387da032f612358d28fcfd5007e77e457d; start from current main. Build owner-only
local data movement without a listener, external service, JS/TS or new crate.
Preserve accounts/approval, room posts/replies/retry IDs/unread, channel definitions,
grants and retained history. Do not restore live chat leases or transplant session
capabilities. Never overwrite an existing board or export; keep paths owner-private.
Themes/art live outside the database and need a separate operator copy.

## Progress

- [x] Read product, privacy, workflow, local-core/chat docs and issues #18/#19.
- [x] Check CI/head and merge requested PR #50.
- [x] Implement versioned SQLite snapshot, read-only export, verify and new-directory restore.
- [x] Validate live WAL consistency, durable roundtrip, old-session denial, malformed
  input, permissions, no overwrite, concurrent publication and real CLI workflow.
- [x] Record decision, setup/limitations, dependency feature inventory and current status.
- [x] Open/attach focused PR and update broad issues without closing them.
- Required CI is tracked by the PR check on its current head; verify it before merge.

## Steps and commands

Use Rust 1.96.0 and existing native C toolchain, Python stdlib tooling. Execute
from repository root, put build output and generated private fixtures outside it:
`cargo +1.96.0 fmt --all -- --check`, Clippy with `--workspace --all-targets
--release --locked -- -D warnings`, `cargo +1.96.0 test --workspace --release
--locked`, release build, existing console/chat PTY smoke and new export/restore
smoke, repository/Python checks and selected dependency scan.

## Discoveries and decisions

- SQLite online backup reads committed WAL state without copying a live main file.
  Copy into a private staged file, validate exact current catalog/data, delete
  transient leases, mark format 1, compact to a standalone DELETE-journal file.
- Read-only export must not initialize/migrate on a typo or older source version.
- No-overwrite hard-link publication is available on Unix/NetBSD without a new
  dependency or platform-specific unsafe rename syscall; stage in the destination
  parent filesystem. Restore reserves a new directory only after validation. Do
  not open that path concurrently during publication.
- Rotate the restored board's internal session-binding nonce; preserve durable IDs
  and hashes. A snapshot can roll approval/password policy back: review before use.
- Add existing rusqlite backup/hooks/limits features only; no new registry packages.
- Initial cap 256 MiB and cooperative 30-second work budget; these are tooling
  limits, not supported capacity, guaranteed wall-clock deadlines or RAM bounds.

## Validation and outcomes

Rust 1.96.0 on the ARM64 Darwin developer host:

- Formatting and release Clippy (`-D warnings`) passed.
- All 40 existing Rust tests passed; ten new recovery integration tests passed.
- Release build and all three real PTY workflows (console, simultaneous chat,
  export/verify/restore) passed. Generated credentials/state are disposable and
  never printed or committed; no listener/public provider was used.
- All 23 Python tests and repository/link/policy/archive checks passed.
- Selected Linux/NetBSD dependency scan passed. Reviewed inventory diff adds only
  rusqlite backup/hooks/limits features; same 39/41 registry packages and licenses.
- `git diff --check` passed. Required GitHub CI will repeat checks on the PR head;
  GitHub check state is authoritative and is not inferred from these local results.

Recovery cases: ten tests cover committed WAL consistency/concurrent writes,
durable IDs/replies/retries/unread/status/grants, old capabilities denied after
restore (including rollback to earlier approvals), unchanged input, removed chat
receipt bytes, private paths/permissions/sidecars, malformed/foreign/future schema,
extra executable catalog objects, bad FK/content/KDF data, size cap, competing
restore publication, missing/older source no-create/no-migrate and busy-store
failure followed by successful retry. A real CLI rehearsal exports while a member
is in chat, restores and logs in, and confirms pending denial and no overwrite.

Host-specific discovery: SQLite NOFOLLOW also rejects trusted macOS `/var`/`/tmp`
aliases in parent components. Canonicalize already-checked trusted parent directories
before opening while preserving no-follow for the final file. A zero trigger-depth
cap blocked required FK cascades; use eight with exact catalog rejection of extra
SQL triggers. Both failures were reproduced and the focused tests passed afterward.

Remaining: native NetBSD application, power-loss/filesystem fault injection,
remote-safe operator separation, encryption/authentication, member-scoped exports,
incremental/scheduled backups and theme/file-area packaging remain outside this PR.

## Review handoff

PR #51 is attached to the project chat. Issues #9/#18/#19/#35 have progress and
remaining-gate comments; none is closed by this bounded increment. Branch
`codex/18-export-restore` starts at the merged #50 main revision. No production
deployment or automatic merge of #51 is implied. The next build should focus on
restricted SSH/identity separation and native NetBSD application validation, with
remaining lifecycle/resource/failure work tracked in the existing issues.

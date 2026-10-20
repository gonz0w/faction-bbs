# 0018: Complete the shared terminal UI and measure its headroom

Issue: [#55](https://github.com/gonz0w/faction-bbs/issues/55), related #28/#20.
Status: implemented and validated; focused PR review pending. PR #60 merged at 76abb2f.

## Goal and constraints

Build the existing local member features into a bounded shared owner: approved
identity, sysop packs/manual seasons, ANSI arrows/Enter/hotkeys, plain/resize,
chronological room pages/replies/retry IDs, local chat/editor/grants, role-checked
sysop actions. Keep two credential operations, kernel locks and base OpenSSH/UID
boundaries. Measure actual NetBSD software, including all 250 SSH transports,
under the unchanged headroom guard. Normal eight-worker services stay unchanged.

## Progress

- [x] Read issue, product requirements and previous evidence; merge #60.
- [x] Implement separate shared UI candidate and bounded scheduling/output.
- [x] Run native parity, approval, escape, fault and 250-member checks.
- [x] Update docs, save sanitized evidence and open a focused PR.

## Steps and commands

Use `experiments/055-shared-ui` with a unique package and isolated target directory.
No root crate or frozen experiment changes: existing source/evidence pins remain
valid without fabricating old measurements or prematurely adding source archives.
Run pinned Rust 1.96 fmt/clippy/test/build and reviewed dependency inventory.
In a signed disposable NetBSD 11 ARM64 2-vCPU/1-GiB VM, build native binaries,
exercise real restricted SSH and UI parity, then repeat the matched #60 workload
with full themes and 250 distinct members. Save installed binary/source identities,
failures, complete memory samples and cleanup. Never publish keys/raw transcripts.

## Discoveries and decisions

2026-10-20: Full UI can reuse the unmodified terminal/core libraries in a separate
candidate. Theme/art load once for the shared owner, not once per connection.
Per-connection output is lazy, capped at 32 KiB after CRLF encoding (safe art may
render 16 KiB); lists page ten rows instead of dumping 100. Credential permits
retain #60's entire queued/active/result lifetime. Round-robin sweeps have a global
work budget; each connection consumes bounded input and a single submitted action.
No historical claim is refreshed to new source bytes.

## Validation and outcomes

Pinned Rust fmt/Clippy/test/build and reviewed dependency checks pass. Native
workspace tests: 58; dispatch: one; full shared UI: 27; normal actual SSH: 20;
lifecycle: 13; minimal frontend contract: 20. Three distinct fresh-boot 250-member
workloads pass all sampled guards with 180.39–180.51 MiB completed-stage
headroom and 135.09 MiB lowest stress margin. Full UI actions, actual member/
owner credentials and calibrated controls are sampled. All historical evidence
checkers still pass without changing frozen hashes. Complete source/binary/counter
ledgers, exact original-settings rollback, integrity and private-fixture cleanup
are recorded in results 033. No production or supported-capacity claim; normal
service/admission/confinement and broad active-workload gates remain #55/#20/#28.

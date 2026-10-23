# 0071: File request desk and fulfillment

Issue: [#140](https://github.com/gonz0w/faction-bbs/issues/140)
Status: implemented and validated; focused PR ready

## Goal and constraints

Complete the local request/create/view/fulfill/close/moderation portion alongside
merged #168 shelves. Preserve existing catalog/upload/grant/SFTP authority, classic
fallback, normal budgets, no JS/TS and device-only private-mail keys. Keep owner
playground off/intact. Defer hardening, external retrieval and federation.

## Progress

- [x] Merge #168 after its Documentation and policy success; no unresolved threads.
- [x] Preserve 292 prior executed/harness inputs and original library result as
  snapshot 38 before changing runtime/fixtures; never rewrite historical hashes.
- [x] Implement schema-15 bounded core requests, revision checks and filtered offers.
- [x] Add common optional library request desk and explicit confirmation screens.
- [x] Complete host/native terminal, SSH/native transfer and complete-recovery checks.
- [x] Update evidence/CI/docs and run all 62 maintained offline policy/evidence
  commands, full Rust/terminal workflow and source preservation checks.
- [ ] Verify the committed source archive and open the focused PR closing #140.

## Validation and decisions

See ADR 0066 and docs/file-requests.md. Use `cargo fmt`, strict workspace clippy,
release workspace tests/build, `tools/smoke_file_requests.py`, library regressions,
`tools/smoke_shared_file_requests.py`, complete recovery and maintained repository
checks. Native testing uses only a separately marked disposable NetBSD 11 ARM64 VM
(2 cores/1 GiB), normal eight sessions/five-second admission. No physical hardware,
new capacity/public endpoint/hardening or private-mail runtime claim.

Initial fixture failures: current-schema assertions needed 15; the local worker
uses an ANSI menu even at 40 columns, so its oracle must follow actual menu prompts.
These are corrected in current fixtures; historical result identities stay fixed.


Final validation: 321 Rust tests per OS; 130 host Python tests; six request and
eight library local/worker PTYs per OS; eight actual shared SSH/native/SFTP controls;
18 complete recovery checks per OS. Native final rebuild is clean/offline across
all 18 packages; actual installed service/owner/sysop hashes match final build.
The 34-byte original synthetic request fulfillment downloaded through restricted
SFTP with exact SHA256; native zero-budget deferral created no grant. Scope and
source pins are in the new evidence record; all old records are unchanged.

An additional polish during review made long title/description/rights/provenance
reviews pageable in full and blocked YES before the final page, with resize tests.
No member content is silently truncated into a confirmed write. The initial shared
shelf regression also passed ten controls; the final current record claims the
new eight-control request journey against the final rebuilt/installed binaries.

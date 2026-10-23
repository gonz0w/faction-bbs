# 0072: Local magazine reader and original Nightwire issue

Issue: [#143](https://github.com/gonz0w/faction-bbs/issues/143)
Status: implemented and validated; [PR #170](https://github.com/gonz0w/faction-bbs/pull/170) open

## Goal and constraints

After checked #169/#140, add a themed local magazine reader, original five-article
starter edition and owner preview/text export. Keep full member submission queues,
authenticated contributor approvals/editor roles and automated scheduling open
under #143. Preserve classic/plain, current local approval/file policy, no JS/TS,
normal budgets and the stopped owner playground; defer later hardening.

## Progress

- [x] Merge #169 after successful checks and no unresolved review threads.
- [x] Freeze 298 previous execution/harness inputs plus whole request result as
  source snapshot 39; all 52 old result identities remain unchanged.
- [x] Implement bounded published-only magazine/contents/article/credits desks,
  optional existing Files/discussion jumps and owner-only preview/new-file export.
- [x] Add an original five-article Nightwire edition under BSD-3-Clause.
- [x] Complete host/native/shared terminal, permission, recovery and Rust workflow checks.
- [x] Record source-pinned evidence and remove owned VM/private inputs/vendor;
  saved owner playground remains stopped/intact.
- [x] Verify the committed source archive with all 63 offline policy/evidence
  commands and open PR #170 (refs #143); GitHub CI is running.

## Validation and discoveries

Use strict clippy, release workspace tests/build, `smoke_magazine.py`, existing
request/library/social/reader/vault regressions, `smoke_shared_magazine.py`, complete
recovery and maintained repository checks. Native tests use only a marked owned
NetBSD 11 ARM64 software VM (2 cores/1 GiB); ordinary 8-session/5000-ms defaults.
See ADR 0067 and docs/board-magazine.md for exact scope and retention limits.

Initial implementation checks caught a local function/name shadow and OsString
conversion, both corrected before executable testing. A credits assertion assumed
an unwrapped phrase; the oracle now follows actual bounded text wrapping. No
historical evidence was refreshed to new source bytes.


Final measurements: 327 Rust tests per OS, 130 Python, seven magazine PTYs per OS,
nine actual shared SSH/native/SFTP controls and eighteen complete recovery controls
per OS. Native actual owner text export supplied the exact 7,673-byte edition
subsequently downloaded by the compact native guest with SHA256 verification.
Published filtering, current approval/area grants, zero-budget deferral, normal
room acknowledgment, stopped withdrawal/revision and explicit owner preview/export
all pass. The record matches 294 native/36 harness inputs and final installed
hashes; normal budgets/schema/dependencies remain. Full submission/editor/verified
contributor workflows remain open under #143; no new provider/public service.

Final review caught a forward-reading stop at the 32-page rewind capacity. Forward
reading now evicts the oldest back position; a maximum 8-KiB/120-column-line
article reaches its final marker on a 20-column terminal, then bounded rewind
still works. Rebuilt/retested the final sources in a new owned software clone.

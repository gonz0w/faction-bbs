# 0058: Explicit multiline room writing and preview

Issues #141/#21/#12, supporting social hangout #133.
Status: implemented and software validated; ready for review.

## Goal and constraints

After checked #147 (head cdf701c, integration merge 687c257), add an opt-in
`/edit` path at the existing room-body prompt. Provide bounded multiline text,
undo/clear, current-authorized reply quoting, paged inert preview and explicit
`.send`/`.cancel`, shared across local/worker and shared-owner sessions. Keep the
one-line shortcut and existing idempotent submission/unread rules. This is the
first writing-desk slice, not completion of #141: caret/full-screen editing and
recoverable drafts require a later design and implementation. Preserve all
private-mail device boundaries, default budgets and existing core/schema.

## Progress

- [x] Merge checked #147; freeze 174 actual prior inputs and whole result in
  snapshot 25, sources/pre141-writing-desk / room-browser-baseline-manifest.json.
- [x] Shared bounded draft commands and existing reader-based inert preview.
- [x] Local/worker and shared-owner room routes; explicit send/cancel.
- [x] Unit tests and private PTYs: send/preview/cancel, quote, UTF-8/controls,
  limits, resize, disconnect/reconnect, approval and retry behavior.
- [x] Clean native NetBSD build, real restricted SSH/native routes and reader,
  browser, operator/art regressions on an owned disposable software clone.
- [x] Pin actual new execution inputs/results, stop/remove owned clone/cache,
  verify saved playground remains off, run all repository/evidence checks.
- [x] Update README/docs/AGENTS/changelog, issue status and focused PR.

## Decisions and remaining work

Drafts contain local room plaintext only, held in one connection; no disk store,
new table/listener/helper/dependency or private mail. Disconnect loses the draft.
Undo/clear and errors do not submit. Preview/count/quote do not mark messages read.
Room/parent and current approval are checked on each desk action; only `.send`
creates a request ID, then reuses existing same-request busy retry. A quote uses
fresh permission-filtered parent text; a whole quote must fit or nothing changes.
Limits remain 1024 stored body bytes / at most 32 draft lines. Classic text stays
inert and available. Saved playground stays off. Hardening remains later.

## Validation and outcomes

Final `cargo test --workspace --release --locked`: **199 tests per OS** on macOS
ARM64 and native NetBSD 11 ARM64 (Rust 1.96). Host `cargo clippy --workspace
--all-targets --locked -- -D warnings` and `cargo fmt --all -- --check` pass.
`python3 -m unittest discover -s tools -p 'test_*.py'`: **118 tests**.

`tools/smoke_writing_desk.py`: **13 local/worker controls per OS**. Actual marked
NetBSD `tools/smoke_shared_writing_desk.py`: **18 restricted SSH/native/plain
controls**, including both host/native device clients, selected room entry,
no-submit preview/quote, zero unread rows, caps, resize, reconnect, pending/
revocation and exactly-one-row same-key busy retry. Full local account/menu and
reader regressions pass per OS; browser **10 per OS**; shared UID/PTY reader **22**.
Native directory **11**, launcher **7**, operator **8**, artwork previews **18**,
local/worker art **7**, actual shared SSH/native art **7** pass.

The final native workspace was cleaned/rebuilt; all **174 selected native inputs**
match copied bytes. The record also pins **11 host harness inputs**, final installed
binaries and **32 unchanged core/build/frontend inputs**. A formatted test was
recopied/re-executed in the final clean suite. Initial long-preview/registration
failures were harness expectations; paging/shared prompt corrections precede the
accepted fresh-fixture run. Quotes now validate total size/line count before any
line-copy allocation. No runtime limits were relaxed.

See [the evidence](../evidence/writing-desk-netbsd/README.md). No measured memory
delta, new 250 capacity, physical SBC, wire adapter, hardening or complete #141
claim. Full-screen caret/paragraph editing, configurable labels, saved drafts and
the remaining social #133 features stay open. Existing gameplay is preserved.

All **49 offline policy/evidence/Python commands** pass, including all 39 saved-
evidence guards. Owned VM/cache removed; saved playground image has no open
handle. Format/diff checks pass. No current Cargo manifest/lock changes.

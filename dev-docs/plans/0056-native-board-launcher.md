# 0056: Member-owned native board launcher

Issues #27/#21/#14. Status: implemented and software validated; ready for review.

After checked #131 (head ef5981d, merge 5fea2cd), make visiting independently
configured boards convenient in the native terminal client. Preserve ordinary
`--profile` behavior and keep server output unable to pick a local endpoint/key.

## Steps

- [x] Merge checked #131; freeze 174 prior source/harness inputs and whole result
  in snapshot 23, sources/pre27-launcher and directory-baseline-manifest.json.
- [x] Optional private `--boards` catalog: <=16 KiB / sixteen labels and absolute
  local profile paths, no remote import, profile validation on every selection.
- [x] Paged arrow/hotkey local picker and explicit connection confirmation.
  Return after disconnect/failure; Ctrl-] then B asks before discarding a session.
  Double Ctrl-] forwards the existing classic-door return byte. Single-profile
  mode stays byte-transparent; Files remains its separate foreground operation.
- [x] Keep one SSH child/session, fresh filters/nonces/recent links per visit,
  fixed SSH policy/accounts, pre-enrolled host keys and independent BBS login/
  approval. No resume/stacked sessions, profile mutation or key forwarding.
- [x] Host/native Rust, PTY navigation/input/host-key checks and actual two-board
  SSH/native software proof. Keep existing directory/art/file behavior checked.
- [x] Update docs/README/ADR/changelog, save new immutable evidence, remove only
  owned temporary VM/cache and open focused PR. Hardening remains later.

## Bounds and validation

Catalog labels/IDs are bounded ASCII and paths use existing profile rules. User
must configure each board and verify its host keys out of band. A directory card
is editorial information and never a trusted local profile. Local keyboard alone
opens/chooses/confirms the launcher; remote text/OSC/graphics cannot select hosts.
No server/runtime/schema/default/dependency change is required. Test 40x18 and
80x24 pages, unknown/wrong keys, canceled choices, explicit disconnect, doubled
escape passthrough and independent pending/approved board sessions. Pin actual
executed inputs; do not relabel old memory results or claim new 250 capacity.

## Outcomes

[Actual software evidence](../evidence/board-launcher-netbsd/README.md): 191 Rust
per OS, 118 Python, seven launcher PTY controls per OS and ten real two-board
controls, plus directory/graphics/operator/art regressions. Two independent
2-core/1-GiB VMs retain the production fixed socket and distinct member host keys.
168 selected native inputs matched; eight host harness inputs are separately
pinned. All 110 checked server/workspace inputs remain unchanged.

The actual cancel/immediate-retry sequence caught a local flush race, fixed before
final runs. The fixture needed native NetBSD process-column syntax and a restarted
restricted listener after rebuild. No production policy/default was relaxed.
Private failure context is retained; only the final candidate has a passing record.
One active client SSH child per OS / four LWPs per board were observed, without
a new memory/capacity claim. Both disposable clones/cache are removed; saved
playground stays off. Member submissions and directory-to-profile import remain
follow-on work; encrypted identity/mail and hardening remain separately scoped.

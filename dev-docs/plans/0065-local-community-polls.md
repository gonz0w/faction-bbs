# 0065: Local community polls

Issue: [#139](https://github.com/gonz0w/faction-bbs/issues/139)
Status: implemented and validated; ready for PR review. Owner: merge #162 and continue useful product work.

## Goal

Build a bounded default-disabled local voting desk: create/edit before any ballot,
explicit single-choice vote/change/withdraw, close/cancel, sysop moderation and
safe text-cell results with plain/ANSI fallback. No election-grade/secrecy claim,
external service, federation, new registry package or changed resource defaults.
Keep private mail device-side, hardening later and the owner playground stopped.

## Progress

- [x] Merge checked #162; select/read #139 and preserve previous measured inputs.
- [x] Add schema-12 core, current approval/revision/visibility and owner recovery.
- [x] Add shared local/worker/service workflow, host-optional menu and sysop policy.
- [x] Validate ordinary voting/closure/privacy/concurrency and native NetBSD paths.
- [x] Update product/README/ADR/AGENTS/changelog and prepare the checked PR.

## Scope and defaults

One question per poll, 2–8 distinct ASCII choices, one ballot/member. Multiple
selection and grouped questionnaires stay future work. An open poll permits
explicit vote changes/withdrawal; the first ballot permanently locks wording,
choices and result policy, even after withdrawal. Closing/cancelling is final.
Choose always/after own vote/after close results; no author/sysop display bypass.
All ballots are plaintext local owner data; no voter-name list is exposed.

Sysop-only creation by default, opt-in approved-member creators; 32 retained polls,
four active polls per ordinary creator and 250 retained ballots/poll. Manual
close/cancel/removal, no automatic time expiry or scheduler. Explicit exact DELETE
removes the poll and ballots. Trusted host-only V menu, S/P policy, inherited
palette and bounded ASCII charts. Snapshot 32 freezes the preceding gathered source/
harness bytes and whole result; all 45 historical saved guards remain unchanged.

## Validation

268 Rust tests per OS /129 Python; eight poll PTYs per OS and twelve actual
restricted SSH/native/plain controls pass. Complete recovery has fifteen controls
per OS, including the actual native distinct non-root owner/file-UID CLI. The final
clean native source matches 230 selected inputs, with nineteen host harness inputs
and fifteen unchanged access/credential inputs pinned. Existing social/editor/art/
menu/client regressions pass. A host-capability test caught a declared poll action
remaining enabled after disable; it now requires the runtime flag. Choice-page
clamping is tested across repeated Next/Previous and resize. Stale choices are
rejected before indexing during review. All 45 prior guards stay intact; the new
record adds guard 46. The owned VM/cache/credentials are removed; the owner's
playground stays stopped with disk/login/keys preserved.

Schema 12 requires stopped/drained verified complete pre-upgrade backup;
old schema-2..11 exports remain read-only verifiable/restorable. Rollback uses the
matching older binary and complete older backup in fresh paths.

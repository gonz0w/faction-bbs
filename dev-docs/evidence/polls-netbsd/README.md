# Local polls: NetBSD software validation

Issue #139, [GitHub issue](https://github.com/gonz0w/faction-bbs/issues/139),
[ADR 0060](../../decisions/0060-local-community-polls.md).

Validation uses owned synthetic boards, host Rust 1.98.0 and a separate NetBSD
11.0 AArch64 software VM (two cores/1 GiB RAM, Rust 1.96.0). The retained owner's
playground remains stopped and preserved. The final clean native workspace build and host build pass **268 Rust tests per
OS**, plus **129 Python tests**. Eight local/worker poll PTYs per OS cover settings,
question/choices/notes, preview/save, repeat/change/withdrawal, permanent wording
lock, plain/compact charts, close/cancel/delete and pending denial. A navigation
control checks repeated Next at the final choice page followed by one Previous.

Twelve actual restricted SSH/host-native/NetBSD-native/plain controls cover result
visibility for two members, no voter-name display, stale edits, Busy ballot retry,
resize, hiding, active revocation, closure races and explicit deletion/disable.
A direct workflow control rejects a stale reviewed choice before indexing the
current choices. Host-only menu tests prove disabling cannot be bypassed by a
previously declared poll action, including crowded 40/80-column menus.

Complete recovery passes **15 controls per OS** with poll settings, choices,
ballots and the permanent wording lock alongside old wall/pages/gatherings/files.
The native run uses the actual CLI with distinct non-root owner/file UIDs. Export/
verify/restore also passes. Native gatherings/pages/wall/full-screen/writer/browser/
operator/launcher/directory/menu/reader/console/Nightwire regressions pass, including
18 artwork reference profiles. Existing local encrypted vault (13) and picker (8)
controls pass on both OSes.

`results.json` pins **230 matched native source inputs** and **19 host harness/
review inputs**. Fifteen dependency/access/credential inputs remain unchanged.
Installed faction/service/sysop binary hashes match the final build; the new
ballot literal is checked. The owned disposable VM, temporary vendor and test
credentials are removed. The owner playground stays off, with disk/login/keys
preserved. `python3 tools/check_polls_evidence.py` verifies this saved record;
the historical/current suite now has 46 saved-result guards.

Snapshot 32 freezes 228 preceding measured/harness inputs and the whole gatherings
result. Historical hashes/results are unchanged. This batch does not measure 250
connected users or physical SBCs. Normal eight sessions/5000-ms admission, two
credential permits, three helpers, 32-KiB output queue and five-second drain remain.
Polls are ordinary plaintext local ballots, not secret or election-grade voting.

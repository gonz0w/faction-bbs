# Local gatherings: NetBSD software validation

Issue #136, plan [0064](https://github.com/gonz0w/faction-bbs/issues/136),
[ADR 0059](../../decisions/0059-local-gatherings.md).

The record in this directory covers the optional local agenda, UTC scheduling,
counts-only RSVPs, login reminder, moderation and deliberate local room/chat jumps.
Validation uses synthetic private boards and a distinct disposable NetBSD 11.0
AArch64 software VM configured with two cores and 1 GiB RAM. The retained owner
playground stays stopped and preserved. No physical SBC or real Ghostty-pixel test.

Current workspace: thirteen packages, no new registry dependency. The actual native
source bytes are compared with the selected source hashes after a clean release
workspace build. Exact source/harness pins and result counts belong in results.json;
no historical record is refreshed. Snapshot 31 preserves the preceding 217 inputs
and whole client-vault integration result before edits.

The final clean build passes **256 Rust tests on each OS** (host Rust 1.98.0;
native NetBSD Rust 1.96.0). Python: 129. Eight local/worker agenda controls per OS
cover settings, explicit preview/save, repeat/change/clear RSVP, plain/compact
reminders, room/chat jumps, cancellation/removal and pending denial. Eleven actual
restricted SSH/host-native/NetBSD-native/plain controls cover two-member counts
privacy, stale revisions, Busy retry, resize, hide and active revocation. Read-side
expiry cleanup yields under a competing WAL writer, retaining preview/retry UI;
core tests also cover past/expiry, quotas and schema-10 read-only recovery.

Complete backup: fourteen controls per OS, including gathering policy/event/RSVP
recovery alongside wall/page/hidden plan/files. Native runs use the actual CLI with
distinct non-root owner/file UIDs. Export/verify/restore also passes. Native wall,
member pages, full-screen checkpoints, writer, browser, launcher, directory,
operator, classic menus/reader/console and original Nightwire regressions pass;
18 artwork reference profiles are checked. The encrypted local vault (13 controls)
and local picker (8) pass on both OSes, without network mail or roaming claims.

`results.json` pins **220 compared native source inputs** and **17 selected host
harness/review inputs**, with fifteen dependency/access/credential inputs unchanged.
Installed faction/service/sysop binaries match final build hashes; the service's
new gathering literal is checked. The owned disposable VM, temporary vendor and
test credentials are removed; the stopped owner playground disk/login/keys remain.
All forty-four historical guards remain, with this record adding guard forty-five.
Run `python3 tools/check_gatherings_evidence.py` for the offline saved-result guard.
The tests exercise bounded hobby-board flows, not 250-user load or a supported
capacity claim. Eight sessions, 5000-ms admission, two credential permits, three
owner helpers, 32-KiB output queue and five-second drain remain unchanged. No
external calendar, message federation, private-mail server decrypt or hardening
adoption is implemented by this batch.

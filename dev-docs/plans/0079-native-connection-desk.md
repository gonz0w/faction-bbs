# 0079: Device-local connection desk

Issue: [#158](https://github.com/gonz0w/faction-bbs/issues/158)
Status: implementation/software validation complete; publication checks in progress

## Bounded outcome

Make saved-board setup understandable before connecting. The local Boards picker
gets I:connection desk, with selected-board readiness, separate terminal/Files
endpoints, deliberate stored-pin lookup, manual recovery guidance and a redacted
share preview. `--check-profile FILE` offers the same local readiness summary on
plain/non-TTY output. Keep host identity, key unlock, SSH admission, local BBS
approval and future invitation/route checks explicitly separate and unknown.

This is the existing-profile slice of #158, not live route probing, verified
contacts, Tor authorization or remote authentication. No network request, automatic
host-key enrollment, fallback route, profile edit, saved report or visit history.
No server/schema/dependency/helper/default changes. Server classic/plain fallback
continues; the native picker remains ANSI/TTY, with a plain local diagnostic CLI.

Use existing profile ownership/size/no-follow rules and strict TOML. Separate
parsing from file readiness without changing Profile::load admission semantics.
Private-key readability cannot prove unlock or matching private/public key halves.
Re-read the chosen profile before any actual connection as before.

Only explicit K invokes base `/usr/bin/ssh-keygen -F`, sequentially for terminal
and Files, using a checked host file descriptor as stdin. No shell/ssh/login,
network or stderr display. Two-second deadline and 32-KiB output ceiling per
lookup, at most four distinct fingerprints per endpoint, cancellation and reap on
all outcomes. Host fields/comments/controls never render as lookup output.
CA/revoked entries are policy hints, never an admission badge. OpenSSH enforces
actual host trust on connection. Readable files are not verified contact identity.

## Progress

- [x] Merge checked #175; update dependent #177 against current main.
- [x] Preserve snapshot 46: all 338 Who inputs and its unchanged result identity.
- [x] Implement local panel/CLI, independent readiness and fixed bounded pin helper.
- [x] Merge #177 after its updated exact-head check passes.
- [x] Validate real PTYs, strict Rust checks and owned NetBSD client software.
- [x] Record exact sources/binaries and offline evidence; update user docs/status.
- [x] Remove owned clone/cache; retain stopped owner playground.
- [ ] Complete checkout/archive checks and publish the focused PR.

## Validation scope

Run targeted client tests/clippy plus maintained full host Rust/Python workflow;
new PTYs cover 40x18/paging/selection, plain report, redaction, missing/malformed
files, hashed/distinct/empty pins, output bounds, cancellation and loopback no
network traps. Run native client tests and relevant launcher/travel/vault checks
on a separately marked NetBSD software clone. Server sources remain unchanged
from #177; record that identity rather than claiming a new server capacity run.
Keep prior measured sources/results immutable and all offline/archive checks.
Hardening and physical hardware remain later.

## Results and corrections

#175 merged at d4f86be and #177 at 545f9d9 after current-main exact-head checks
passed. Main's post-#177 tree exactly matches this batch's starting 784cb3d tree.
No history rewrite is needed to join that reviewed baseline.

Strict workspace clippy, 361 host Rust tests and 130 Python tests pass. The changed
client passes 21 tests and fourteen new PTYs on both macOS and an owned NetBSD
11 ARM64 clone, plus launcher seven/travel six/vault-picker eight/local-vault
thirteen regressions per OS. Three host native-graphics controls also pass. All
67 selected native source/harness identities match, the final binaries are pinned,
and 317 non-client inputs remain identical to Who. No new server load/capacity
run is claimed. Normal eight-session/credential/three-helper defaults remain.

The first frozen native attempt found an empty private vendor cache; populated it
from the unchanged lockfile and rebuilt successfully. Source collection initially
included an untracked macOS .DS_Store, which is neither uploaded nor compiled; the
final set excludes it. The cleanup helper assumed unversioned package directories
and stopped after shutting down the owned VM; explicit stopped-PID/port checks and
versioned-package validation completed cleanup before recording teardown success.
Saved playground file hashes/disk metadata remain unchanged and it stays off.
A private offline-runner extraction initially selected the checkout step; the
correct named policy block is used for final checkout/archive checks. These are
harness corrections, not product or dependency changes.

Snapshot 46 preserves all 338 previous execution/harness/dependency bytes and the
whole Who result. Guard 60 verifies the narrow current record. Complete all 70
maintained offline commands in the checkout and committed source archive before
publishing. #158 remains open for verified-contact age/trust, route/invitation
adapters and deliberate live probing; this batch is local existing-profile setup.

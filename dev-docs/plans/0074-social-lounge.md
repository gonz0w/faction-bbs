# 0074: Opt-in local chat availability and polite pages

Issue: [#137](https://github.com/gonz0w/faction-bbs/issues/137)
Status: implemented and software-validated; PR preparation

## Goal and constraints

After checked/squash-merged #171, make local chat a more useful hangout: explicit
available/busy/away/hidden presence across local chat channels, bodyless page
requests and accept/decline, mute, blocks, UTC quiet hours and opted-in sysop paging.
No forced switch, private-message text, last-seen public log or OS authority.
The first batch operates only during a live chat entry; board-wide presence and
notifications outside chat remain #137 follow-on work. Keep classic/plain,
normal budgets, stopped owner playground and later hardening deferred.

## Progress

- [x] Merge #171 after its successful check and resolved review conversations.
- [x] Preserve 310 previous execution/harness inputs as source snapshot 41; all
  54 historical result identities remain unchanged.
- [x] Add schema-17 bounded live presence/invitations and durable personal controls.
- [x] Complete core permissions/expiry/privacy and shared command tests.
- [x] Validate local/worker/shared SSH/native NetBSD and complete recovery.
- [x] Record exact executed bytes, remove the owned clone/private inputs/vendor.
- [x] Complete maintained checks and exact committed-source archive verification.
- Open a focused PR linked to #137, retaining outside-chat presence/notifications.

## Validation and discoveries

Use clean native and host release workspace tests/build, strict clippy, maintained
Rust/PTY/Python/offline checks and actual marked disposable NetBSD 11 ARM64 software
journeys. Pages have no custom subject/body. Availability is explicitly ephemeral;
quiet hours/mute/block intent survive reconnect and backups. Page rate timestamps
are temporary, separate from durable preferences, and removed from exports. No
new listener/helper/package, capacity, hardware or hardening claim.

The repository permits squash merges only; #171 merged using its exact checked
head after the merge-commit method was refused. No merge policy was bypassed.

The first native build passed all Rust tests but a prior writing-desk PTY still
asserted schema 16. Updated that assertion and the preview-seed companion to 17;
the final clean native build reruns the maintained workflows. No historical
result or preserved fixture source was changed.

Final software observations: 345 Rust tests per OS, 130 Python tests, eight social
local/worker PTY controls per OS, seven actual restricted SSH/native social
controls and twenty complete-recovery controls per OS. Existing
editorial/library/writing/art/vault/travel regressions and the maintained Rust/PTY
job pass. Some expiry/cooldown tests deliberately age owned fixture timestamps,
rather than claim measured wall-clock or supported capacity.

The social-netbsd record matches 307 selected native inputs, 40 harness inputs
and thirteen unchanged access/credential sources; final installed binaries match
the tested clean build. Normal resource limits/packages remain. The owned
software clone, test credentials and temporary vendor were removed; the owner
preview remains stopped and privately preserved. Offline guard 55 validates
this current software record while historical identities stay fixed.

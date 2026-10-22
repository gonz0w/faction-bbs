# 0052: Local operator status and diagnostics batch

Issues: [#95](https://github.com/gonz0w/faction-bbs/issues/95),
[#18](https://github.com/gonz0w/faction-bbs/issues/18),
[#98](https://github.com/gonz0w/faction-bbs/issues/98)
Status: implemented; [PR #128](https://github.com/gonz0w/faction-bbs/pull/128) open, GitHub checks pending

## Goal and constraints

After merging passing #127, group the first local sysop dashboard with operations
visibility and read-only diagnostics. Provide an explicit OS-authorized local
command, themed ANSI/plain panels, bounded read-only aggregate snapshots, stale/
unavailable labels, keyboard help/pause/refresh/exit and modest sampling. No member
shell/host privilege, message bodies/keys, host applies, new owner helper/listener,
capacity/default-session change or automatic operator login. Hardening stays later.

## Progress

- [x] Read related issues; merge #127 only after exact-head checks succeeded.
- [x] Preserve all 176 measured Nightwire source/harness inputs and two whole
  records in nineteenth snapshot `sources/pre95-operator` before runtime changes.
- [x] Add opt-in owner-produced bounded status and local read-only console.
- [x] Validate actual service/console/software NetBSD behavior and costs.
- [x] Update operator docs, README, #95/#18/#98 and focused PR #128; attached to task.

## Design and sequence

Service config explicitly enables a private one-way snapshot in its existing
owner-only state directory. It includes aggregate counts and bounded live session/
credential/output/door state, without identities/content or a new listener.
Aggregate SQL has an instruction/time budget and can report unavailable. Dashboard
reads only the private regular snapshot, never opens/migrates the live database.
No BBS account/session/role authorizes access: Unix owner/root permissions do.

Use the existing CLI/terminal dependencies and a separate local `faction-sysop`
binary, not the restricted member binary. Sample at modest intervals with bounded
history; pause sampling and offer plain one-shot output. Host data is local and
separately marked; any unavailable network/peer/backup metrics stay explicit.
Optional dedicated-operator launch is documented only, preserving OS login/shell.

## Discoveries and decisions

2026-10-22: #127 check SUCCESS verified twice at head 7e658c1; authorized squash
merge 8f04aae. An initial approval review used stale check state; the guarded
exact-head command succeeded after a fresh read. No review bypass.

NetBSD primary interface reference: [sysctl(7)](https://man.netbsd.org/sysctl.7),
retrieved 2026-10-22. `hw.ncpu`, `hw.physmem64`, `vm.loadavg` provide CPU count,
physical capacity and load, not per-process or used-memory measurements. A fixed
read-only helper may collect these outside the BBS owner; no shell command field.

## Validation and outcomes

- 179 Rust tests per OS; host Clippy/format and 115 Python tests pass. Native
  Rust 1.96.0 source build on NetBSD 11 ARM64 (2 cores / 1 GiB).
- Six Nightwire-themed operator UI controls per OS pass; ten real shared
  owner/member/lifecycle/resource controls and seven actual SSH/native artwork
  regressions pass. Native original-art previews: 18; native local artwork: seven.
- Default-palette dashboard: 2,396 KiB RSS in 12 half-second samples; owner
  5,144 KiB RSS, original four LWPs, dashboard one LWP. Minimum sampled whole-VM
  free memory 334.23 MiB. Transient sysctl-child peak and 250-user capacity unclaimed.
- Test-driven fixes: match the existing decoder's uppercase letter hotkeys; use
  B/H/I rather than unsupported numeric keys; restore input state without ANSI
  reset output in plain operator mode. Actual 40x18 dimensions are displayed.
- Initial source-copy/old-art fixture calls used a missing administrative PATH
  and host path rather than guest source; corrected in the fixture invocation.
  No secret/credential or product-policy change followed those fixture failures.

Keep #95 open for remaining collectors/complete story; #98 host writes and
firewall #97 are explicitly outside this batch. Preserved playground stays off.
All 33 saved-evidence guards, repository policy and nineteen-snapshot /
thirty-two-historical-record guards pass. The exact 152 selected native inputs
match the copied execution tree; eight host harnesses are pinned separately.
Owned clone/cache cleanup is complete and the saved image is not open. Focused
PR #128 is attached and all three grouped issues have status comments; GitHub
checks remain pending. Clean committed archive also passes all source/policy/
evidence guards. Remaining direct-console visual/exit polish stays in #95.

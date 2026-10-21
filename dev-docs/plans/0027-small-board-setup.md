# 0027: A small board that a sysop can run

Issue: [#71](https://github.com/gonz0w/faction-bbs/issues/71), related [#18](https://github.com/gonz0w/faction-bbs/issues/18) and [#35](https://github.com/gonz0w/faction-bbs/issues/35)
Status: implemented; final review and PR checks

## Goal and constraints

The owner directed on 2026-10-21: prioritize a running product for ordinary users
and defer edge cases. Deliver a native first-install path for the current shared
Rust/SQLite board: approved SSH login, editable ANSI theme, rooms and local chat.
Use the existing eight-session defaults and base SSH restrictions. No HTTP,
experimental loader/setuid/NPF profile, host tuning or physical SBC dependency.

## Progress

- [x] Preserve unfinished #70 work separately; switch to a focused product branch.
- [x] Implement read-only preflight and explicit native-root first installation.
- [x] Exercise install, bootstrap, registration/approval, ANSI/rooms/chat and restart/reboot on NetBSD.
- [x] Put the usable path first in README, update sequencing and prepare the focused PR.

## Steps and commands

Build the existing workspace and native frontend on NetBSD. Run
`python3 tools/install_netbsd.py --authorized-keys /root/invited.pub` for a read-only
preflight, then add `--install`. Bootstrap interactively as factiond. Follow
`docs/quickstart-netbsd.md` to start services explicitly and invite/approve members.
Use the disposable VM only for native setup validation. Preserve old synthetic
research data rather than rewriting prior results or resetting a live board.
Run repository checks, installer policy tests and the relevant actual terminal
journeys; no additional 250-member research matrices in this increment.

## Discoveries and decisions

The first local implementation already supplies the essential member workflows.
Manual installation and research-heavy documentation obscure that running core.
This helper installs existing components rather than introducing a new runtime or
network protocol. It refuses an existing installation and does not manage upgrades.
It neither starts services nor changes administrative SSH or firewall policy.

The issue-70 combined-pressure fixture completed its last pressure interval with
129.44 MiB sampled minimum headroom, zero swap and two observed credential peaks.
The full matrix did **not** pass: post-load restart hit a timed-out NPF connection
snapshot. Earlier fixture/timer failures are preserved privately with source maps.
No combined-capacity acceptance is claimed. That experimental-profile work is
paused, not a prerequisite for ordinary small-board deployment. The unfinished
source tree is retained under the durable local git ref
`research/70-paused-checkpoint` (stash-format commit, including untracked sources;
restore with `git stash apply research/70-paused-checkpoint`); the
private disposable VM artifacts remain private. Do not refresh historical hashes.

## Validation and outcomes

The first full member rehearsal passed registration/pending/approval, theme,
arrow/hotkey menus, durable post/read, peer chat and restart. Its fresh boot failed:
normal `su ... &` startup inherited the boot session and the backend exited during
teardown. This is an ordinary availability defect, not deferred edge research.
A tiny **unprivileged, non-setuid** native helper now calls `daemon(0, 1)` and execs
only the fixed shared/worker backend with fixed environment, preserving log stdio.
No resident supervisor is added. A second reboot kept the detached backend
alive but the explicit private-address SSH bind raced DHCP. The dedicated SSH rc
script now follows LOGIN and retries only its exact configured listener for up
to 30 seconds, without wildcard/address/port fallback. The first retry candidate
mistook the daemonizing parent's successful return for successful binding; the
corrected version checks the real listener pidfile after each attempt. A manual
diagnostic start during that candidate's boot poll invalidated that apparent pass;
only a later fully unassisted fresh-boot rehearsal can establish boot success. This is normal startup
availability; no host network or admission policy was relaxed. The backend still writes/locks its own pidfile.
The original rc bytes already exist in the fourth source snapshot; records
035/036/040 now resolve preserved sources and retain independently pinned complete
record hashes. Old matrices do not claim to have measured the new launcher.


Final current-source native first install and the **16-check SSH/member/setup
rehearsal passed**, including a fully unassisted fresh boot, worker rollback and
return to shared, private export/verify and hidden credentials. The separate
native owner export/restore PTY rehearsal passed. See the
[dated validation record](../validation/netbsd-small-board.md) and sanitized JSON.
The complete 100-test Python suite, repository policy/archive checks and all
historical saved-evidence validators pass. No new Rust source changed; CI still
runs the existing workspace/terminal regression checks. This is a hobby preview,
not a packaged supported appliance or a 250-user support promise.

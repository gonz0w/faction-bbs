# Native board launcher: actual two-board NetBSD software proof

2026-10-22, #27/#21/#14. [Sanitized results](results.json) record macOS ARM64 and
actual NetBSD 11 ARM64 native clients visiting two independent software boards.
Each disposable VM has two cores / 1 GiB configured RAM; this is two VMs, not a
claim that two whole boards were benchmarked on one 1-GiB machine. The saved
playground remained off. See [GitHub issue](https://github.com/gonz0w/faction-bbs/issues/27),
[ADR 0051](../../decisions/0051-member-owned-native-board-launcher.md) and
[member workflow](../../../docs/board-launcher.md).

## Passing controls

- **191 Rust tests per OS**, native Rust 1.96.0; host Clippy denies warnings,
  formatting and **118 Python tests** pass. Catalog validation/permissions,
  compact maximum-catalog navigation and fragmented local keyboard escapes are
  covered. New source reuses the existing profile and strict SSH policy.
- **Seven launcher PTY controls per OS**: no startup connection, arrow/hotkey/page
  choice, canceled confirmation, 40x18 maximum catalog, selected-profile reload/
  failure recovery, local Ctrl-C/terminal restoration, loose-catalog refusal and
  a loopback trap proving no connection before confirmation/after cancel/error.
- **Ten actual two-board controls**: board A registration/local approval and a
  file area; local confirmation cancellation; native Files area listing/return;
  declined then immediately repeated local disconnect with explicit confirmation;
  independent registration/pending denial on B; B's own sysop approval then login;
  normal logout return; wrong/unknown host-key refusal before BBS login; actual
  native NetBSD client visits to A and B with Files return; and one SSH child per
  active host/guest client with four LWPs per shared owner.
- Regression controls: eleven directory PTY checks per OS, three local graphics
  gate controls per OS, eight native operator UI checks, eighteen native artwork
  previews, seven native local/worker art routes and seven actual restricted-SSH/
  native single-profile artwork routes. The two-board Files proof covers metadata
  negotiation/listing/return; it is not a fresh SFTP byte-stream/restore proof.

Both boards use their ordinary fixed socket paths, separate databases and distinct
member SSH host keys. Host-only loopback forwards and private guest networking
connect the fixture; no public listener or firewall/OS tuning is enabled. Each
member still authenticates/registers independently; no identity federation or
profile import occurs. The second board uses byte-identical copied installed
owner/service/TTY/startup binaries from the first, not a modified server build.

## Corrections and limits

The first actual visit attempt found a client prompt race: cancellation printed
its acknowledgement before flushing old input, so the immediate next shortcut
could be swallowed. The final candidate flushes before acknowledging; the passing
oracle exercises that exact rapid cancel/retry sequence. Earlier candidate source
and failure logs were retained privately; no failed result was relabeled as pass.

The initial one-VM fixture correctly failed the production fixed-socket/owned-theme
checks. Validation moved to separate VMs and correctly owned packs, retaining
production policy. Rebuilds stop the fixture SSH listener, which was explicitly
restarted. The NetBSD process-count harness also needed its native `ps` selection/
header syntax rather than Linux-style combined empty headers. Those corrections
changed fixture expectations/setup, not server authorization or resource limits.

168 selected native build/runtime/validation inputs match the actual first-VM
copied tree; eight relevant host harness inputs and both boards' installed binary
hashes are recorded separately. All **110 checked server/workspace inputs** remain
byte-identical to the prior directory candidate. This selected set is not every
VM/base/toolchain image or every host test input. Startup helper provenance remains
the existing installed fixture; this batch does not rebuild/harden that helper.

One active client SSH child is sampled on each OS. Catalog <=16 KiB / sixteen
profile paths, four per page; per-visit profile <=8 KiB, path <=512 bytes. Client
filters/nonces/link history restart each visit and the old SSH child is reaped.
Server eight-session/five-second admission, two credential permits, three helpers/
four LWPs, 32-KiB output/five-second drain and reviewed dependencies are unchanged.
No new RSS/peak/250-member, all-port, physical-SBC, real-GUI/visual, encrypted-mail,
roaming identity, federation, full-transfer or hardening certification is claimed.

## Reproduction and preservation

Build the reviewed locked workspace unprivileged on each supported software host.
Run `tools/smoke_board_launcher.py --help`, `tools/smoke_directory.py` and the
existing graphics/artwork tools. Prepare two independently owned marked NetBSD
VMs using the normal installation path, separate private synthetic accounts/state
and distinct pre-enrolled member host keys. Keep production socket/defaults intact.

`tools/smoke_board_visits.py --help` accepts private admin/member argv, two profiles,
synthetic password files and a native guest-launcher argv. Both guests must have
`/etc/faction-disposable-test`; the first uses `/var/db/faction-profile-fixture`,
the second `/var/db/faction-visit-b`. The first must explicitly enable optional
Files with its normal separate account policy. The guest launcher runs the real
NetBSD-built client under an unprivileged member-device account, with its own
private profile catalog. The tool creates synthetic registrations, local approvals
and an area through actual BBS sessions. Never target a saved/live board.

Keep all keys/passwords/argv/failed transcripts private. Pin selected source bytes
and binary identities after final tests. Verify each QEMU PID/disk path before
shutdown, wait for exited processes/closed disk handles, remove only those owned
clones/cache and verify no open handle on the saved original. Teardown flags report
actual completed actions; no personal terminal/login-profile change is made.

Snapshot 23, `sources/pre27-launcher`, and `directory-baseline-manifest.json`
preserve 174 distinct prior source/harness inputs and the whole original directory
record. All 23 snapshots/36 historical result identities remain fixed. Preserve
this new measured source/harness set and whole result before future changes.

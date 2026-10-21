# 0026: Ship the ordinary small-board path first

Date: 2026-10-21
Status: accepted product priority; native first-install/member/boot validation passed
Related: [#71](https://github.com/gonz0w/faction-bbs/issues/71), #18, #35, #55, #70

## Question and constraints

What gets a hobby sysop and invited members into a usable Faction board now?
The owner explicitly asked to defer edge cases and build for ordinary users.
Rust/NetBSD, approved membership, SSH, sysop ANSI identity, private local content,
no JS/TS and small-host defaults remain requirements.

## Decision and alternatives

Package the existing ordinary shared backend path into a first-install helper
and short sysop/member quickstart. Keep eight sessions, two credential slots,
current Argon2 and base SSH restrictions. Installation is explicit native-root
setup, read-only without `--install`, refuses an existing deployment and starts
no services. Bind loopback unless the sysop selects one interface. Keep HTTP,
experimental privileged profiles, NPF policy and host tuning out of this path.

Continuing combined 250-user matrices would delay the common experience; retain
that incomplete work under #70/#55/#20. A new appliance image/upgrade manager
would expand scope; source build plus first install is sufficient for this preview.
Lua menus/isolated doors remain the next extension track. Files/mail/directory
and later network/payment adapters remain explicit backlog work.

The ordinary fresh-boot rehearsal exposed backend SIGHUP loss and an SSH bind
race with DHCP. Use a tiny **unprivileged, non-setuid** native startup helper to
detach and exec one of the two fixed backends with fixed environment. It has no
resident process or new member privilege. Preserve rc log stdio and let the
backend own its real locked pidfile/socket. Do not ignore HUP globally or
introduce a new supervisor. Base SSH startup retries only the configured listener
for a bounded 30-second interval, checking its actual pidfile after daemonization;
it never widens the listener address or changes host networking.

## Primary sources and observations

Retrieved 2026-10-21:

- [NetBSD daemon(3)](https://man.netbsd.org/daemon.3): controlling-terminal detachment,
  cwd/stdio options and changed PID. The helper uses `daemon(0, 1)` before exec;
  the backend pidfile, rather than the launcher's PID, establishes readiness.
- [NetBSD rc.subr(8)](https://man.netbsd.org/rc.subr.8): custom start commands and
  actual process/pidfile checks. The custom SSH start explicitly validates its
  prerequisites and rejects a duplicate running listener.
- [NetBSD sshd_config(5)](https://man.netbsd.org/sshd_config.5): dedicated config,
  fixed listen address, forced command and forwarding/authentication controls.
- Native NetBSD 11 ARM64 VM: the existing core completes the ordinary member
  journey. Early reboot candidates failed; the final unassisted fresh-boot result
  is recorded in [the dated validation](../validation/netbsd-small-board.md). No 250-user matrices are
  rerun or credited to the changed startup code.

## Tradeoffs, confidence and reversal

The helper adds a small C build input, but executes as the existing service owner
and exits via backend exec. It does not add privileged loader/initializer code,
per-member overhead or a runtime package. Root/operator and shared backend remain
trusted; this is not arbitrary-code isolation. First install is intentionally not
an automatic upgrade/uninstall transaction. Partial failures require inspection.
Confidence in the member core follows prior native evidence; setup/boot confidence
requires the actual current-source rehearsal. Other ports/SBCs stay untested.

Reverse by stopping member SSH and backend, restoring the prior rc scripts and
using the existing manual setup/worker rollback. Keep live lock files and data
in place. Before any future upgrade, export/verify and rehearse restore.

## Falsifiable validation

Native first install must leave both services disabled. Real SSH must enforce
pending-before-identity/content, sysop approval, custom ANSI arrows/hotkeys,
durable peer room reads, two-member chat and hidden credentials. Normal restart
and a completely unassisted boot must retain approved access and theme. Verify
private export, fixed UID/file boundaries, native helper analysis/build and worker
rollback. Repository checks and old evidence hashes must remain valid.

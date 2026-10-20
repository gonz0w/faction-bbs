# 0035: Minimal filesystem, exec and IP boundaries

Date: 2026-10-20. Issue: [#55](https://github.com/gonz0w/faction-bbs/issues/55),
related #19/#20/#23/#28. Status: experimental implementation/native software
validation; production sandbox, complete budgets and supported capacity deferred.

## Question and implementation

Can we restrict host filesystem/exec/IP authority without consuming the headroom
retained for 250 NetBSD members? The opt-in profile adds a pre-main native
initializer, one minimal noexec nullfs root and explicit NPF group/user policy.
The core, maintained UI/schema, credential costs, two permits/slots, three
helpers, eight-session default and existing ceilings are unchanged. No new
registry dependency, external kernel module or resident supervisor is introduced.

[ADR 0021](../decisions/0021-experimental-os-isolation.md) records the decision;
[the operator contract](../../docs/experimental-isolation.md) describes privileged
startup, setup, limitations, rollback and explicit firewall review. The profile
alone adds a setuid frontend, increasing trusted native startup code. It remains
experimental pending independent review. Existing normal binaries are preserved.

## Alternatives and primary sources

Retrieved 2026-10-20:

| Candidate | Evidence and disposition |
| --- | --- |
| Base nullfs/chroot and mount flags | [mount(2)](https://man.netbsd.org/NetBSD-11.0/mount.2) documents read-only/noexec/nosuid/nodev. Adopt these narrow facilities with native tests; chroot alone is insufficient. |
| NPF | [npf.conf(5)](https://man.netbsd.org/NetBSD-11.0/npf.conf.5) documents user/group matching for TCP/UDP. Adopt an explicit stateless experimental candidate after live endpoint tests, not an inferred all-family sandbox. |
| PF | [pf.conf(5)](https://man.netbsd.org/NetBSD-11.0/pf.conf.5) marks this NetBSD facility obsolete/discouraged; reject for new adoption. |
| External secmodel_sandbox | [project source](https://github.com/smherwig/netbsd-sandbox/tree/0ba66019e9277d58e12fb5319867fe9bb3331b0c) shows an external kernel/Lua sandbox dependency. Interesting broader candidate; not required for this slice. |
| Cells | [project](https://netbsd-cells.petermann-digital.de/) and [NetBSD 11 development branch](https://github.com/MatthiasPetermann/netbsd-src/tree/netbsd-11-cells-dev-2) require a patched kernel/control stack. Evaluate separately rather than changing the base image silently. |
| Future kauth boundary | [kauth(9)](https://man.netbsd.org/kauth.9) exposes authorization hooks. More coverage needs a separate maintained implementation and native adversarial gate; no module is installed here. |

## Native findings and rejected hypotheses

A pre-opened outside-root static ELF did not provide a working fexecve entry
mechanism on this release. A pre-main constructor worked with already mapped
runtime-library dependencies. Both are native findings, not all-port conclusions.
Separate nullfs aliases also failed Unix socket connection. The selected common
root/run view passes the original descriptor protocol and peer-UID assertions.
Frontend permissions deny owner-only config/theme/database reads in that view.

The UID-only NPF candidate failed a native terminal TCP denial. It was rejected;
the selected candidate additionally requires a dedicated fixed primary group and
blocks it before the two users. Both roles pass the resulting live endpoint
tests. The cause of the rejected result is not established as an upstream defect.
No private endpoint, packet trace, key, credential or transcript is published.

Positive root controls prove the IPv4/IPv6 listeners, UDP delivery and raw socket
capability actually exist. Sealed probes require TCP connect failure, no UDP
echo/unsolicited receive or delivered forbidden payload, and denied raw/datagram
ICMP sockets. The valid static ELF first executes outside, then fails pathname
and descriptor exec inside. Other probes deny host paths, root writes, config
writes, frontend config/database reads, fchroot and privileged/group regain.

The native frontend suite binds mocks through the actual shared nullfs view;
binding only through a lower alias was a failed setup, not positive peer evidence.
Root-controlled theme packs need root ownership after archive transfer. Cargo
does not track an external object changing at the same path; the build tool uses
a content-addressed object link argument to force a relink. These corrections
precede the counted source/binary identities and results.

## Method, measurements and interpretation

The official signed NetBSD 11 ARM64 image and kernel were verified afresh against
SHA-512 and the signed manifest/key fingerprint. HTTPS key retrieval is not an
independent web-of-trust attestation. The VM uses two vCPUs, configured 1 GiB,
1006 MiB physical/972 MiB UVM and no swap. There is no physical SBC evidence.
The dataset retains 1000 approved independent-salt accounts, sysop, 50 rooms and
100000 approximately 1-KiB posts. Generated credentials remain private.

Counted tests: 63 unchanged native Rust tests; 29 full shared UI/authority/fault
oracles; 31 actual sealed frontend assertions; 16 profile lifecycle/startup
checks; live filesystem/exec/IP/privilege positive and negative controls; and
bidirectional credential/pid lock compatibility across host/sealed views. After
restoration, the ordinary frontend passes 31 checks, worker rollback passes the
20 SSH/13 lifecycle checks and six native PTY application/credential smokes.
The kernel refused enabling user mounts at the running securelevel; this records
that kernel behavior, not a passed initializer test with user mounts enabled.
Actual runtime ELF hardening and native analyzer checks are captured separately.

Three fresh boots retain all base services, 250 actual distinct approved member
transports/frontends and at least 501 dedicated SSH processes. Each performs
menu/chat stages at 32/100/200/250, delivery, failed/successful credentials at
200/250, full room/post/menu/chat work at 250, two separate owner credentials,
touched 0/19/38-MiB controls, loaded stop and fresh-login recovery. Every sampled
owner/frontend keeps the existing soft/hard process/files/LWP/core/mlock ceilings.

Research settings remain explicit: compact allocator in both host and minimal
root, pageable NTP and experimental process/file limits. Native readbacks inside
both sealed roles verify the allocator, including setugid behavior. Service setup
does not apply these settings or activate NPF. Fresh-boot selection of NPF is a
separate marked-VM operator action. Restore/boot/readback, offline removal of known synthetic test rows and
verification of the original stopped seed follow measurement, before private VM/key/log cleanup.

Two initial fresh-boot functional completions failed the heaviest sampled stress
control: 126.36 and 123.47 MiB. Both are retained as rejected evidence. The
initializer used `sysctlbyname` for one fixed user-mount check. The selected
candidate uses `CTL_VFS/VFS_GENERIC/VFS_USERMOUNT` directly instead. At the matched
32-user chat stage, frontend virtual mappings fell from about 11.37 to 7.34 MiB
per process; virtual size is not physical RAM. This removes name lookup rather
than weakening the check or guard. NetBSD's public [PR 59147](https://gnats.netbsd.org/59147),
retrieved 2026-10-20, describes allocation during userland sysctl name resolution
and difficulty using it in ELF constructors. The native measurements determine
our capacity outcome; the upstream report alone does not prove a RAM saving.

| Fresh boot | Headroom before loaded stop (MiB) | Lowest stress headroom (MiB) | Loaded stop (s) |
| --- | --- | --- | --- |
| 1 | 178.59 | 137.84 | 1.926 |
| 2 | 178.84 | 133.60 | 1.777 |
| 3 | 178.11 | 132.71 | 2.884 |


The unchanged guard is free + max(0, clean-file - wired) >= 128 MiB, zero swap,
plus 64-process and 128-file reserves. Whole-VM counters and all dedicated SSH
and base services count. The predicate is sampled/batched rather than continuous
enforcement. RSS repeats shared mappings; no summed-RSS/USS/PSS budget is inferred.
The loaded controls are known touched pages, not a worst-case cold Argon2 bound.

[Complete result 036](results/036-netbsd-isolation.json) pins current executed
sources, native transfer hashes, actual installed/build binaries, full sampler
and limit ledgers, controls, boots, settings/restoration and cleanup. The separate
profile leaves every measured input in result 035 unchanged, so all existing
records/snapshots remain byte-identical. Validate with
`python3 tools/check_isolation_evidence.py` and the existing historical checkers.

## Remaining adoption gates

Independent review of setuid/loader/startup policy; all relevant network families,
protocols/interfaces, same-UID/process/tracing and arbitrary-code threats; custom
firewall composition and live policy-change handling; supported boot/image/
timekeeping maintenance; authenticated-transport admission; broad/hostile/soak
loads, worst-case themes/output and complete application/host budgets. Reboot and
restoration are not proof of a supported appliance lifecycle. Extensions still
require separate constrained launch/UID capabilities. This advances #55 without
closing those broader gates or changing the eight-session default.

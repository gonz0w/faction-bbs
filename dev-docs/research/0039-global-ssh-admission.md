# Global SSH admission and private-source separation

Date: 2026-10-21. Issues: #55, #23, #18, #28.
Status: native software experiment; adoption and supported capacity remain open.
[ADR 0024](../decisions/0024-global-ssh-admission.md),
[operator boundary](../../docs/experimental-global-admission.md),
[plan](https://github.com/gonz0w/faction-bbs/issues/55) and
[result 040](results/040-netbsd-global-admission.json).

## Question and owner direction

After #68's four-query helper allowance, an attacker can still open fresh TCP
transports. Can a fixed-size pre-fork global gate bound that work without adding
a proxy/resident process or losing the remaining 250-member memory margin?
The owner chose unchanged native OpenSSH penalties and exploration of private
network identities to distinguish peers sharing a NAT. Local approval is unchanged.

## Candidate and alternatives

The ADR compares base concurrency/penalties, inetd, NPF/IPFilter, a proxy, a base
patch, narrow listener interposition and authenticated private peers. Retain base
OpenSSH; probe a C library only in its dedicated listener. It interposes actual
native `listen`, `accept` and `execv` imports, returns original sockets, and removes
loader variables before session exec. There is no resident admission daemon,
new SSH parser/crypto or member-side library after exec. This adds privileged
loader/call-path coupling; a small base patch remains a credible alternative.

Global burst eight/refill 5000 ms and per-source burst two/refill 10000 ms are
defaults. Explicit experimental global 250 ms gives source refill 500 ms; direct
same-source clients in the matrix are deliberately paced at least 550 ms. A
per-source rejection preserves global tokens. Only global admissions create
source entries; a source refills fully in four global-refill periods. At most
eight plus four global permits arise in such a window, so 32 slots leave margin
without an unbounded address table. Exhausted table entries deny rather than
allocate. IPv4-mapped IPv6 is normalized; IPv6 scope is conservatively grouped.
Source history is volatile across listener restart; global budget is persistent.

The root-only 120-byte state is locked, metadata/checksum/inode checked and fsynced
before OpenSSH may fork for a returned socket. Reboot/changed boot stamp starts
empty; normal restart retains tokens. Drop counters flush unsynchronized at most
once per second and can lose a crash tail; permits are durable. No address,
credential, message or tunnel key is persisted there. HUP deliberately stops the
listener, preserving authenticated sessions; rc restart performs preflight again.
Exact base SSH/module hashes and unchanged penalties/netblocks/exemptions are
required. Global accounting is distinct from the existing 8104-byte key registry.

## Native setup and controls

Official SHA-512/signature-checked NetBSD 11 ARM64 software image, 2 vCPUs/1 GiB,
Rust 1.96, native C/Python/base SSH; no physical SBC test or OS security downgrade.
Actual SSH is **OpenSSH_10.3 NetBSD_Secure_Shell-20260408-hpn13v14-lpk**. Earlier
ADR 0023/plan 0024 prose incorrectly said 10.2; unchanged saved 037–039 records
already contain 10.3. The source reference is not runtime build-lineage proof.

All base services, all dedicated SSH processes, key helper transients, frontends,
shared owner, kernel/free/file counters and the unchanged 128-MiB/zero-swap guard
remain in whole-VM accounting. Compact allocator/pageable NTP and increased
maxproc/maxfiles are explicit fixture inputs, restored afterward, never automatic
service behavior. One SSH invitation key serves distinct approved BBS accounts;
250 unique invitation keys or private tunnel peers are not measured.

Native gate controls check the default/explicit bounds, actual listener forks
(eleven rapid multi-source offers: eight banners/eight forks), source-share token
preservation, durable same-boot restart, live reset lock, signed SSH and listener-
only steady-state mappings, HUP/session survival, corruption stop/no repair,
hardlink/mode/missing state and source-policy/hash preflight refusals. A separate authenticated-PTY audit also reads native
process environments and open-file ownership: no loader variables survive into
member descendants, and only the listener holds the budget descriptor. This
audit is narrower than an all-process FD audit at 250. The full
29 native shared UI/approval/theme/revocation/failure checks also pass with
explicit 250-ms source pacing under the 100-ms experimental global profile.

The 250-member driver retains original 32/100/200/250 menu/chat stages, simultaneous
member and owner credential controls, two excess-key denials, full UI/post actions,
0/19/38-MiB touched controls, process ceilings, load-stop and fresh-login recovery.
It adds bounded multi-source silent TCP pressure while all 250 members keep useful
chat; non-sender receipts are counted explicitly. This short control is separate
from the historical 15-minute intervals and is not a long soak or CPU budget.

## Failures retained

`dev-docs/evidence/global-diagnostics/` preserves executed failed candidate bytes:
initial preflight rejected base SSH's legitimate 0555 mode; initial rc pid detection
misread a success status with empty output; initial runtime-state placement failed
boot recovery because NetBSD clears `/var/run`. All failed closed. Final state
uses private `/var/db`, and base 0555/0755 plus actual nonempty pid detection are
handled. Initial fixture omitted loopback listener binding and an early temporary
UI entry script had indentation wrong; those runs did not measure application
failure or count as passing native controls.

The auxiliary network peer needed software emulation after HVF boot failures,
and explicit `/etc/modules.conf` boot loading for `if_wg` at securelevel 1. A file
named `/etc/modules` was ineffective; no securelevel lowering was used. Auxiliary
resources are outside server capacity accounting; server-side module/interface
costs remain included where configured. These fixture failures establish neither
small-VM incompatibility in general nor hardware behavior.

One later backend restart preflight timed out inspecting `npfctl list -n` at
the existing 20-second deadline; it failed closed. The same unchanged policy
subsequently returned in 0.211 seconds (`show` in 0.142 seconds), and the retry
started the backend. Later setup also refused to start the listener without a
live backend socket after boot had reported success; explicit backend preflight/
start restored the fixture, and the cause is not established. Retain these diagnostics alongside #20 timing/host
uncertainties; do not describe all post-run observations as uniformly clean.
The first full matrix passes at 250 members with **133.68 MiB** minimum
stress/control headroom and **142.20 MiB** under 60 seconds of TCP pressure:
2,865 offers and 16,683 confirmed non-sender peer receipts. Listener mappings
remain four; 752 checked session/auth/frontend processes contain no gate.
The second matrix includes two configured private peers and passes with
**132.86 MiB** minimum stress and **141.14 MiB** under pressure, 2,855 offers and
17,928 confirmed peer receipts. Total: **5,720 offers / 34,611 peer receipts**.
Both 60-second intervals retain all 250 approved members. These are configured
peer costs, separate from a workload of 250 actual VPN clients. The worst-case
preauth plus touched-control/credential overlap is not measured concurrently.
All numbers are software VM samples.

The initial private-peer probe required ICMP, which failed under the selected
policy despite a working tunnel handshake. The final test uses actual SSH TTY
flows and positive/forged UDP controls; no ICMP-support claim or policy relaxation.

## Interpretation and remaining gates

The candidate bounds sockets returned before the observed base fork path; it
cannot bound kernel SYN/TCP work, packets, link saturation, all CPU/IO work or
post-authentication behavior. One source cannot consume the entire steady-state
global rate, but multiple hostile sources can still compete. Native NAT penalties
remain; listener restart is not a fairness fix. A cryptographically bound inner
address is a separate network admission identity, never local BBS approval.
NetBSD labels wg itself experimental; no provider/overlay production adoption.

Independent privileged-code/loader review, OS update/HUP/rollback policy, durable
failure, broad malicious-source overlaps, actual 250 unique-key/peer costs,
maintained timekeeping, calibrated CPU/IO/network budgets and long soak remain
open under #55/#23/#18/#20. #28 is the closed budget-definition reference,
not an open implementation ticket. Normal defaults remain eight.

## Primary sources and reproducibility

Retrieved 2026-10-21: [loader](https://man.netbsd.org/ld.elf_so.1),
[native accept](https://man.netbsd.org/accept.2),
[SSH configuration](https://man.netbsd.org/sshd_config.5),
[inetd](https://man.netbsd.org/inetd.8), [NPF](https://man.netbsd.org/npf.conf.5),
[IPFilter](https://man.netbsd.org/ipf.conf.5),
[wg experimental boundary](https://man.netbsd.org/wg.4),
[wg configuration](https://man.netbsd.org/wgconfig.8) and the pinned
[sshd call path](https://github.com/NetBSD/src/blob/36cf5108df513943b2914535a5e7c2f86fa35fd7/crypto/external/bsd/openssh/dist/sshd.c).
Native binary/import/fork/mapping observations supplement, not replace, source
review. No newer SSH was installed to manufacture a dependency result.

`tools/build_ssh_gate.py`, `tools/check_global_admission.py`,
`tools/check_global_host.py`/`check_global_scaling.py`, `global_workload.py` and
`tools/check_private_peers.py` are explicit disposable-fixture tools. Saved evidence
checking does not boot a VM. Thirteen gate controls, six real channel/restart controls, the direct native exec
audit and 29 UI checks pass on the final candidate. Four immutable historical
source snapshots preserve
all twelve prior records, including all 145 original result-039 execution inputs.
035/036 still check unchanged current sources; old hashes are never refreshed.

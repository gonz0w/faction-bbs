# 0025: Global SSH admission and private-source separation

Issue: [#55](https://github.com/gonz0w/faction-bbs/issues/55), related #23/#18/#28.
Status: native experiment complete; final checks and PR publication, 2026-10-21.

## Goal and constraints

Add a falsifiable global admission-before-fork bound while keeping base OpenSSH
parser/signatures/options/source penalties, local approval, unchanged Argon2/two
permits/locks, all ceilings, default eight and the 128 MiB/zero-swap guard. User
chose unchanged penalties and exploration of per-member private-network identities
for stronger separation behind NAT. Ordinary direct SSH remains independent;
no provider, public HTTP, JS/TS or physical hardware is required.

## Progress

- [x] Merge #68 at exact passing head; no unresolved review threads.
- [x] Preserve all 145 measured 039 inputs and unchanged record in snapshot 5b62d0a.
- [x] Compare kernel firewall/inetd, base change, listener interposition and
  private-network identity options against security/memory/maintenance gates.
- [x] Implement/probe a narrow pre-fork gate with persistent restart budget;
  retain source address and unchanged native penalties, no proxy or SSH parser.
- [x] Native rate, restart/HUP, missing/corrupt/alias/config, child environment/
  mapping/FD, ordinary client/local approval and failure controls.
- [x] Explore distinct authenticated private peers sharing an outer address;
  verify isolation/anti-spoof/source penalties rather than exempting the NAT.
- [x] Two fresh full 250-member matrices and useful-member bounded abuse controls;
  keep every earlier failure/source pin and verify host/VM cleanup.
- [x] Update README/operator/security/research/ADR; publish issue-linked PR after final checks.

## Steps and discoveries

Preserved current input bytes before changes. Existing MaxStartups caps simultaneous
preauth, not global fork rate; inetd's automatic ten-minute shutdown and prior
per-connection zombie costs are poor fits. A packet proxy obscures peer identity.
Unmodified per-IP penalties cannot distinguish members behind one NAT. Private
peer identities can supply distinct verified inner addresses, but remain transport
admission separate from local BBS membership. NetBSD wg(4) itself is experimental;
no provider or production support is inferred from its manual.

Candidate: a scoped native accept/listen/execv interposer in the one root listener,
with fixed persistent root-only budget and explicit root preflight. It delegates
original sockets unchanged and removes itself before sshd-session exec. HUP must
stop or preserve the gate, never silently reexec unprotected. Native symbol and
whole-process-map proof are mandatory; production use needs independent review.
No host-wide preload, firewall replacement or automatic tuning is acceptable.
This candidate may be rejected on any boundary or memory failure.

## Validation and outcomes

Initial historical preservation validators pass (four snapshots/twelve records).
Thirteen native gate controls, six channel/restart controls, direct native
post-exec environment/descriptor audit and 29 full shared UI checks pass. Two
fresh 250-member matrices pass unchanged sampled guards: stress minimum
132.85546875 MiB, pressure minimum 141.140625 MiB, zero swap; 34,611 peer receipts
in two 60-second intervals/5,720 TCP offers. The second includes server-side
configuration of two private peers, not 250 VPN clients. Actual reboot retains
inode/charges and starts empty at securelevel 1. Twenty-two failed attempts by
private peer A leave peer B's fresh approved BBS/chat useful; positive/forged UDP
controls pass without changing penalties or resetting during the interval.

Retain initial preflight mode/pid/runtime-state and ICMP candidate sources. One
post-load NPF inspection timeout and later missing-backend setup refusal remain
unexplained diagnostics (#20/#55); explicit unmodified-policy restart succeeds.
Final native rollback passes 31 ordinary frontend, 20 SSH, 13 worker lifecycle
and six PTY flows; 63 native Rust tests passed on the built application. Native
ELF/analyzer checks pass. Experimental listener/VPN/module/NPF/memory/timekeeping
settings are restored and read back after boot. Both task VMs/private inputs and
TCP/UDP forwards are removed before publication. Saved current execution inputs:
159; all four historical snapshots/twelve records and direct035/036 pins retained.
Repository/evidence/unit checks and exact-head CI status are recorded in the PR.
The experiment does not close adoption, independent review, combined overlap or
maintained-host/long-soak gates.
 Keep global accepted-socket/fork work distinct from kernel
TCP/SYN queues, network saturation, post-auth BBS work and universal fairness.

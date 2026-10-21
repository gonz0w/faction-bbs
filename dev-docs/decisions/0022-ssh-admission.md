# 0022: Experimental bounded SSH transport admission

Status: accepted for a software experiment; production adoption deferred.
Date: 2026-10-20.
Owner: project research implementation under the owner's #55 continuation;
independent privileged-code review remains required.
Issue: [#55](https://github.com/gonz0w/faction-bbs/issues/55), related #19/#20/#28.
Supersedes: none; ordinary SSH deployment and ADR 0021 remain independent.
Research: [0036](../research/0036-ssh-admission.md).

## Question and constraints

Can authenticated SSH costs be bounded at key authorization before member
frontend/BBS admission,
while retaining base OpenSSH and the measured 250-member headroom? An invitation
holder may open connections without a terminal; slow unauthenticated peers also
consume transport resources. Application session/credential caps act too late to
bound all such processes. Preserve approval/capabilities, unchanged Argon2/two
permits/two kernel slots, small defaults, SSH isolation and all whole-host costs.

## Gate screen and alternatives

| Candidate | Security/resource gate | Disposition |
| --- | --- | --- |
| Existing base daemon settings only | MaxStartups covers preauth; MaxSessions is per transport | Does not supply an authenticated aggregate cap; preserve as rollback. |
| Base NetBSD inetd accept/rate controls | Native accept cap is useful; launch limiter disables the service for ten minutes; per-IP records/NAT behavior require review | Decline this limiter for automatic short refill, retain as a credible simpler alternative. |
| Firewall states/source limits | Exact profile is currently stateless; TCP/state/source policy differs from authenticated transport lifetime | Unknown; do not silently replace an operator firewall. |
| Application-only gate or terminal-UID process cap | SSH may never create the BBS frontend; root monitors precede terminal UID | Insufficient for this boundary. |
| New Rust launcher | Memory-safe parser/table ownership is attractive; fork/exec, descriptor/signal/lifetime operations still need native unsafe/FFI work and measurement | Future comparison; no language-wide rejection. |
| Small C listener plus base sshd inetd mode | Fixed table/bucket works functionally; two of three stress controls fall below 128 MiB | Reject for the 250-member memory target; preserve results and executed bytes. |
| Base AuthorizedKeysCommand with dedicated UID and fixed birth registry | No resident daemon or per-connection config-sender zombie; native state/restart/authorization gates required | Select for this experiment; global pre-fork rate limiting remains separate. |

Use qualitative comparison because a production security score would hide unknown
gates. Current default admission, private communities and privacy are mandatory;
broader availability, IPv6, crash service management and independent review are
unknown production gates. No score can compensate for them.

## Decision and tradeoffs

Use base OpenSSH's AuthorizedKeysCommand under a separate noninteractive UID.
Root-owned helper code copies bounded root-managed invitation lines unchanged;
OpenSSH parses keys/options and verifies signatures. AuthorizedKeysFile none and
strict actual rc.d preflight prevent a key-file/certificate/forced-command bypass.
The helper has no resident process, socket/SSH parser, proxy or new SSH crypto.

A fixed registry reserves one slot per real-root parent plus kernel birth time.
Repeated key queries reuse it. Effective UID is deliberately not used for liveness:
OpenSSH temporarily adopts the helper effective UID while collecting output.
Exited identities are pruned, PID reuse cannot retain an old reservation, and
old authenticated monitors remain counted across listener restart/reparenting.
Default cap/burst are eight, refill five seconds; higher/faster is explicit.

This is a key-admission/authenticated cap, **not a total TCP or global pre-fork
rate cap**. Base MaxStartups 3:30:8, penalties, login/unused deadlines, one channel,
forced command and forwarding restrictions remain. Up to eight additional
preauthentication connections and shared-NAT penalty behavior remain explicit.
Global token consumption can still deny availability; lookup amplification,
preauthentication CPU/fork pressure and reserved fairness need further work.

State is a synchronized, checksummed, fixed-size private file with a persistent
flock inode. Missing/corrupt/writable/aliased state fails closed. Root-only reset
refuses live recorded monitors. The rc initializer can create new runtime state
only after proving no member listener, terminal identity or orphaned SSH session
survives. It never repairs corrupt state or unlinks a live lock. Default process
visibility is required; startup never lowers security.curtain or changes host policy.

No schema/content change, registry service or modified base SSH is introduced.
Rollback cost is moderate: drain the selected member transports, stop this
listener and select the preserved ordinary path. Existing community data remains
local and exportable. Admission state is host-local operational authority, not
community content to export or federate. No independent production acceptance is
implied by native prototype results.

## Evidence, confidence and reversal gates

Primary NetBSD 11 `sshd(8)` documents inetd mode. `sshd_config(5)` documents the
different MaxStartups/MaxSessions/timeout scopes. NetBSD inetd source documents
its accept/rate implementation. Links, retrieval dates and actual native results
are in research 0036; the runnable contract is
[experimental admission](../../docs/experimental-admission.md).

Native negative/positive probes are required because this adds privileged code.
Short full UI/credential/control trials must retain the unchanged 128-MiB guard,
zero swap and original ceilings, with every SSH process and base service counted.
The rejected inetd candidate retained a zombie config sender per transport; it
remains in process/host accounting. The initial 32-member live-rlimit query failure
is preserved separately from the later genuine memory-guard failures. The selected
key helper preserves the base daemon process model and adds only transient work.

Confidence is bounded by software-VM results, not physical SBC or all-platform
claims. Reopen this choice on a failed guard, SSH monitor lifetime/cleanup change,
base OpenSSH update, state/birth/restart anomaly, unbounded resource finding or
independent review objection. Lua/doors still require separate launch authority.

Actual outcome: 14 key/state/config/restart checks, six channel/restart checks,
29 UI and three fresh 250-member software boots pass the unchanged guard;
minimum stress 129.34 MiB, zero swap. The inetd candidate fails twice and remains
rejected. See the complete result/research for exact sources, restoration,
ordinary rollback and remaining gates. Production adoption remains deferred.

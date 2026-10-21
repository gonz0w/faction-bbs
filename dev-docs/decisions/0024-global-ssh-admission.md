# 0024: Listener-scoped global SSH gate and private peer identities

Date: 2026-10-21. Status: experimental candidate; production adoption deferred.
Related: #55/#23/#18/#28 and [research 0039](../research/0039-global-ssh-admission.md).

## Question and constraints

Can global admission-before-fork be bounded without a proxy, resident worker,
weaker base SSH behavior or lost headroom on the 1-GiB/250-member target? Keep
all existing default-eight, crypto/credential/approval/resource guards. The owner
selected unchanged OpenSSH penalties and exploration of private identities for
members behind one NAT; a relaxed source policy is not the selected direction.

## Alternatives and decision

| Candidate | Merit | Cost / disposition |
| --- | --- | --- |
| Base MaxStartups and penalties alone | Existing maintained parser and bounded concurrent preauth | No global accepted-socket/fork rate; retain underneath candidate. |
| inetd rate controls | Base facility | Ten-minute service disabling, and earlier per-connection candidate failed headroom twice; reject for this slice. |
| NPF/IPFilter | Kernel packet policy | Reviewed manuals do not establish the required persistent global accepted-socket/fork budget. Keep explicit isolation policy; no claim that all possible firewall approaches are impossible. |
| TCP proxy | Separate rate owner | Extra process/parser and peer-address delivery problem; reject here. |
| Local OpenSSH patch | Direct, explicit call path | Maintaining a base security patch across upgrades; viable fallback comparison. |
| Listener-only native interposition | One fixed table/state, no daemon/proxy/SSH parser, original socket | Additional privileged loader/call-path coupling, fsync latency and HUP semantics; bounded experiment only. |
| Authenticated private peers | Separate verified inner addresses behind a shared outer NAT | Additional key/routing/revocation/kernel boundary; separate experiment, not a replacement for global limits or local approval. |

Probe a tiny C accept/listen/execv interposer with 120-byte persistent root state,
32 volatile source entries, global burst eight/refill 5000 ms and source burst
two/refill 10000 ms. Charges synchronize before return to the listener's native
fork path. Restart keeps the global budget; changed boot metadata starts empty.
Root preflight pins exact SSH/module bytes and unchanged source penalties; missing
or damaged authority fails closed. Remove the module before session exec. HUP
stops the listener; explicit rc restart is required. No host-wide preload.

The initial `/var/run` state candidate failed boot recovery because NetBSD clears
runtime files. Preserve its executed bytes and failed outcome; the corrected
candidate uses `/var/db` and must prove persistence over actual reboots.

## Evidence, confidence and reversal

[Result 040](../research/results/040-netbsd-global-admission.json) and its checker
hold native fork/rate/restart/mapping/failure evidence and full memory controls.
Confidence is limited to those named software observations; it does not establish
all-base-version compatibility, production capacity, packet-flood resilience or
fairness against arbitrary many hostile identities. The fixed source cache can
conservatively deny service; no peer may gain more global tokens by exhausting it.
Source buckets remain volatile across listener restart.

Rollback removes only opt-in rc selection and returns to the checked base key
listener, retaining authority files and original protections. No schema/content
migration or mandatory transport dependency is introduced. Upgrade and independent
privileged-code review costs are material. If those costs outweigh a small base
patch or kernel approach, reject this interposer rather than weakening guards.

## Falsifiable adoption gates

- Native global permit/fork bound, persistent restart/reboot behavior, failure and
  original socket identity, with no module/environment/authority in member execs.
- Original full BBS approval/theme/channel/confinement behavior and all memory,
  zero-swap/process/file/credential ceilings under matched 250-member controls.
- Scoped peer identity and anti-spoof tests with unchanged penalties; no universal
  shared-NAT or 250-unique-key/peer claim from one invitation key.
- Independent privileged-code/loader review; upgrades, durable-failure/long-soak,
  maintained image/timekeeping and complete CPU/network/IO budgets remain open.

## Primary references

Retrieved 2026-10-21: [NetBSD loader](https://man.netbsd.org/ld.elf_so.1),
[accept](https://man.netbsd.org/accept.2), [sshd_config](https://man.netbsd.org/sshd_config.5),
[inetd](https://man.netbsd.org/inetd.8), [NPF policy](https://man.netbsd.org/npf.conf.5),
[IPFilter policy](https://man.netbsd.org/ipf.conf.5), [native wg](https://man.netbsd.org/wg.4)
and [wgconfig](https://man.netbsd.org/wgconfig.8). The pinned
[reference SSH call path](https://github.com/NetBSD/src/blob/36cf5108df513943b2914535a5e7c2f86fa35fd7/crypto/external/bsd/openssh/dist/sshd.c)
informs the experiment; it is not proof of the installed binary's build lineage.
Actual native version/symbols and executable hashes are recorded separately.

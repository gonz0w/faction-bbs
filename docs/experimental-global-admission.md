# Experimental global SSH admission

Status: opt-in NetBSD software experiment, pending independent privileged-code
review and adoption under [#55](https://github.com/gonz0w/faction-bbs/issues/55).
It extends the [key-admission profile](experimental-admission.md); it is not a
supported Internet-facing service or a change to normal eight-session defaults.
See [research and evidence](../dev-docs/research/0039-global-ssh-admission.md).

## Boundary

A small native library is loaded into **only the dedicated root OpenSSH listener**.
It interposes the native `listen`, `accept` and `execv` calls, returns original
sockets with original peer addresses, and removes its loader environment before
OpenSSH executes `sshd-session`. It adds no resident daemon, packet proxy, SSH
parser, signature implementation or per-member library mapping after exec.
There is a short normal fork-to-exec interval before the inherited mapping goes
away. The base loader, libc, exact OpenSSH binary and library are trusted code.

Default policy permits a burst of eight accepted sockets, then one per five
seconds globally. Each source gets a burst of two, then one per ten seconds.
Exhausted sources do not spend other sources' global tokens. The fixed 32-entry
source table lives only in listener memory; IPv4-mapped IPv6 is normalized.
Source history resets on listener restart, but the **global budget does not**.
IPv6 scope IDs are not separate identities; this can conservatively group peers.

A root-only 120-byte budget under `/var/db/faction-gate` is locked for the
listener's lifetime and synchronized before an admitted socket reaches OpenSSH's
fork path. It stores aggregate counters and process/boot metadata, no addresses,
keys or content. Normal restart retains its budget; a changed boot stamp starts
empty and refills conservatively. Ordinary NetBSD `/var/run` cleanup cannot erase
this persistent budget. The checksum detects corruption, not a hostile root.
Root inspection is diagnostic and may refuse a racing read. Drop counters may
lose their final unsynchronized second after a crash; charged permits do not.

This bounds the candidate's returns from `accept` and corresponding base-listener
forks. It does not bound SYN/TCP queues, packets, kernel network work, all CPU use,
post-authentication work or deliberate root resets. Multiple hostile addresses
can still compete for the finite global allowance. A source address is not a
person: direct callers behind a NAT share both this bucket and native penalties.

OpenSSH's native penalties, netblock sizes and exemptions stay at base defaults.
There is no source exemption, automatic firewall replacement, weaker SSH crypto,
BBS approval bypass or change to Argon2, credential permits or process ceilings.

## Explicit setup and recovery

Use a **disposable marked VM** for reproduction. Existing isolated/key profiles
must already pass their own setup and checks. The build helper only builds:

```sh
python3 tools/build_ssh_gate.py --output-dir /private/build/ssh-gate
```

As root, install the two resulting files as root-owned, single-link mode 0755
under `/usr/local/libexec`, along with the root-owned preflight
`tools/prepare_ssh_gate.py` and rc script `deploy/netbsd/faction_ssh_gate`.
Ancestors must be root-owned and not writable by group/others. The preflight
requires native NetBSD Python 3.13, base `nm`, and the existing key preflight.

Stop the member listener and drain **all** member transports before configuration:

```sh
/usr/pkg/bin/python3.13 -I -B /usr/local/libexec/prepare_ssh_gate.py --configure
```

This explicitly creates private state/configuration and pins base SSH/library
hashes. Faster refill needs `--experimental-capacity`; default remains 5000 ms.
The 250-member experiment selects 250 ms globally and 500 ms per source, with
separate explicit key/application capacity settings. These are research inputs.
No installer automatically enables a service or changes host/kernel policy.
Disable the alternative member listeners before opting into `faction_ssh_gate`.
Only one member listener may own the common pidfile/locked gate state.

`/etc/rc.d/faction_ssh_gate onestart` verifies policy before loading the library.
Check `onestatus` and a real restricted terminal connection afterward: the base
daemon's parent exit status alone is not a complete readiness assertion.
A missing/changed module, changed base binary, altered native source policy,
unsafe/missing/corrupt/aliased state or surviving orphan preauth cohort refuses
startup. It does not automatically create missing persistent authority. Keep the
state when restarting. `--reset-budget` is an explicit root-only stopped/drained
research operation, not a fairness remedy. Never delete or repair live authority.

**HUP deliberately stops the listener.** Its self-reexec is intercepted so it
cannot silently continue without the gate. Use an explicit rc stop/start;
authenticated sessions survive and remain counted by the separate key registry.
After upgrades, stop/drain, review the changed binary's call path, repeat native
fork/mapping/failure tests, then explicitly reconfigure pinned hashes. A hash
match alone is not a compatibility or security review.

Rollback stops/disables this listener and starts the existing key listener after
its own checks. Preserve both state files. Neither rollback nor restart should
be described as uninterrupted admission or preservation of volatile SSH penalties.

## Private identities behind one NAT

The owner chose to retain native penalties and explore authenticated private
network identities. A per-device tunnel key bound to one non-overlapping inner
address can let OpenSSH distinguish peers sharing an outer NAT. Tunnel/device
admission, SSH invitation keys and **local approved BBS identity remain separate**.
Do not trust caller-selected forwarding headers or a claimed username/address.

NetBSD's native `wg(4)` is itself labeled experimental. Test allowed-source
anti-spoofing, revocation, actual inner peer addresses and useful fresh BBS login
during another peer's penalties before recommending it. A shared tunnel endpoint
that NATs all inner members back together cannot provide that separation. No
provider, mandatory overlay, Internet exposure, or 250-peer VPN support is adopted.

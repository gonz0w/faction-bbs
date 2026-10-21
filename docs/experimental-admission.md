# Experimental SSH key admission on NetBSD

This opt-in profile adds an authenticated-connection cap and a global key-admission
budget to base OpenSSH. It is separate from ordinary `faction_sshd`, keeps the
[isolated frontend](experimental-isolation.md), and remains a development experiment.

## Bounds and authority

A dedicated `factionauth` UID runs root-owned `faction-key-admission` through
OpenSSH's `AuthorizedKeysCommand`. The helper copies the bounded root-managed
invitation key file unchanged; **OpenSSH alone parses keys/options and verifies
signatures**. `AuthorizedKeysFile none` prevents an alternative key-file path.
The actual rc.d preflight rejects bypasses, alternate certificate authority,
forwarding, changed forced commands and a privileged helper identity.

Before emitting key lines, the helper reserves one of 250 fixed registry entries
for its real-root SSH parent and kernel process birth timestamp. Repeated key
lookups reuse that reservation. The parent temporarily has the helper effective
UID while OpenSSH collects output; its real-root identity and birth remain stable.
A process must still present an authorized key/signature, and local BBS login and
sysop approval remain separate. The registry stores no keys, addresses, BBS IDs,
roles, credentials or content. It is private to the dedicated helper and root.

Defaults are **eight key-admission reservations**, burst eight and **5,000 ms**
per new token. A token is not refunded when a connection ends. A full/rate-limited
lookup returns no authorized keys. Native process-birth checks remove exited
reservations, while existing authenticated connections retain their slots across
SSH listener restarts. There is no resident admission daemon, byte proxy or SSH
parser. The core's unchanged Argon2/two permits/two kernel slots remain independent.

This cap applies at key authorization. Base SSH separately retains its
`MaxStartups 3:30:8` preauthentication bound, `MaxAuthTries 3`, 30-second login
and unused-connection deadlines, one channel and forwarding/command restrictions.
**Up to eight preauthentication connections may exist in addition to the reserved
cohort.** This is not a global pre-fork rate limiter or a total eight-TCP cap.
Base SSH's source penalties remain enabled; failures behind one NAT can affect
other members there. The helper adds no per-IP map or ban, and the native study
admits 250 legitimate members through one fixture gateway.

## Explicit setup

The native builder makes no installation/account/configuration changes:

```sh
python3 tools/build_key_admission.py --output /tmp/faction-key-admission
```

An operator separately creates a dedicated local `factionauth` user/group with no
interactive login or other duties. Keep it distinct from root, `factiond` and
`factionterm`. The rc initializer creates `/var/run/faction-auth` owned by that helper, mode 0700,
only after proving quiescence. An operator may create it explicitly during initial
setup; existing unsafe metadata is refused rather than repaired.
Install the executable as root-owned 0755 `/usr/local/libexec/faction-key-admission`
and `tools/check_key_admission_config.py` as root-owned 0555
`/usr/local/libexec/check_key_admission_config.py`, with protected parents.
Install `deploy/netbsd/faction_key_admission` as `/etc/rc.d/faction_key_admission`.

Copy `deploy/netbsd/key_admission_sshd_config.in` to
`/etc/ssh/faction/key_admission_sshd_config`, replace the owner UID and explicitly
select the private listen address. Keep the file root-owned 0600 and the
invitation key source root-owned, single-linked, not writable by peers and at most
64 KiB. The helper must be able to read the public-key source; it needs no private
host key or private BBS data. Retain `AuthorizedKeysFile none` and the dedicated
command user. The template does not use Include or Match; preflight rejects them
rather than validating only one branch of a composed policy.

Drain the previous member listener before selecting `faction_key_admission=YES`
in root-managed rc configuration. For a new installation, the rc initializer creates a valid registry only after
proving quiescence; an explicit root `faction-key-admission --reset` is also
available when all old transports are known gone. Start the isolated backend
before the new listener. Normal key lookups never create missing state.
Do not run multiple unrelated member profiles against the same endpoint/PID file.
Use the installed root preflight and base `sshd -t` before start. No script enables
NPF, lowers privacy policy, tunes allocator/NTP/sysctls or enables public HTTP.

Higher caps or faster refill require `--experimental-capacity` in the trusted
AuthorizedKeysCommand arguments. The 250-member study uses `--capacity 250
--refill-ms 250 --experimental-capacity`, separately from application
`max_sessions=250`, `launch_ms=100`, `experimental_capacity=true`. Policy changes
and resets refuse live reservations. This is not a supported deployment budget.

## State, restart and recovery

`/var/run/faction-auth/admission.state` is a fixed-size, single-link, helper-owned
0600 file, locked across validation/update and synchronized before emitting keys.
Never unlink or replace its live lock inode. A checksum detects torn/corrupt
records; malformed, writable, aliased or unreadable state fails closed. Root
`--inspect` prints aggregate capacity/refill/active/peak/admission/denial counts;
it does not print process identities. Its `launches` counter means newly reserved
key-admission monitors, not TCP accepts or SSH forks; `reuses` counts repeat key
lookups, and full/rate counters count denied key lookups. `--reset` requires no live recorded monitor.
It does not silently repair corrupt authority records.

A normal base-SSH listener stop preserves existing sessions. Restarting it keeps
those reservations authoritative, including monitors reparented to init; there
is no fresh allowance on top of them. Stop the backend to drain BBS frontends;
base SSH's unused-connection timeout handles no-channel clients. Recovery of
corrupt/missing state requires proving **all** old member transports gone before
reinitializing. A host reboot ends old processes; preserved monotonic state is
reset only when no recorded monitor remains. Never use an unconditional delete/
restart loop. Existing ordinary binaries/worker rollback remain independent.

The helper requires the default NetBSD `security.curtain=0` visibility so a missing
PID can mean death rather than hidden metadata. It refuses a hidden-process
policy; startup never lowers that setting. Support for stricter shared-host
visibility needs another explicit design. Its dedicated UID has authority over
admission state, and remains trusted. Base OpenSSH retains its configured error
logging; aggregate inspection does not anonymize OS logs.

## Evidence and limits

[Research 0036](../dev-docs/research/0036-ssh-admission.md) preserves the failed
inetd-mode listener candidate and current native evidence. That candidate completed
functional 250-member runs but failed the unchanged stress guard twice; it is not
the selected path. Kernel-reported zombie RSS fields were not treated as unique
or current physical RAM. No SSH signal/cryptographic/credential setting was
weakened to remove those processes.

The selected collectors include the base listener, every SSH descendant and any
transient key helper in transport accounting. Whole-host/kernel/base-service
costs and the unchanged memory guard remain. RSS sums repeat shared mappings.
Independent helper/authority review, preauthentication fork/CPU floods, source
fairness/NAT penalties, IPv6/interoperability, complete active/soak budgets and
maintained host/timekeeping/recovery policy remain adoption gates. A bounded peer
can still deny availability. No supported 250-member or physical SBC claim follows.
Lua/doors still need separate launch authority.

# Restricted SSH development gateway

Updated 2026-10-20. The current application runs through ordinary OpenSSH terminal
clients on the tested NetBSD 11 ARM64 VM. This is an early, bounded development
gateway. Public-service hardening, a maintained appliance, capacity and physical
SBC support remain unvalidated. Keep real/private deployment decisions deliberate.

## Member experience and authority

Connect with an operator-admitted SSH key and request a terminal:

```sh
ssh -tt -p 2222 factionterm@YOUR_PRIVATE_HOST
```

Choose `[L]ogin`, `[R]egister` or `[Q]uit`. Registration creates a pending local
account and exits. Reconnect after local sysop approval. An approved login gets
this board's identity/welcome, ANSI arrows/Enter or direct hotkeys, room posts and
local command chat. `TERM=dumb ssh -tt ...` requests the plain fallback. One BBS
authentication/registration attempt is allowed per admitted connection.

SSH keys grant transport admission, not a BBS identity or approval. The shared
`factionterm` OS account never becomes a community account; people enter their
own BBS handle/password. Keys are not yet bound one-to-one to BBS accounts. BBS
revocation stops future checked actions/current chat and denies community access
on reconnect; it does not delete an SSH key or prevent requesting a new pending
registration. Manage transport keys and membership separately.

The sysop can use role-checked BBS administration after BBS login. Bootstrap,
filesystem-owner recovery and private export/restore remain separate local-owner
commands. The `faction` executable must **never** be the SSH forced command.

| Process | OS identity and access |
| --- | --- |
| OpenSSH | Base OS transport and configured key policy; administrator-controlled. |
| `faction-ssh` | Dedicated terminal UID. No database dependency, state path or host-command parser; connects only to the configured local handoff socket. |
| `faction-broker` | Non-root board owner. Accepts one bounded terminal handoff and starts a fixed worker/config; no owner-operation RPC. |
| `faction-member` | Trusted board-owner worker. Opens private SQLite state, performs BBS login/approval checks and runs shared terminal UI. Its entry point has no owner-console command parser. |
| `faction` | Local owner console for initialization, recovery and snapshots. No direct member invocation. |

The terminal UID cannot read the `0700` state directory/`0600` database. The broker
checks the handed-off kernel descriptor is a TTY owned by the configured terminal
UID; the frontend checks a protected socket owned by the configured backend UID.
Local socket directory/group permissions are part of this boundary. They do not
prove BBS membership or provide a process sandbox. The board owner/root still
controls all local data; a compromise of a trusted worker has that owner's rights.
All frontends share one terminal UID, so code execution as that UID could affect
other frontends/TTYs. No hostile-code isolation or anonymity guarantee follows.

## NetBSD owner setup outline

Build the workspace with Rust 1.96+ and a native C compiler; both bundled SQLite
and the small NetBSD descriptor bridge use C. Run tests first. These commands
describe deliberate root setup **on the target NetBSD system**, not an installer
or instructions to change the developer host. Use a fresh dedicated SSH instance;
preserve your existing administrative access and review local account policy.

```sh
groupadd factionipc
useradd -g factionipc -d /home/factiond -s /bin/sh factiond
useradd -g factionipc -d /home/factionterm -s /bin/sh factionterm
install -d -o factiond -g factionipc -m 700 /home/factiond /var/db/faction
install -d -o root -g wheel -m 755 /home/factionterm /usr/local/libexec /etc/ssh/faction
install -d -o factiond -g factionipc -m 710 /var/run/faction
install -o root -g wheel -m 755 target/release/faction target/release/faction-member target/release/faction-ssh target/release/faction-broker /usr/local/libexec/
su - factiond -c '/usr/local/libexec/faction --state-dir /var/db/faction init'
```

Keep both identities out of unrelated groups. The terminal home and authorized
keys must be administrator-controlled; members must not edit shell startup files,
SSH rc files, environment files, programs or configuration. The `/bin/sh` account
shell exists for OpenSSH forced-command dispatch. It does not grant an interactive
host shell through the tested SSH policy. Use the exact restricted instance for
that account; another permissive host SSH daemon must not admit it. The disposable
test VM's separate administrative daemon accepts only its temporary admin key.

Put admitted public keys in `/etc/ssh/faction/authorized_keys`, one per line with
`restrict,pty` before the public key. Use root ownership and mode `0644` (public
keys are not secrets). Never copy a member's private key to the BBS. The dedicated
daemon uses the target's existing `/etc/ssh/ssh_host_ed25519_key`; protect it and
verify the expected host key on clients through your trusted administrative path.

Render [the SSH template](../deploy/netbsd/sshd_config.in) with the actual backend
UID, then inspect/test before starting a separate daemon:

```sh
faction_owner_uid=$(id -u factiond)
sed "s/@FACTION_OWNER_UID@/$faction_owner_uid/" deploy/netbsd/sshd_config.in > /etc/ssh/faction/sshd_config
chmod 600 /etc/ssh/faction/sshd_config
/usr/sbin/sshd -t -f /etc/ssh/faction/sshd_config
/usr/sbin/sshd -T -f /etc/ssh/faction/sshd_config
```

The template deliberately binds **loopback**, port 2222. For remote use, choose an
explicit intended private interface/address and matching firewall policy, inspect
effective settings, then test from the intended network. Never replace it with a
wildcard as an implicit setup step. An invitation-only private network is optional;
no overlay provider or public HTTP service is required or installed by Faction.
Private-interface/overlay policy is the operator's responsibility and has no
automated installer/accidental-exposure audit yet.

Start the broker under the board owner using the terminal account's numeric UID:

```sh
faction_terminal_uid=$(id -u factionterm)
su - factiond -c "/usr/local/libexec/faction-broker --socket /var/run/faction/terminal.sock --terminal-uid $faction_terminal_uid --worker /usr/local/libexec/faction-member --state-dir /var/db/faction"
```

It runs in the foreground. For managed startup, use the opt-in
[NetBSD service scripts](service-lifecycle.md) after validating the dedicated SSH
configuration. The broker refuses root, the same terminal/backend UID, unsafe
state, a live socket or a writable/nonregular worker. Its locked pidfile permits
conservative recovery of an owned disconnected socket; never delete a live lock.
All parent paths/executables remain trusted administrator/owner configuration.

Append `--theme /ABSOLUTE/OWNER/PACK --season /ABSOLUTE/OWNER/OVERRIDE` to the broker
for [sysop packs](theme-packs.md), in that order. Paths go only from trusted broker
configuration to the worker; members cannot select filesystem paths. Pack ownership
must pass the existing owner checks. Invalid worker pack configuration closes that
session. An empty configuration uses built-in text, without bundled scene art.

## Limits, disconnects and evidence

- Broker defaults to eight simultaneous handoffs/workers, burst eight launches, then one
  replenished launch token per five seconds. OpenSSH handshake caps are separate.
  No per-key/account/IP fair queue or capacity for 100–250 members is claimed.
- Handoff is 72 bytes plus exactly one terminal descriptor, with a two-second
  receive deadline after its serialized admission starts. Oversized/extra/truncated
  rights and non-TTY/wrong-owner descriptors fail. No client command/state path is
  transferred. TERM is at most 63 characters from a small ASCII alphabet.
- Supervisor threads use 128-KiB stacks. Each member still has an application
  process plus OpenSSH/frontend overhead. Whole-host shared/private memory and
  busy-board costs remain to be measured before increasing the cap.
- Line/password prompts have a 120-second receive budget. The broker defaults a
  session to 30 minutes (owner may shorten with `--session-seconds 1..1800`);
  OpenSSH also has configured handshake/channel/keepalive limits. These development
  maxima are not a complete time policy.
- Disconnect closes the frontend control socket. The broker permits up to one
  second for normal PTY cleanup, then kills/reaps the worker; supervisor polling
  adds scheduling delay. Abrupt kills can leave a chat lease until the existing
  60-second expiry/reclamation rules apply. Normal logout restores terminal state.
- A blocked output cannot hold a database transaction; the absolute session limit
  bounds worker lifetime while the broker is running. Worker parent monitoring
  now handles broker loss, and a synthetic blocked writer exercises the absolute
  deadline. Production output deadlines and the complete
  failure campaign remain separate validation.

Native NetBSD tests passed for real registration/pending/approval, room posting,
two simultaneous chat members/draft preservation, current revocation, resize/plain
fallback, observable worker disconnect, unknown key and combined exec/SFTP/TCP/Unix
forwarding denials. Extra/truncated local rights were rejected with unchanged
broker descriptor count. Agent/X11/tunnel denials are effective configuration checks,
not individually exercised wire attacks. SSH crypto is provided by the OS; there
is no implemented Telnet, external IRC wire endpoint, federation or browser portal.

See [native application evidence](../dev-docs/research/0024-restricted-ssh-netbsd.md),
[ADR 0012](../dev-docs/decisions/0012-restricted-ssh-and-terminal-handoff.md) and
[private recovery](export-restore.md). Keep backups/themes separate and validate
account policy before cutting over a restored board.

## Credential work and scaling experiments

Each board now permits two simultaneous password derivations across cooperating
owner/member processes. Excess credential work closes that attempt with
`credential work busy; reconnect later`; reconnect and try again. Local approval
still applies after successful password verification. Argon2 costs remain unchanged.
The private state directory contains persistent `credential-0.lock` and
`credential-1.lock` coordination files; never unlink/replace them while any board
process may be running. They contain no credentials and are recreated for a
fresh restored board; owner snapshot data does not package live locks.

Normal service remains eight workers and the original launch cadence. Higher
limits are explicit owner-only research controls, documented in the
[memory/scaling experiment](../dev-docs/research/0027-memory-scaling.md), with
whole-VM headroom guards and separate OS resource limits. They do not establish a
supported member count or configure the host automatically.

## Shared-owner research boundary

An [isolated shared-board prototype](../dev-docs/research/0029-shared-board-and-credential-peaks.md)
now compares readiness terminals with per-member workers, using the same separate
terminal UID and unmodified OpenSSH frontend. It is not installed by these setup
instructions. Its incomplete theme/plain/resize/navigation behavior must pass
parity and operational gates before member deployment. Keep the normal gateway
at eight workers; higher research counts are not supported capacity.

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
| `faction-tty` | Native NetBSD terminal frontend; dedicated terminal UID. No database dependency, state path or host-command parser; connects only to the configured local handoff socket. |
| `faction-service` | Non-root shared board owner; full bounded member UI, internal connection-bound sessions and fixed credential workers. No owner-operation RPC. |
| `faction-broker` | Rollback non-root board owner. Accepts one bounded terminal handoff and starts a fixed worker/config; no owner-operation RPC. |
| `faction-member` | Trusted board-owner worker. Opens private SQLite state, performs BBS login/approval checks and runs shared terminal UI. Its entry point has no owner-console command parser. |
| `faction` | Local owner console for initialization, recovery and snapshots. No direct member invocation. |

The terminal UID cannot read the `0700` state directory/`0600` database. The broker
checks the handed-off kernel descriptor is a TTY owned by the configured terminal
UID; the frontend checks a protected socket owned by the configured backend UID.
Local socket directory/group permissions are part of this boundary. They do not
prove BBS membership or provide a process sandbox. The board owner/root still
controls all local data; a compromise of the shared owner or a trusted worker has that owner's rights.
All frontends share one terminal UID, so code execution as that UID could affect
other frontends/TTYs. No hostile-code isolation or anonymity guarantee follows.

## NetBSD owner setup outline

Build the workspace with Rust 1.96+ and a native C compiler; both bundled SQLite
and the small NetBSD descriptor bridge use C. Run tests first (`cargo test --workspace --release --locked` and
`cargo build --workspace --release --locked`). Build the maintained native
frontend with `python3 tools/build_terminal_frontend.py --output target/release/faction-tty`.
Its small C boundary passes native descriptor/TTY and static-analysis checks; it
requires manual bounds/lifetime review and currently targets NetBSD only. These commands
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
install -o root -g wheel -m 755 target/release/faction target/release/faction-member target/release/faction-ssh target/release/faction-broker target/release/faction-service target/release/faction-tty /usr/local/libexec/
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

Use the [NetBSD shared service guide](service-lifecycle.md) to render the strict
root-owned configuration and start the shared backend, then the dedicated SSH
daemon. The maintained template forces `faction-tty`; it never invokes the owner
console. The service refuses root, equal terminal/owner identities, unsafe state,
invalid packs or live/unsafe endpoint replacement. Parent paths and binaries are
trusted operator configuration. Keep `terminal.pid` and credential lock inodes
in place while any backend is active.

The `theme` and `season` settings select [sysop packs](theme-packs.md) once at
startup. Empty configuration uses built-in text without bundled scene art.
Worker rollback remains available through the same guide and database schema.

## Limits, disconnects and evidence

- Default shared service admits eight terminals, with burst eight then one
  replenished launch token per five seconds. Higher/faster limits require an
  explicit experimental setting. OpenSSH handshake controls are separate; there
  is no complete per-key/account/IP fair queue or authenticated transport cap.
- Handoff is 72 bytes plus exactly one kernel terminal descriptor, with a
  two-second receive budget. Extra/truncated rights, non-TTY/wrong-owner
  descriptors fail. No member ID, role, state path or command is transferred.
  TERM is bounded to 63 allowed ASCII characters.
- One board owner shares validated themes and bounded session state. A handoff
  thread and two fixed credential threads replace per-member application workers;
  two cross-process kernel slots still cover every Argon2 operation. Base OpenSSH
  and one small native frontend per terminal remain in all memory accounting.
- Prompts expire after 120 seconds; the configured session lifetime is 1–1800
  seconds. Nonempty output batches expire after five seconds, with bounded encoded
  output/input and rotating work. No transaction waits behind terminal output.
- Disconnect drops that connection's opaque session/lease and restores terminal
  state. Shared owner failure affects every attached member; worker rollback has
  a smaller application failure scope. Abrupt termination can leave presence until
  the existing 60-second lease expiry. Complete hostile-code confinement remains
  unimplemented; the trusted service has owner access to the private database.

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

Normal shared service remains eight sessions and the original launch cadence.
Higher/faster limits require explicit experimental configuration. The maintained
service does not install whole-VM guards, sysctl changes or a tuning profile.
See [service setup and rollback](service-lifecycle.md).

## Evidence and remaining work

The worker, shared-owner and full-UI comparisons remain historical executed
research: [0029](../dev-docs/research/0029-shared-board-and-credential-peaks.md),
[0031](../dev-docs/research/0031-repeatable-headroom.md), and
[0032](../dev-docs/research/0032-shared-ui-themes.md). Their
[source and result bytes](../dev-docs/evidence/README.md) remain verifiable.
The maintained [shared service](shared-service.md) supplies the current setup.
Short 250-member VM observations require an explicit experimental allocator and
pageable-NTP profile, with all base services retained. Ordinary setup never
applies it. Stronger confinement, maintained image/timekeeping policy and broader
active workloads/soak/authenticated-transport admission remain #55/#20/#28 work.

The separate [experimental key-admission profile](experimental-admission.md)
adds an authenticated reservation cap and global key budget using base SSH's
AuthorizedKeysCommand. It requires dedicated installation and keeps ordinary
rollback independent. This is not a total TCP or pre-fork rate cap; local BBS
approval and all ordinary SSH restrictions still apply.

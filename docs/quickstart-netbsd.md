# Run your first Faction board on NetBSD

The small-board path runs the existing shared Rust backend, SQLite storage and
base OpenSSH. It provides approved accounts, ANSI menus/themes, message rooms and
local chat. Start with **eight connected terminals**. This is a hobby preview;
there is no supported release or prebuilt appliance yet. Native setup is tested
on NetBSD 11 ARM64 in a VM. Physical SBCs and other NetBSD ports remain untested.

No HTTP, external database service, experimental admission library, setuid
frontend, NPF profile or host tuning is needed. The two password-work slots and
current Argon2 parameters stay unchanged.

## Build on the target

Use a dedicated NetBSD host/VM, Rust 1.96+, Python 3.10+, Git and the native C
compiler. Build as a regular user from this repository:

```sh
cargo build --workspace --release --locked
python3 tools/build_terminal_frontend.py --output target/release/faction-tty
python3 tools/build_service_daemon.py --output target/release/faction-daemon
```

The helper below is for a **first installation**. It refuses existing Faction
accounts or deployment paths. For an existing manual installation, keep using
[the service guide](service-lifecycle.md); this helper is not an upgrader.
Keep your administrative SSH session/access separate from the BBS.

## Install without starting a listener

Copy the **public** SSH key of your first invited member/sysop to
`/root/invited.pub`. Use plain OpenSSH `.pub` lines, one per key; do not copy
private keys. More invited public keys can share this file. The installer adds
`restrict,pty` itself and accepts no member-supplied SSH options.

As root, in the built repository, inspect the read-only preflight first:

```sh
python3 tools/install_netbsd.py --authorized-keys /root/invited.pub
python3 tools/install_netbsd.py --authorized-keys /root/invited.pub --install
```

The default listener address is **127.0.0.1**, port **2222**. To host on an intended
private interface, pass the same explicit `--listen-address` and optional `--port`
to both commands. Wildcard addresses are rejected. The helper does not configure
a private network or firewall; use your chosen private interface and host policy.
If the host SSH keys do not exist, generate them first with `ssh-keygen -A`.

The helper installs the owner `factiond`, restricted terminal account
`factionterm`, private state/configuration, the native frontend, shared backend,
worker rollback binaries and opt-in rc.d scripts. Both services stay **disabled**.
It does not edit the administrative SSH daemon. That daemon must not admit the
`factionterm` account through a permissive alternate configuration.

The installed board has these useful paths:

| What | Path |
| --- | --- |
| Private board/database | `/var/db/faction` |
| Editable board name, welcome, palette and menu | `/var/db/faction/themes/base/theme.toml` |
| Service settings (eight-session default) | `/etc/faction/service.toml` |
| Invited SSH public keys | `/etc/ssh/faction/authorized_keys` |
| Dedicated BBS SSH settings | `/etc/ssh/faction/sshd_config` |
| Backend startup diagnostics | `/var/log/faction-broker.log` |

## Create your sysop and start the board

In a real terminal, initialize the first locally approved sysop. Passwords are
hidden and never passed as command arguments or environment variables:

```sh
su - factiond -c '/usr/local/libexec/faction --state-dir /var/db/faction init'
```

As factiond, edit `themes/base/theme.toml` inside `/var/db/faction` to set your
board name and welcome. Add your own licensed/original login art using the
[theme guide](theme-packs.md). Arrows/Enter and displayed hotkeys work together;
manual [seasonal packs](menus-and-themes.md) are optional. No archived scene art
is bundled. Restart the backend after editing its loaded theme.

As root, start the backend and then the dedicated member SSH service:

```sh
/etc/rc.d/faction_broker onestart
/etc/rc.d/faction_sshd onestart
/etc/rc.d/faction_broker onestatus
/etc/rc.d/faction_sshd onestatus
```

On the server itself, connect with your invited key:

```sh
ssh -tt -p 2222 -i ~/.ssh/your_member_key factionterm@127.0.0.1
```

From another machine, use the explicitly configured private address instead.
Verify the host key through your trusted administrative path. SSH admits the
invitation key; the BBS then asks for its **own** handle/password. Log in with the
sysop you initialized. You should see the board identity, welcome and menu.

## Invite a friend and have a conversation

1. Add their public key to the root-owned authorized-keys file using the form
   `restrict,pty ssh-ed25519 BASE64_PUBLIC_KEY`. Keep mode `0644`; no restart is
   needed for key-file edits. Do not reuse this account for host administration.
2. They connect to `factionterm` with their own SSH key and choose **R** to register.
   Their new BBS account remains pending and cannot see community content.
3. Log in as sysop, choose **S**, then **A**, and enter their handle to approve them.
4. They reconnect, choose **L**, and enter their local BBS credentials. Each person
   has a separate handle even though SSH uses the common restricted terminal UID.
5. Choose **R** to find rooms, **P** to post and **N** to read new messages. The
   sysop's **S / C** creates another room. **C** enters local chat; type `/help`,
   talk in `#lobby`, and use `/quit` to return to the menu. **Q** logs out.

Try two terminals so you can see a peer's chat arrive. Lobby chat and all room
content stay on this BBS. Add the optional [Lua arcade](lua-doors.md) for a sysop-defined Doors menu,
Signal Hunt and Neon Salvage. Files, private mail, directory listings, native
legacy-game hosting, federation, payments and browser access remain backlog work.
The ordinary path keeps base SSH abuse protection; rapid reconnects may need a
pause. It has no blanket arbitrary-code sandbox or 250-member capacity promise.

## Keep it running and keep a backup

After checking the board, opt into startup at boot:

```sh
printf 'faction_broker=YES\nfaction_broker_backend=shared\n' > /etc/rc.conf.d/faction_broker
printf 'faction_sshd=YES\n' > /etc/rc.conf.d/faction_sshd
```

To stop or reload a theme, stop SSH admission, then the backend. Restart in the
opposite order. A backend restart disconnects members; announce maintenance.
Never delete live pidfiles or credential lock files.

```sh
/etc/rc.d/faction_sshd onestop
/etc/rc.d/faction_broker onestop
```

As the board owner, create a new private snapshot file:

```sh
su - factiond -c '/usr/local/libexec/faction --state-dir /var/db/faction export /home/factiond/board-backup.json'
su - factiond -c '/usr/local/libexec/faction verify-export /home/factiond/board-backup.json'
```

Backups contain private plaintext data and password hashes; protect them and
copy them off-host securely. Back up your theme/art and operator configuration
separately. Rehearse [restore](export-restore.md) into an empty state directory
before depending on a backup. See [service lifecycle](service-lifecycle.md) for
crash recovery, worker rollback and configuration checks.

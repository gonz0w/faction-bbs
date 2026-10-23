# Private terminal playground

The development playground is a persistent NetBSD 11 ARM64 VM for trying current
Faction menus in an ordinary terminal such as Ghostty. It uses the existing
restricted SSH entry point and separately approved BBS accounts. Its admin and
member forwards bind to `127.0.0.1` on the developer host. No public service is
started by these tools.

## Connect and explore

An operator first prepares a private software fixture. For the currently prepared
checkout, its ignored `local/demo/CURRENT_PREVIEW` file identifies that directory.
From the repository:

```sh
preview_dir="$(cat local/demo/CURRENT_PREVIEW)"
cat "$preview_dir/login.txt"
ssh -F "$preview_dir/ssh_config" faction
```

Choose **L** at the gateway, then enter the BBS handle/password from the private
login file. `sysop` has BBS settings authority; `cam` is an approved ordinary
member. The synthetic neighbor `nightowl` is seeded content, not an online person.
These local demo accounts share one generated demo password. They are not a
production account policy. The member SSH identity never grants a host shell.

Start with an 80-column or larger window. After the Nightwire cover and welcome,
use arrows + Enter or the displayed hotkeys:

- **R**: five named rooms with authored introductory posts; select a room to read
  or reply. At the body prompt, `/edit` opens the writing desk.
- **W**: the local graffiti wall; **M**: member pages and dated `.plan` updates.
- **C**: local chat; `/join #afterhours` or `/join #bsd`, then `/quit` to return.
- **D**: existing Lua arcade, including Signal Hunt and Neon Salvage. Game expansion
  and original DOS hosting remain separate work.
- **T**: native text/color/graphics samples. Classic ANSI remains the default;
  capabilities and opt-in settings control richer output, not the terminal brand.
- **S**: local BBS settings for `sysop`; **Q**: log out.

The prepared preview enables these features explicitly; installations keep their
existing defaults. Files/SFTP, home mail and federation are not provisioned by this
playground batch. Native-device encrypted mail is still separate from SSH.
Compact 40x18 terminals retain the classic card/menu fallback.

## Stop, return and preserve work

```sh
python3 tools/playground.py --state-dir "$preview_dir" status
python3 tools/playground.py --state-dir "$preview_dir" stop
python3 tools/playground.py --state-dir "$preview_dir" start
```

`status` reports the recorded VM identity, not application readiness. After `start`,
allow the guest to boot before connecting. `connect` is an alternative to the SSH
command. A live test may need five seconds between admissions; normal eight-session
and credential limits are unchanged.

Stop requests guest shutdown through its pinned private admin connection. It does
not force-kill an unresponsive VM. PID, process birth and command checks refuse a
reused or unrelated PID. Start refuses a partial surviving pair or open disk and
keeps the same loopback ports; a port collision requires operator attention. A
private lock serializes lifecycle commands. Stop/start never seeds or resets data.
The disk, accounts, posts, wall, profiles, keys and logs remain in private storage.
This preserves a testing session, not a verified backup.

The copy-on-write overlay depends on the earlier saved base image and kernel.
Keep those files: copying only the overlay does not make a standalone image.
The saved base is not booted or modified; the preview uses it as backing storage.
Never run both instances with the same writable disk. Complete board backup and
fresh-path recovery remain the workflows in [backup checks](backup-checks.md).

## Preparing another fixture

This tool controls an **already prepared** macOS/Apple Silicon HVF software fixture;
it is not a general NetBSD image downloader, installer or supported appliance.
Use [NetBSD setup](quickstart-netbsd.md) and existing software research fixtures
for accounts, pinned SSH trust, native binaries and optional Lua installation.
Preparation needs a distinct writable overlay, matching ARM64 kernel, owner-private
state directory and the following private files:

- `disk.qcow2`, `qemu.pid`, `controller.pid`, `serial-controller.py` and `member-port`;
- `ssh-admin.json` and `member-argv.json`, containing the explicitly prepared pinned
  loopback SSH argv, plus the referenced private keys and known-host files.

Adopt that running fixture once with:

```sh
python3 tools/playground.py --state-dir "$preview_dir" register --kernel "$preview_kernel"
```

Registration checks the owned disk/port process identity and SSH endpoint/pinning
before publishing its private lifecycle records. Restarts use a fixed 2-core,
1-GiB QEMU command, loopback-only forwards and a serial companion. The companion
applies the existing fixture's `disable vmt`/`quit` userconf boot workaround. Other
hosts, kernels and boot recipes need their own validation.

For sample community data, build the `faction-core` example `playground_seed` and
run it **as the dedicated board owner**, outside the member entry point:

```sh
playground_seed --new-private-state "$private_parent/new-board" < "$private_password_file"
```

The parent must already be owned/private; the destination must not exist. The
example refuses existing paths/symlinks, shared parents and malformed passwords.
It uses current core bootstrap/approval/post/page/wall/chat operations and prints
no password. A failed partial seed is retained for inspection and is never silently
reseeded. Stop/drain the backend before configuring it to use the resulting board.
Keep credentials, transcripts, SSH profiles and images outside Git. Remove only
an explicitly selected, stopped private preview when it is no longer needed.

# Native terminal client

Status: optional working Unix preview, validated on macOS and NetBSD 11 ARM64
software VM. See [ADR 0033](../dev-docs/decisions/0033-native-terminal-files.md)
and [validation](../dev-docs/evidence/native-client-netbsd/README.md).
The client provides integrated file transfers, recent room/chat file links and an
optional [local board launcher](board-launcher.md) for independent configured communities.
Home identity, XMPP and encrypted mail are **not implemented** in this increment.

## Connect

Build with Rust 1.96+, a native C compiler and base OpenSSH. On NetBSD the small
client-only PTY bridge uses base libutil; it is never linked into the BBS owner.

```sh
cargo build --release --locked -p faction-client
./target/release/faction-client --profile /absolute/private/board.toml
```

Create a local, user-owned `0600` TOML profile. Replace the example values with the
sysop's member SSH and separate file-service endpoints and your own absolute paths:

```toml
version = 1
host = "board.example"
port = 2222
file_host = "board.example"
file_port = 2223
identity = "/home/member/.ssh/faction"
public_key = "/home/member/.ssh/faction.pub"
known_hosts = "/home/member/.ssh/faction_known_hosts"
```

The preview accepts Ed25519 admission/transfer keys. The identity must be usable
without an interactive SSH prompt: load a passphrase-protected key into your local
SSH agent first. `identity` may point to its public key when using a loaded agent.
The client forwards no agent, disables SSH escape commands/forwarding/local commands,
and ignores personal SSH config. The SSH account names are fixed `factionterm` and
`factionfiles`; local BBS login and sysop approval are still required independently.

Enroll **both endpoints' host keys** in the specified known-hosts file after
verifying their fingerprints with the sysop through a trusted channel. A raw
network key scan alone does not verify identity. Nonstandard ports use OpenSSH's
`[host]:port` notation. Unknown or changed keys fail closed; there is no interactive
accept-new-host prompt. Paths in the profile must be absolute, without parent
traversal or OpenSSH expansion characters. The profile must be private, and its
referenced files must be regular, owned by you or root, without group/world writes.

On the board, the sysop must install and explicitly start the optional
[restricted file service](file-areas.md). The client enables no listeners, changes
no host policy and does not bypass a private-network deployment.

## Files without leaving the BBS

Log in normally, see the board's ANSI identity and select **[F]iles**. The client
negotiates only at that screen, then shows its own local Files menu. Arrow keys,
Enter and displayed hotkeys work:

- **[A]reas** and **[L]ist** show permission-filtered, paginated metadata.
- **[U]pload** asks for a local path, area, safe remote filename and description,
  then explicit confirmation. Progress appears in place; uploads await sysop review.
- **[D]ownload** accepts a file ID, full `faction-file:` reference or recent `#number`.
  Choose a new local path and confirm; the client checks size and SHA256 before saving.
- **[I]nfo** inspects a published file. **[B]ack** returns to the still-open BBS session.

A room/chat reference seen by this client appears in its bounded recent-link list
(eight references). Select its number in Download instead of copying an object ID.
Links are resolved **only on the connected board**, with current approval and area
permissions rechecked. There is no remote-board discovery or automatic fetching.

Transfers use a second restricted SSH/SFTP connection managed by the client while
the terminal session stays connected. No second terminal, shell command or pasted
public key is needed. Ctrl-C during transfer progress cancels the operation; a
failed operation releases its scoped grant. Any partial upload remains private and
charged until sysop removal/GC under existing file policy. A network failure can
require returning to/reconnecting the board; transfers do not resume automatically.

Downloads create private same-directory temporary files and never overwrite an
existing destination, including symlinks. On ordinary errors partial local files
are removed. Choose a directory owned by you without group/world write permission;
this is an intentional preview restriction. Transfer forms currently accept ASCII
paths; remote names are bounded ASCII basenames. Limits remain two server streams,
32 KiB chunks, 16 MiB/file and five-minute grants. No extra server session/thread
ceiling, supported-capacity or physical SBC claim follows from this preview.

Remote text is filtered for terminal controls. Bounded file frames are decoded separately from the optional exact-command
graphics gate; terminal output cannot choose local paths, hosts, commands or start a
transfer. Normal keyboard actions and local confirmation choose those operations.
The native Files screen is local and currently has a fixed palette; the board's
sysop themes/art continue outside it. Client theming can evolve independently.

## Plain SSH and future mail

Ordinary SSH remains supported. Its Files screen retains manual key/grant/SFTP
commands, including `link OBJECT_ID` and `cancel TRANSFER_ID`. It does not acquire
local filesystem access from the native protocol.

The optional [local mail vault](local-mail-vault.md) now adds a separate trusted
local screen, encrypted drafts/device pins, hidden unlock and encrypted
backup/recovery. It opens no SSH connection. Native inbox/send, OMEMO device
keys/ratchets and roaming login remain unimplemented under #90/#91.
[Encrypted attachments](private-mail.md#encrypted-attachments) are separately scoped:
current sysop-readable file areas are not a substitute for client-encrypted blobs.

## Optional modern image preview

Append `--kitty` after the profile to opt in locally. Before SSH, a dedicated
bounded query checks the member terminal. Only a positive response enables the
exact fixed-asset [Kitty graphics gate](kitty-graphics.md); ordinary native
filtering stays enabled by default, and sync remains filtered. After BBS approval
choose T/G. This does not permit arbitrary remote APC, local image-file loading,
member art or file/mail authority. C/D/reconnect and teardown preserve fallback.

The [real terminal matrix](terminal-compatibility.md) now tests the full macOS
native-client/restricted-NetBSD-member-SSH path in named emulator/multiplexer
profiles. Default filtering and opt-in image retention/deletion pass; sync remains
filtered. This adds no mail, image-import or universal visual certification.


The member-owned [board launcher](board-launcher.md) optionally opens the existing
[local offline vault](local-mail-vault.md) with M between board visits. Its store
path is an explicit private catalog setting. No SSH child exists during unlock;
network private mail and roaming authentication remain follow-on work.

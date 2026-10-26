# Local encrypted checkpoint console

The optional native client now has member-device screens for the experimental
[v2 encrypted checkpoint backend](../dev-docs/native-mail-checkpoints.md). They
open no SSH connection, mailbox listener or XMPP session. There is no network
inbox, compose/send command or automatic synchronization in this console.
Existing [v1 draft/pin commands](local-mail-vault.md) and the board picker remain
separate. Keys, unlock secrets and private content stay on your device.

## Build and start

Use Rust 1.96 and a native C compiler with OpenSSL 3 headers/libraries. On NetBSD,
select the base OpenSSL explicitly:

```sh
OPENSSL_INCLUDE_DIR=/usr/include OPENSSL_LIB_DIR=/usr/lib \
  cargo +1.96.0 build --release --locked -p faction-client --features native-mail
./target/release/faction-client --mail-checkpoints "$HOME/.faction-mail-v2" init
./target/release/faction-client --mail-checkpoints "$HOME/.faction-mail-v2" open
```

On other supported build hosts, omit the OpenSSL path assignments when the native
build can discover the installed OpenSSL 3. The feature is disabled in ordinary
builds. Run it in a trusted local terminal; a remote board cannot select this
command or your store path. The parent directory must already exist; initialization
requires a new absolute directory, with no symlink components.

Initialization asks for a locally claimed `user@home.example`, a local device ID
from 1 to 2147483647, and a new 12–128 character printable ASCII passphrase twice.
Input is hidden. Neither the address nor a device key grants home-account ownership,
BBS approval or a role. Save the new recovery secret privately before acknowledging
the display. Clearing the viewport does not erase terminal scrollback or recording.
No password or recovery secret belongs in arguments, environment variables or logs.

## Inspect and maintain your store

Use the displayed hotkeys. Plain ASCII output is the default. Append `--unicode`
after the action only as a deliberate local rendering override; controls, bidi
marks and invisible formatting remain escaped. This does not enable graphics or
trust a remote terminal-capability claim.

| Key | Local action |
| --- | --- |
| P | Inspect your full public Ed25519 identity, device ID and enrolled peer. |
| V | Enroll one peer after independently comparing the full typed `ed25519:` identity and device ID. |
| H | Read authenticated local history and retained outbound operation IDs; upgraded received records offer explicit local acknowledgement. |
| E | Deliberately enable encrypted local read tracking after confirmation. |
| O | Inspect durable pending ciphertext IDs and bounded metadata. |
| T | Explicitly retire one pending operation or all pending ciphertext. |
| R | Revoke the enrolled peer locally. This does not revoke their remote device or copies. |
| F | Generate a fresh local device after explicit pending retirement; verify the peer again. |
| B | Create an encrypted history-only backup in a new absolute directory. |
| S | Rotate the current local passphrase and recovery secret. |
| D | Read preserved v1 drafts and Curve25519 pins, without turning them into native trust. |
| L | Lock and exit. |

A pending item means ciphertext remains available to the trusted backend caller.
It does **not** prove that another caller never delivered it before a crash.
This UI attempts no network delivery. Retirement records only a local decision;
it is not a delivery, read or exactly-once receipt. History and operation IDs
survive retirement. No message content is sent to presence, chat or sysop statistics.

The initial backend retains one verified peer, 64 history records and eight
pending operations, with a 30-day session lifetime. It rejects additional work
when full and never silently evicts history. Expired, revoked and history-only
states remain readable; they cannot silently restart old protocol state. A fresh
device preserves history and legacy records, so it does not empty a full history.
Full multi-device support and history-retention changes require separate work.

## Upgrade a v1 vault deliberately

First create and test a v1 encrypted backup using the old console. Then run:

```sh
./target/release/faction-client --mail-checkpoints "$HOME/.faction-mail" upgrade
```

Confirm the replacement, unlock the existing v1 file, choose a local device ID
and a new passphrase, then save the new recovery secret. Upgrade preserves drafts
and Curve25519 pins but rotates local encryption/unlock material. Legacy pins are
never Ed25519 enrollment. V1 commands and the existing picker cannot open the v2
file afterward; use this console. Old backups retain their old secrets and format.

## Recovery and failures

```sh
./target/release/faction-client --mail-checkpoints "$HOME/.faction-mail-v2" recover
```

Recovery-secret unlock durably marks the current store **history only** before
returning its content. It then requests a new passphrase and displays a new recovery secret.
Password reset or password reopen cannot clear that quarantine. Official backups
are also history only. To resume protocol work, deliberately retire pending
ciphertext, choose a different local device ID, and independently verify/enroll
the peer again. Old history remains readable; old ratchets do not resume.

Wrong secrets, malformed state, concurrent access, cancelled confirmation and
write errors close the screen. After an ambiguous write, drop the handle and
reopen the canonical store; a reset may already have published the new password.
There is no automatic temp/backup fallback or rollback to a usable old handle.
Ctrl-C/D and five minutes without keyboard input also exit and restore the TTY.
Do not remove a live lock file.

Backups, arbitrary manually copied active files, terminal recordings and recipient
copies cannot be revoked by changing your password. Arbitrary active-file rollback
is not detected. This is a bounded experimental implementation, not an audit,
physical storage certification or supported-capacity claim. See the
[format and recovery contract](../dev-docs/native-mail-checkpoints.md).

For deliberate direct home connections, use the separate default-off
[manual encrypted mail entry](online-mail.md). This offline console retains its
existing setup, recovery and inspection behavior.

The online console offers a deliberate [archive schema upgrade](archive-mail.md).
It retains the v2 encryption envelope and opts into payload schema 3. This offline
console can read that authenticated history, but opens no archive connection.

[Local unread mail](local-unread-mail.md) describes the separate explicit read-state
upgrade, received-only local counts and post-body `read` acknowledgement. Enabling
tracking opens no network connection. Older checkpoints stay unchanged until opt-in.

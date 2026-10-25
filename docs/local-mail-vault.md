# Local encrypted mail vault

Status: optional native-client **offline preview**, 2026-10-21. This increment of
[#90](https://github.com/gonz0w/faction-bbs/issues/90) stores encrypted local drafts
and manually verified device pins. It does **not receive, send or queue mail**,
generate OMEMO device keys, enroll devices on a server or authenticate a home
identity. The inbox remains disabled pending the native OMEMO adapter. The [local board picker](board-launcher.md#open-your-local-offline-vault-between-visits)
can open an existing vault between SSH visits when explicitly configured. [Mail direction](private-mail.md)
and [native client](native-client.md) describe the broader work.

## Start locally

Build the native `faction-client` with Rust 1.96 and a C compiler. Use a local
terminal on your own trusted device. The parent directory must already exist;
the vault directory must be new for initialization. Use an absolute path without
symlink components (`pwd -P` can help find the physical parent path).

```sh
cargo build --locked --release -p faction-client
./target/release/faction-client --mail-store "$HOME/.faction-mail" init
./target/release/faction-client --mail-store "$HOME/.faction-mail" open
```

Initialization asks for a full address such as `alice@home.example`, a new local
passphrase twice and then displays a random offline recovery secret. Save that
secret privately before clearing the screen. Your address is a **local claim**;
it grants no home identity or BBS access. This preview accepts lowercase ASCII
addresses with a dotted domain, not the complete internationalized XMPP syntax.

The local passphrase is hidden and must contain 12–128 printable ASCII characters.
Choose a strong, distinct secret. It is neither sent to a BBS nor obtained from a
server password verifier. A home/BBS password reset cannot unlock this vault.
Secrets are never accepted in command arguments, environment variables or remote
terminal commands. This mode creates no SSH connection or mailbox listener.

Arrows/Enter and displayed hotkeys work. Draft, view/delete, verify/remove device
pins, create an encrypted backup, or lock and exit. A draft is saved locally and
explicitly labeled **not sent or queued**. Bodies are one line, at most 4096 bytes;
there are at most 32 drafts and 64 pins. Private subject/body display is ASCII in
this preview. Unknown device IDs or changed fingerprints are never silently
accepted: compare all 64 hex characters of the Curve25519-format public identity key with the member through an independent
trusted channel, then explicitly confirm. These local records do not yet drive
an encryption adapter. The candidate OMEMO-2 library uses a different public-key
representation; existing pins must not be silently reinterpreted or compared as
raw wire keys. See [native qualification](../dev-docs/research/0053-native-omemo-reuse.md). Removing one is not remote device revocation.

## Backup and recovery

Choose `[B]` and supply a **new absolute directory** under an existing trusted
parent. The client copies only authenticated ciphertext, never a plaintext export
or the recovery secret. Keep an encrypted backup and its member-controlled secret
safe; test opening the backup with the normal `open` command before relying on it.
Existing directories/files are never overwritten by backup or initialization.

If you forget the local passphrase but retain the vault/backup and its recovery
secret, run:

```sh
./target/release/faction-client --mail-store "$HOME/.faction-mail" recover
```

Recovery authenticates the old vault, requests a new local passphrase and rotates
the local data key and recovery secret. Save the newly displayed secret. Old
passwords/recovery secrets stop opening the **current file**; old backups still
open with their old secrets. Deleting drafts/pins does not erase backups,
filesystem snapshots, terminal scrollback or copies others already hold.
Loss of both unlocking secrets means loss of this local data. No sysop recovery.

## Boundary and limits

Each vault is a member-owned 0700 directory with 0600 regular files, no symlink
components or multiply linked vault files. Exclusive locking refuses concurrent
access. Saves use private ciphertext temporary files, sync and atomic rename;
initialization/backup use create-only publication. Malformed, oversized, modified
or wrong-secret files fail before private content renders. Errors exit and relock,
rather than retaining unsaved changes to trust records. Five minutes without
keyboard input exits the mail screen; Ctrl-C/D also exits and restores terminal
settings. Core dumps are disabled for this mode.

The local persistence format uses Argon2id (64 MiB, three passes, one lane) and
XChaCha20-Poly1305 with random keys/nonces and authenticated headers. It is not an
OMEMO wire format or an identity proof. Local unlock costs run on the member's
device; server Argon2 settings, credential permits and session limits are unchanged.
The implementation uses best-effort zeroization; it does not promise locked RAM,
no swap, erasure of compiler copies, hostile-same-user isolation or a secure display
against a compromised OS/terminal. File size, existence and modification times
remain visible. This is a reviewed implementation slice, not an independent audit.

Host and native NetBSD software checks are recorded in the
[validation report](../dev-docs/research/0046-native-local-mail-vault.md).
Native online inbox/send/reply, atomic ratchet/history persistence, independent
OMEMO interoperability and home-loss/export policy remain #90. Connection-bound
home authentication/local approval remains #91; encrypted attachments remain #88.
No public file-area upload substitutes for a private attachment.


The current combined client preserves the standalone `--mail-store` commands and
adds the optional local picker entry. The earlier vault validation is immutable;
current integration checks are recorded in [GitHub issue](https://github.com/gonz0w/faction-bbs/issues/90).


The separate default-off [v2 checkpoint backend](../dev-docs/native-mail-checkpoints.md)
now stores native protocol/history/outbox state through a trusted local library
API. These v1 terminal commands do not initialize or open v2 vaults; explicit
upgrade rotates local unlocking keys and preserves v1 drafts/pins. Existing Curve
pins are not native Ed25519 enrollment. The optional [checkpoint console](checkpoint-console.md) now provides explicit
v2 setup/upgrade, device verification, history/outbox inspection and recovery.
Live inbox/send and XMPP remain separate #90 work.

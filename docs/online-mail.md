# Manual encrypted home mail

The optional native client connects from your device to your own home server for
an explicit encrypted-mail cycle with one independently verified peer. Compose,
keys, decryption and accepted history stay on your device. Run this console in a
trusted local terminal. It is separate from a remote BBS session. An explicit
[local picker E entry](picker-home-mail.md) can open the same console between visits.

The experimental client supports live v2 exchange and an explicit opt-in
[archive mode](archive-mail.md). Archive mode saves one page of retained encrypted
mail at a time with a durable encrypted cursor. A completed cycle does not prove
that the home retained every message or has no other mail waiting.

## Build and initialize

You need Rust 1.96, a native C compiler, and an existing
account on an invited home that supports the selected direct-TLS XMPP profile.
Build from the repository root. On NetBSD, select base OpenSSL explicitly:

```sh
OPENSSL_INCLUDE_DIR=/usr/include OPENSSL_LIB_DIR=/usr/lib \
  cargo +1.96.0 build --release --locked -p faction-client --features network-mail
./target/release/faction-client --mail-checkpoints "$HOME/.faction-mail-v2" init
```

On Linux, install OpenSSL 3 development files and omit the NetBSD assignments
when that installation is discoverable. macOS uses its native TLS backend.
The tested complete NetBSD builder had 2 GiB RAM; 1 GiB observations cover
runtime and cached incremental compilation, not a fresh complete build. `network-mail` is disabled by default and enables `native-mail`.
Ordinary SSH/files and existing local vault commands do not enable network mail.

Initialization requires a new absolute store directory with no symlink components.
Enter your existing home account, choose a device ID, set a local checkpoint
passphrase and save the recovery secret privately. The account must later match
the online configuration exactly. See the [checkpoint console](checkpoint-console.md)
for initialization, upgrade, backup and recovery details.

## Configure your home

Obtain the numeric endpoint, CA certificate and full SHA-256 certificate pin from
an independently trusted source for your home. The pin covers the server's entire
DER-encoded leaf certificate. Do not trust a certificate merely because the first
connection presents it. A changed certificate requires a deliberate configuration
update after verification.

Save the configuration and CA as separate files owned by your current OS user.
Each file must be a regular file with mode `0600` and one hard link. Each file's
immediate parent directory must be owned by you with mode `0700`. Use absolute
paths with no symlink components. The client rejects unknown configuration fields,
including password fields.

Example `home.toml`:

```toml
version = 1
account = "alice@home.example"
endpoint = "192.0.2.1:5223"
domain = "home.example"
ca = "/home/alice/.faction-home/ca.pem"
pin = "0000000000000000000000000000000000000000000000000000000000000000"
```

The address and zero pin above are documentation placeholders. Replace them with
your verified home values and actual private CA-file path. `endpoint` must contain
a numeric IP address and port. `domain` must equal the domain in `account`; it is
used for TLS name verification and XMPP addressing. The account syntax is the
current restricted lowercase ASCII `user@domain` profile. The pin is exactly
64 lowercase hexadecimal characters. No password belongs in this file, command
arguments, environment variables or logs.

Start with absolute paths:

```sh
./target/release/faction-client --mail-online \
  "$HOME/.faction-mail-v2" "$HOME/.faction-home/home.toml"
```

Append `--unicode` for deliberate Unicode rendering. Subject/body input accepts
UTF-8 in either mode; classic mode displays non-ASCII characters as escapes.
Terminal controls and invisible formatting remain escaped. The console first
asks for your **local checkpoint passphrase**. Each connection action asks
separately for your **home account password**, with input hidden. Home-password
entry accepts 1–128 printable ASCII characters; `--unicode` does not change
credential syntax.

The home password is used only in memory for that connection. The home necessarily
receives its own account password through the verified TLS channel. It receives
public keys, routing/timing/size metadata and ciphertext, but no local vault secret,
private device key, subject or body. Home-account authentication does not grant
BBS membership or approval.

## Exchange your first message

Choose which participant will send first. Only that participant prepares the
initial session with **K**. Preparing both sides independently can create competing
sessions; the client does not silently replace them.

1. On both devices, use **P** to inspect the full public Ed25519 identity and device
   ID. Compare the complete values through an independently trusted channel.
2. On both devices, use **V** to enroll the other account, device ID and exact
   `ed25519:` identity. A server device list or old Curve25519 pin is not verification.
3. On both devices, use **U** to publish the public device list and public prekeys
   already represented in the durable checkpoint. This sends no message content.
4. On the first sender, use **K** and confirm `prepare`. It checks the enrolled
   peer's advertised device and bundle, then starts the session once.
5. On the first sender, use **C**. Enter a nonempty subject of at most 128 UTF-8 bytes
   and a nonempty one-line body of at most 1024 UTF-8 bytes. Review the recipient and content, then
   type `queue` to save encrypted ciphertext durably. This action does not connect.
6. Coordinate a time to use **N** on both devices. Each cycle connects once, checks
   public device information, writes pending ciphertext and briefly receives mail.
   The receiver can inspect an accepted message with **H**, then compose a reply
   with **C** and run another coordinated **N** cycle. It does not need **K** after
   accepting the first message.

**O** lists durable pending operation IDs. A successful socket write leaves them
pending. Retry with **N** to send the same persisted ciphertext and operation ID;
repeated accepted frames do not add another received-history record.

Use **T** only for an explicit local decision to retire one operation or all pending
ciphertext. Retirement stops further retries from this store. It is not proof of
peer delivery, reading or exactly-once processing. History remains available.

## Opt in to retained offline mail

After publishing and verifying the peer, choose **G** in the local console and
confirm `upgrade`. This deliberately changes the encrypted checkpoint payload
from schema 2 to schema 3; a read-enabled schema-4 checkpoint retains its read
metadata. Older clients reject formats they do not support. Existing messages, private
keys, trust, recovery wrappers and pending ciphertext remain. Repeating G keeps
the existing cursor and epoch.

In archive mode, **N** sends pending ciphertext without announcing availability
or accepting live mail. **A** requests one own-home archive page with at most four
entries. Use A again deliberately for another page. The home must support the
selected MAM2/RSM profile and retain messages; the outgoing storage hint requests
retention but cannot enforce it. See the [archive guide](archive-mail.md) for
first use, full-store stops and stale-position recovery limits.

## Private attachments

With `network-mail`, **M** opens the default-off [private attachment desk](private-attachments.md) in this trusted local console. Opt in explicitly before queueing or fetching attachments; the migration preserves mail and read/archive state but older clients cannot open the new checkpoint format. Queueing is local and upload is a separate foreground action. The separate ciphertext spool needs its own backup and retention decisions. Ordinary text mail and Sync remain available.

For the optional `device-provider` build, an explicitly selected private local
profile uses a separate same-home mTLS binary transport for attachments. Profile
validation happens when an attachment action needs it, so ordinary mail remains
available if the provider is unavailable. Device fetch does not ask for your home
password; upload uses it only for the later ordinary encrypted-mail release. See
[private attachments](private-attachments.md) for configuration and failure handling.

## Stop, recover or resolve a refusal

Each connection has one attempt and a shared cooperative ten-second deadline.
The receive phase ends after two seconds without a stanza, subject to the overall
limits. There is no background reconnect. Ctrl-C/D cancels and locks; **L** locks
and exits between actions. The existing idle-terminal timeout also applies.

If a cycle stops, pending ciphertext remains. Check the private configuration,
home credential, enrolled peer and store state before retrying. After a checkpoint
write failure, close and reopen the canonical store. Never remove a live lock or
substitute a temporary/backup file to continue an ambiguous operation.

The store retains one peer, 64 history records and eight pending operations, with
a 30-day session lifetime. Full, expired, revoked and recovered/history-only state
requires deliberate maintenance; the client never silently evicts history or
resets a session. See [local recovery](checkpoint-console.md#recovery-and-failures).
A fresh device does not clear retained history. It also does not remove an old
published home device ID: this adapter refuses publication if the home advertises
another ID. Resolve that public-list state explicitly before publishing a new
identity, and independently verify the new identity with the peer.

Private content and counts do not enter BBS presence or sysop statistics. Reliable
bounded manual [archive catch-up](archive-mail.md) is optional; remaining mailbox
requirements stay in [#90](https://github.com/gonz0w/faction-bbs/issues/90),
background/private-chat UX under [#99](https://github.com/gonz0w/faction-bbs/issues/99),
roaming under [#91](https://github.com/gonz0w/faction-bbs/issues/91), and attachments
under [#88](https://github.com/gonz0w/faction-bbs/issues/88). Multi-device use, a full
XMPP/OMEMO conformance claim, a security audit and supported-capacity claims are
outside this adapter. See the [technical contract](../dev-docs/native-home-mail.md).

See [local unread mail](local-unread-mail.md) for explicit encrypted read decisions
and the parked native Files-desk entry. A stale local count never establishes a
complete home inbox, and no read receipt is transmitted.

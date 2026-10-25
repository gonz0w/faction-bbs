# Native mail vault and adapter boundary

Version: local container 1; [ADR 0035](decisions/0035-native-local-mail-vault.md),
[#90](https://github.com/gonz0w/faction-bbs/issues/90). This is persistence and
trusted local UI, not a replacement XMPP/OMEMO protocol.

## Exact byte format

| Offset | Size | Field |
| --- | --- | --- |
| 0 | 8 | ASCII `FACTVLT1` |
| 8 | 16 | Random local vault-generation identifier |
| 24 | 16 | Random Argon2id salt |
| 40 | 24 | Password-wrap XChaCha20-Poly1305 nonce |
| 64 | 48 | Encrypted 32-byte data key plus 16-byte tag |
| 112 | 24 | Recovery-wrap nonce |
| 136 | 48 | Encrypted data key plus tag |
| 184 | 24 | Payload nonce, fresh on every save |
| 208 | variable | Encrypted local payload plus tag |

Password key derivation is fixed Argon2id v1.3, m=65536 KiB, t=3, p=1, output32.
Recovery is a random full-entropy 32-byte key, displayed as 64 lowercase hex chars;
it does not pass through a password KDF. Wrapper AAD is ASCII
`Faction local mail vault v1`, a NUL, purpose (`password` or `recovery`), then
bytes 0..40. Payload AAD is bytes 0..208. All nonces come from OS randomness;
no nonce reuse/counter restoration requirement. All format/length checks precede
KDF/decryption. There is no file-controlled KDF parameter or algorithm negotiation.
No plaintext/key bytes are decoded or displayed on authentication failure.

All payload counts/field lengths are unsigned big-endian u32. Fields: home address
(length + UTF-8 ASCII bytes), pin count, pins (address field, device u32, public
fingerprint32), draft count, drafts (recipient/subject/body fields). Exact EOF is
required. Bounds: payload256KiB, pins64, drafts32, address254, subject128, body4096.
Address syntax is the documented restricted lowercase ASCII preview, not PRECIS.
Device IDs are 1..2147483647 per XEP-0384 setup. Pin fingerprints represent the
full Curve25519-format public identity key for the OMEMO-2 profile, not SHA256 of
an arbitrary string or a home authentication key. A future adapter must perform
required encoding conversion using reviewed library functions and prove interop;
manual records cannot by themselves establish possession or authenticated identity.

An exclusive owner-only lock file remains open for the vault lifetime. All path
components are opened descriptor-relative with NOFOLLOW. Private ciphertext temp
files use CREATE/EXCL/NOFOLLOW, fsync before atomic rename, and directory sync;
initialization/backup publish by a no-clobber link. Cleanup removes temporary names
on ordinary error. A crash may retain only a ciphertext temporary file. Current
vault files must be owner-only regular single-link files. Do not unlink a live
lock file. Existing hostile same-user processes/OS compromise are outside this
boundary; rollback of authentic older backups is possible and explicitly allowed.

## Trusted local presentation

Only explicit `--mail-store DIRECTORY init|open|recover` enters this mode before
loading any SSH profile or spawning a transport. No remote OSC/ANSI frame starts
it, supplies a path/passphrase, changes a pin or initiates a send. Never allow
`@NATIVE1`, SSH_USER_AUTH, an environment variable or typed home address to become
mail/home authority. Secrets use non-echo raw input with discarded typeahead,
bounded zeroizing buffers, no argv/env route, and core dumps disabled. UI/keys
leave scope on lock/cancel/inactivity/error; do not promise complete memory or
terminal-history erasure. The ordinary Files/SSH branch is unchanged.

## Required next adapter increment

Keep XMPP TLS/account credentials, device keys/ratchets, authenticated decrypted
messages, home proof issuance and local BBS approval separate. Compare native
libraries before changing this container. It currently holds a local storage key,
drafts and pins, **no OMEMO private identity/prekeys/ratchets/incoming messages**.
The first native adapter must pin `urn:xmpp:omemo:2` and `urn:xmpp:sce:1`, pass
independent-client cross-decryption, enforce verified devices and authenticate
inner sender/recipient before displaying subject/body. No downgrade or automatic
server-added device. Define bounded XML/frame/archives/reconnect/dedup/outbox
processing and atomic state+message commits before publishing/sending ratchets.
Avoid a trait that allows a remote plugin to inject claimed verified plaintext.

Back up usable encrypted history and necessary protocol state explicitly; a
new device/home password alone cannot recover ratcheted history. Define migration,
revocation/old-backup limits and recovery enrollment before making support claims.
Current generic draft/pin persistence must not masquerade as those controls.

The [two-way NetBSD qualification](research/0054-omemo-interop-netbsd.md) rejects
the original library and qualifies only an explicit corrected research candidate.
Its typed `ed25519:` pins and raw restart snapshots are not v1 vault records.
Existing Curve-format pins require deliberate verified enrollment, never silent
conversion. [#203](https://github.com/gonz0w/faction-bbs/issues/203) owns source
adoption review and encrypted atomic state/message/outbox persistence before a
transport can send. The current product vault remains version 1 and unchanged.

For #91, a locally typed home address is only a label. Online proof must establish
issuer, full subject, member device possession, target board audience, lifetime
and replay policy, then enter a trusted live connection-bound server path. Keep
that path separate from mail keys and user-writable files. The visited BBS maps
the proven subject to its **own** current approval, roles and epoch and denies
local revocation. Home outage cannot disable local rooms/files/games. Native
OpenSSH certificate signing remains blocked by #92; external signer controls
are not a shipped issuer. No central directory, password forwarding or role import.

For #88, attachment blobs are encrypted on the member device before any upload.
Private names/types/integrity/key material belong inside the authenticated E2EE
message. Keep ciphertext retrieval/quotas/save checks distinct from public file
references and do not unlock keys in the file service or a visited BBS.

# Native encrypted checkpoint and outbox contract

Version 2, [#203](https://github.com/gonz0w/faction-bbs/issues/203),
[ADR 0078](decisions/0078-encrypted-native-checkpoints.md). Default-off experimental
member-device library, with [NetBSD/host evidence](evidence/mail-checkpoints/README.md).
The existing [v1 vault UI](../docs/local-mail-vault.md) and its command line stay
unchanged. This backend does not connect to XMPP, publish prekeys/devices, unlock
through SSH, synchronize mailboxes or provide the terminal inbox UI.

## Build and authority

```sh
cargo +1.96.0 test -p faction-client -p faction-omemo \
  --features faction-client/native-mail --all-targets --release --locked -- --test-threads=1
python3 tools/check_omemo_dependencies.py
```

On NetBSD select base OpenSSL with `OPENSSL_INCLUDE_DIR=/usr/include` and
`OPENSSL_LIB_DIR=/usr/lib`. No pkg-config is needed with that explicit pair.
The source/build/update contract and full third-party notices are in the
[native crate](../crates/faction-omemo/README.md).
`faction_client::mail_checkpoint` exists only with the optional feature. The local
caller supplies hidden unlock bytes, trusted current time, explicit verified
identity choices and delivery decisions. Never populate these from remote ANSI,
OSC, environment variables, plugin claims or untrusted transport metadata.

## Envelope and payload

`mail.fkv` remains in a same-UID 0700 directory with a lifetime 0600 exclusive
`mail.lock`; all path components use descriptor-relative NOFOLLOW. The v2 envelope
keeps v1's offsets, fixed KDF parameters and nonce/key sizes, but uses:

| Item | Version 2 value |
| --- | --- |
| Outer magic, bytes 0..8 | `FACTVLT2` |
| Wrap AAD | `Faction local mail vault v2`, NUL, purpose (`password`/`recovery`), header bytes 0..40 |
| Payload AAD | `Faction checkpoint payload v2`, NUL, bytes 0..208 |
| Decoded payload magic | `FACTCP02`; explicit archive/read upgrades use `FACTCP03` / `FACTCP04`, and explicit attachment opt-in uses `FACTCP05`. |
| Decrypted/file caps | 524288 / 524512 bytes |

See [v1 offsets](native-mail-vault.md). No file-controlled KDF algorithm/cost
negotiation exists. Every payload save gets a fresh random XChaCha nonce; an
explicit upgrade/reset rotates data key, salt, both wraps and recovery secret.
Current and backup files cannot revoke secrets for older copies.

Payload integers are unsigned big-endian; strings/blobs have a u32 byte length;
booleans must be 0/1 and exact EOF is required. The core order is:

1. Magic, u64 revision, length-prefixed unchanged v1 `Data` encoding (drafts/pins).
2. Local device u32, full Ed identity32, native snapshot blob.
3. Peer-present boolean; if present, address, device u32, full Ed identity32,
   revoked boolean. Started boolean, creation/expiry u64 seconds, history-only flag.
4. History count and records: direction (0 sent/1 received), peer address/device/
   identity, sent time, subject/body, operation16, encrypted-frame digest32,
   intent digest32. Receive records use zero operation/intent; send IDs are nonzero.
5. Outbox count and entries: operation16 then full canonical `Wire`.

In FACTCP05, each history record appends a descriptor-present boolean after the
intent digest. If present, the descriptor is name and media type (each u32-length
UTF-8 bytes), size u64, hash32, key32, IV12, and locator (u32-length UTF-8
bytes), in that order. Following the unchanged outbox are an archive-present
boolean and optional existing archive fields, then a read-present boolean and
optional existing read count/booleans. The magic itself is the attachment
capability marker; there is no extra capability flag. FACTCP02/03/04 retain their
existing encodings. Descriptor validation and the existing counts and bounds
apply on decode before native state is usable.

`Wire` fields are namespace/from/to strings, sender/recipient device u32, sender
Ed identity32, sent time u64, key blob, prekey boolean and payload blob. The frame
hash is SHA-256 of `Faction encrypted frame v2`, NUL, then those exact fields.
The outbound intent hash uses `Faction outbound intent v2`, NUL, recipient address/
device/Ed identity, original sending time and length-prefixed subject/body.
For attachment mail, a distinct deterministic intent uses `Faction outbound
attachment intent v1`, NUL, then recipient address/device/full identity, sending
time, subject/body and the canonical descriptor fields above. The ordinary text
intent remains unchanged. Digests and private metadata live inside authenticated ciphertext. The input
namespace is exactly `urn:xmpp:omemo:2`; no fallback profile is accepted.

Bounds: one local identity and one deliberately verified peer per vault; legacy
32 drafts/64 pins with their original caps; native snapshot32768, retained skipped
keys32, isolated verification derivations64; history64 including sent records;
pending8; subject128/body1024 UTF-8 bytes; SCE8192, key256, payload8208 bytes.
Native snapshots include store/prekeys/session/skipped keys but are never exported
unencrypted by the checkpoint API. Decode also checks state/peer/pending/history
consistency and imports the pinned native format into a temporary context.
No silent eviction or unlimited backlog. A full store blocks new messages; future
history UI/export policy remains part of #90.

## Transactions and retry

`create` initializes v2; `upgrade` consumes an unlocked v1 vault, preserves drafts/
legacy pins and returns a new recovery secret only after durable publication.
V1 tools reject v2 magic. `bundle` exposes public material only for the trusted
local caller; it is not a server publication mechanism. `enroll` accepts an exact
`ed25519:` pin after external verification, never a legacy Curve label or a server
list. `start` is explicit and cannot reset an already-started session.

`queue` takes a nonzero operation ID, subject/body and original sending time. It
clones complete state, encodes SCE, advances native crypto on the candidate and
adds history plus immutable ciphertext in one encrypted checkpoint. It returns
ciphertext only after file+directory sync. A repeated ID with the same intent
returns identical pending bytes; changed content/time rejects. An acknowledged ID
never seals again. `pending_deliveries` returns persisted `{operation, wire}` so
restart callers can deliver/ack without a second caller-side database. `pending`
is a ciphertext-only convenience view. `acknowledge` retires pending ciphertext;
it records the trusted caller's decision, not remote exactly-once/read semantics.

`receive` accepts encrypted frames only. Current exact JID/device/Ed binding and
revocation are checked before duplicates. It stages native decryption, then
validates inner SCE before committing snapshot, history and replay receipt together.
Exact duplicates return the existing receipt, without another history row/key
consumption; modified frames must authenticate. Replay/validation/full/expired
rejections leave canonical bytes unchanged. No plugin supplies claimed verified
plaintext. Low-level native handles remain within trusted local implementation.

SCE uses the reviewed XML parser with DTD/entity resolution disabled, no declarations,
comments/PIs/CDATA/unknown shape, 128-node/8192-byte bounds and escaped subject/body.
The initial timestamp subset is whole-second UTC, years1970..9999, within300 seconds
of the enclosing sending time, not arrival-now. Offsets/fractions/leap seconds and
other extensions require separate qualification. Fixed256-character random-content
padding has limited length hiding; full XEP/XMPP conformance is not claimed.

## Failure, time and recovery

Each mutation builds a candidate first, writes ciphertext to an exclusive private
temp, syncs it, atomically publishes and syncs the directory. New directory names
sync their parent; initial/backup publication refuses existing targets. No accepted
plaintext or sendable frame escapes on write failure. Any such I/O error poisons
the handle, including reset. Drop it; do not undo memory state or retry on that
handle. After reset ambiguity, the new password may already be canonical.

Reopen locks the directory, reads only canonical `mail.fkv`, authenticates it and
syncs the file and directory before importing private native state. Temp files
are never authority, backup fallback is never automatic and live locks are never
unlinked. Crash tests cover process death around rename and simulated delivery;
this is not power-cut, damaged-filesystem or physical-storage certification.

Operations use a trusted local clock and reject before creation or after the
fixed30-day session expiry. No automatic key/session restart. History stays readable;
revocation also blocks pending retry and duplicates. `discard_pending` is explicit
retirement. `fresh_device` requires pending retirement and a different device ID,
generates a new native identity and requires fresh explicit peer verification.
Old history/operation IDs survive. This bounds the fixed peer/session's retained
prekeys without implementing live multi-device publishing/rotation.

Official `backup` writes a fresh encrypted, authenticated history-only copy.
Recovery-secret unlock also durably quarantines the canonical checkpoint before
returning it; password reopen cannot clear quarantine. History remains readable,
but sending/receiving requires deliberate fresh device enrollment. A normal password
reset on a healthy current handle preserves its protocol state; resetting cannot
unquarantine recovered state. Old backups, manual copied active files, recipients'
copies or compromised OS state cannot be revoked by a new password. There is no
anti-rollback guarantee for arbitrary active-file copies.

Core dumps are disabled before native secret allocation/unlock. Owned buffers use
zeroization; parser normalization, compiler copies, swap, OS/terminal compromise
and complete memory erasure are not guaranteed. Existing server limits, local BBS
approval, SSH/files and plaintext community storage remain separate. No server
reads member mail/keys, and no home password reset recovers them.

Home addresses remain locally claimed labels until the separate home-authentication
work proves account/device authority. This adapter supplies no BBS role, approval
or roaming identity. [#205](https://github.com/gonz0w/faction-bbs/issues/205) tracks
the [trusted local checkpoint console](../docs/checkpoint-console.md); live transport
integration remains separate.

Linux C11 builds explicitly enable `_DEFAULT_SOURCE` for the system endian macros;
NetBSD retains `_NETBSD_SOURCE` and Darwin adds neither definition. The saved VM
record measured the earlier build-script bytes, retained through the
[post-proof build follow-up](evidence/mail-checkpoints/linux-build-followup.json).
Runtime/vendor sources and the BSD/Darwin compiler definitions did not change.

## Manual home-adapter boundary

The optional [home adapter](native-home-mail.md) calls `publication(now)` for
public bundle material and 32 existing public prekeys from one durable snapshot.
Expiry, revocation, quarantine or a poisoned handle prevent publication. Receiving
a prekey message may change which keys remain publishable; the caller refreshes
this view after processing messages during an IQ wait.

`receive_network` requires a `Wire` with `sent_at = 0`, because standard OMEMO
carries the sender time inside authenticated SCE. It derives the time only after
native authentication and strict SCE decoding, then commits the same canonical
Wire digest/history representation. Exact duplicate lookup tries only the bounded
received-history timestamps. It never substitutes arrival or server delay time.
The original `receive` API and its enclosing-time comparison remain unchanged.
Neither API changes the v2 envelope/payload schema or grants network trust.
The caller samples its trusted cycle clock immediately before each operation.

## Explicit archive payload schema

The optional [archive contract](native-mail-archive.md) adds an explicit
FACTCP02-to-FACTCP03 payload migration inside the unchanged FACTVLT2 encryption
envelope. Schema 2 remains exact and has no cursor. Schema 3 atomically carries
home/device/epoch-scoped cursor progress with protocol/history/outbox state;
ordinary unlock never silently migrates. Recovery and official backups remain
history-only, and local secret reset cannot restore live ratchets.

The [encrypted local read-state contract](local-mail-read-state.md) defines the
explicit FACTCP04 transition, received-frame/index-bound local acknowledgements,
strict metadata validation and preservation through backup/recovery/fresh devices.
No read receipt or BBS authority is added.

## Attachment opt-in

`enable_attachments` deliberately migrates to FACTCP05; ordinary unlock, queue and receive do not upgrade earlier formats. `attachments_enabled` reports the choice. Earlier formats retain their existing attachment rejection. The encrypted history retains an optional descriptor per record and independently present archive and read state. No descriptor or key sidecar is written. Existing checkpoint, history, outbox and native limits remain in force.

The [attachment mail contract](attachment-mail.md) describes descriptor-bound queue/retry, authenticated receipt and local verification of explicitly supplied ciphertext. The checkpoint does not persist or transfer bulk bytes. Recovery remains history-only and does not restore live ratchets.

# ADR 0078: Experimental encrypted native mail checkpoints

Status: accepted for the default-off member-device backend, 2026-10-25.
Authority: [#203](https://github.com/gonz0w/faction-bbs/issues/203), following
[qualification #200](https://github.com/gonz0w/faction-bbs/issues/200) and
[correction #202](https://github.com/gonz0w/faction-bbs/issues/202).
This adopts specific experimental bytes, not a supported release, audit or live
XMPP service. [ADR 0035](0035-native-local-mail-vault.md) remains the v1 UI contract.

## Choice

Use the existing Rust member client with an optional `native-mail` feature and a
separately gated `faction-omemo/native` library. Vendor only six exact corrected
Picomemo 1.3.1 generated-C/HACL/OpenSSL include files. Preserve their full notices,
licenses and independent source pins. The [vendor contract](../../crates/faction-omemo/VENDOR.md)
records the original revision/archive, explicit `ad-order-v1` patch, selected
hashes and update rules. Build only these bytes, natively, with system OpenSSL 3.
No build-time download, generation, arbitrary source override or legacy-profile
fallback. A disabled native crate discovers no C/hash/pkg-config tools and links
no libcrypto. Ordinary server/SSH/files/v1 vault operation needs no adapter.

External-source builds would save a small source copy but require every builder
to acquire, patch and verify the right variant. Vendoring the reviewed subset
makes the optional backend reproducible offline. Rust `roxmltree` 0.21.1 with
only `std` parses bounded SCE; existing library primitives handle cryptography.
No Python reference, AGPL component, new ratchet algorithm or JS/TS is adopted.
The [native graph inventory](../dependencies/native-mail.json) covers selected
Linux/NetBSD packages, licenses and features separately from the default graph.

## Durable state

Use one member-owned version-2 encrypted file for protocol state, legacy drafts/
pins, accepted history/dedup and immutable outbound ciphertext. Keep v1's fixed
Argon2id and XChaCha20-Poly1305 parameters, with separate magic and AAD. Upgrade
is explicit and rotates both local unlocking paths; old v1 backups still open
with their original secrets. Legacy Curve pins never become verified Ed devices.

Run native crypto/SCE validation on isolated candidates. Publish only ciphertext,
with private temporary creation, file sync, atomic replacement and directory sync.
Return accepted history or sendable ciphertext only after success. Poison a handle
after any checkpoint-write I/O failure; authenticate and sync canonical file plus
directory before importing state on reopen. Never restore old in-memory keys after
an ambiguous rename/sync error. New vault directories also sync their parent.

The first backend bounds one local device/verified peer, 64 history records,
eight pending messages, 512 KiB decrypted state and a 30-day session window.
Reject overflow/expiry, retain history and require explicit action. Official
backups and recovery unlocks are durably history-only until fresh device enrollment.
Arbitrary copied-file rollback cannot be detected by counters in that same file.

## Consequences and reversal

This is a library/storage boundary; terminal inbox/setup UI and live XMPP remain
outside this increment. [The contract](../native-mail-checkpoints.md) defines
bytes, APIs, recovery and limits. Optional-feature builds and source/graph checks
are dependencies of required CI. Updates require a finite issue, reviewed new
provenance/licenses, preserved old evidence, host/NetBSD and independent reciprocal
interop controls. System OpenSSL updates remain platform maintenance, not a claim
of identical qualified bytes or full-library professional audit.

Disable the feature to retain ordinary operation. V1 tools deliberately reject
v2 files. V2 backup/history remains readable through its typed API; loss of both
unlocking paths loses local history. Changing the native session format requires
an explicit migration or fresh enrollment, never silent reinterpretation.

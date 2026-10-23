# 0035: Member-owned local encrypted mail state

Status: accepted bounded implementation, 2026-10-21; no OMEMO/SSO adoption.
Issue: [#90](https://github.com/gonz0w/faction-bbs/issues/90), parent #84.

Build local storage/unlock/presentation in the Rust native client while native
protocol-library selection remains open. A separate explicitly local command
runs without an SSH child or server control frames. Existing Files control cannot
unlock/compose mail. Offline drafts and manually verified device pins are useful
persistence controls; do not advertise inbox delivery, device enrollment, proven
home identity or private roaming based on them.

Reuse pinned RustCrypto Argon2 0.6.0 and chacha20poly1305 0.11.0 with alloc/zeroize
only. Both offer MIT/Apache-2.0 choices, as do the seven newly selected packages.
The complete selected Linux/NetBSD package trees pass filename policy; see
[dependency inventory](../dependencies/local-core.json). Do not implement a cipher
or ratchet. The upstream historical audit is not an audit of our local format,
current dependency versions or integration. Constant-time multiplication is a
platform precondition; this does not enable every old NetBSD CPU automatically.

Use a random 256-bit local data key. Independently wrap it using a key derived by
fixed Argon2id v1.3 (65536 KiB, t=3, p=1, 32-byte output) and a random 256-bit member
offline recovery secret. No server password hash, server key or escrow. Fresh
24-byte nonces for every XChaCha20-Poly1305 operation. Authenticate the exact
version/id/salt and wrapper purpose; authenticate the complete prefix for the
payload. Wrong secret/tampering never decodes or renders plaintext. Fixed work
parameters and strict lengths prevent untrusted files requesting arbitrary KDF
costs. Recovery generates a new data key and both fresh wrappers, rather than
letting an old wrapping key decrypt future current-file payloads.

The storage is bounded (256 KiB plaintext, 32 drafts, 64 pins), owner-only and
symlink/hardlink checked with descriptor-relative operations. Retain one exclusive
lock inode; sync private ciphertext and publish atomically, with create-only init
and backups. No automatic plaintext export, synchronization, remote keys or new
service/helper. Errors relock; five-minute input inactivity locks. Best-effort
zeroization/core-dump denial do not imply swap/terminal/host compromise protection.
Old encrypted backups remain recoverable with their old secrets and can roll back
local pin removals; no anti-rollback guarantee or remote revocation is claimed.

Alternatives: server escrow/reset violates owner requirements; plaintext local
state or home-password-only recovery exposes keys/history; research Python helper
violates the native-product boundary. Libsodium remains a sound comparison point
but would add a mandatory native library/FFI boundary for this increment. RustCrypto
reuses existing native KDF/source gates and adds no resident process. A reviewed
native OMEMO implementation is still required; Picomemo (ISC, no professional
audit per upstream) is a concrete candidate for a separate C boundary/interop
probe, not adopted crypto. Libomemo-c declares GPL-3.0; do not silently change the
BSD distribution's licensing obligations. See [research 0046](../research/0046-native-local-mail-vault.md)
for exact screened trees, sources and remaining gates.

The format is a preview local container, version 1, not portable OMEMO history.
Adoption/reversal cost grows once actual ratchet/device state is persisted; define
migration and atomic message/state commits before that step. No mailbox data,
rooms, files, account approval or local roles are replicated. Roaming #91 needs
an independently verified home proof bound to a live connection and approved
local membership. Attachments #88 must encrypt blobs/metadata on the device and
transport keys only within authenticated encrypted mail.

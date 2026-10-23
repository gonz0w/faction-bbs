# 0046: Native local encrypted mail state and trusted terminal UI

Issue: [#90](https://github.com/gonz0w/faction-bbs/issues/90), parent #84.
Date: 2026-10-21. Scope: maintained Rust implementation and software-only native
NetBSD validation. [ADR 0035](../decisions/0035-native-local-mail-vault.md) selects
the bounded persistence increment; [format/boundary](../native-mail-vault.md).

## Result and observation limits

The native client now creates/unlocks a member-owned encrypted local vault,
composes/reads/deletes local drafts, explicitly verifies/removes device pins and
backs up/recovers authenticated ciphertext. The trusted screen runs from an
explicit local command, without an SSH profile/child, remote unlock RPC or new
server listener. The home address is a claim, not proof. The inbox/send controls
are explicitly unavailable. No OMEMO private device/prekey/ratchet state, network
send/reply/history, independent OMEMO interoperability or roaming auth is shipped.

The final source revision passes **105 workspace Rust tests** on macOS ARM64 and
native NetBSD 11 ARM64. **13 real native-client PTY checks** pass on each, including
hidden secrets, locally unsent drafts, verified pin confirmation, concurrent-lock
refusal, private encrypted backups without known plaintext markers, wrong-secret
denial, arrow-selected draft view, recovery/current-secret rotation, old-backup
access, persisted delete/pin removal, tamper denial, cancellation/echo restoration
and oversized-secret refusal. Rust tests add decoder/data/count bounds, changed-key
and out-of-range-device refusal, truncation/trailing bytes, authenticated format
components, no-clobber init, private/no-link files and immediate expired-idle refusal.
Known-marker scanning is one observation, not a general encryption proof.

NetBSD uses the signed official 11.0 ARM64 image/kernel already pinned by research
023, in a fresh QEMU/HVF VM with 2 vCPUs, 1024 MiB and loopback-only administrative
SSH forwards. Builds, tests and local mail PTY execution run as a separate
unprivileged `factionbuild` account. Native Rust is 1.96.0 and Python is 3.13.14;
host Rust is 1.96.0, with Python 3.14 for PTY/repository tools. C frontend/daemon
builds also pass; this increment changes no BBS owner source, defaults, helpers,
thread/credential limits, schema or SSH/SFTP services. No new live SSH/SFTP
end-to-end run, two-home network-mail rerun, load/capacity or hardware result is
claimed. The unchanged old #89/#93 results are preserved separately.

[Machine-readable evidence](../evidence/local-mail-vault-netbsd/results.json)
pins actual executed sources and the host/guest probe outputs. Exact guest build
input hashes are compared with the host files before recording. Disposable stores,
credentials and VM inputs are removed after validation. The checker verifies
record identity/constraints/source hashes; it does not rerun the VM or audit crypto.

## Dependency and design comparison

| Candidate | Finding | Decision for this increment |
| --- | --- | --- |
| RustCrypto Argon2 + XChaCha20-Poly1305 | Fixed KDF, random keys/nonces, authenticated header/payload; existing Rust build and source-policy gates | Select local storage only; no network protocol/ratchet implied |
| Libsodium C | Reusable password hashing/AEAD APIs, additional required library/FFI and native packaging boundary | Keep comparison; no need to introduce it for this slice |
| Server escrow/plain local state | Cannot meet member-only decrypt/reset boundary | Reject |
| Research Python Twomemo helper | Actual paired reference proof, but outside native product boundary | Retain as independent research control only |
| Prose Rust client | MPL-2.0 client manifest depends on git Signal protocol v0.41.2 and broader native stack | Unselected; complete license/source/NetBSD graph and actual OMEMO-2 profile not cleared |
| Picomemo C | ISC; explicit OMEMO-2 build path, serializable store/session API; own source tree has no forbidden names | Promising bounded native FFI/interop probe; not adopted and no professional audit per upstream |
| Dino libomemo-c | Explicit newer OMEMO protocol support; own source tree no forbidden names; GPL-3.0 declaration | Licensing/distribution decision unresolved; no implicit BSD license change |

Pinned new local dependency is chacha20poly1305 0.11.0 with alloc/zeroize only.
Seven new registry packages (aead, chacha20, chacha20poly1305, cipher, inout,
poly1305, universal-hash) offer MIT/Apache-2.0; all source filenames in the complete
selected Linux/NetBSD graphs pass the no-JS policy. The graphs now contain 46/48
registry packages respectively. Locked package checksums and reviewed features
are in Cargo.lock and [inventory](../dependencies/local-core.json). Argon2 0.6.0
and zeroize 1.9.1 reuse existing dependencies. This is not a vulnerability audit,
current-version independent cryptographic audit or every-platform graph review.
RustCrypto documents constant-time-multiplication requirements; no universal old
NetBSD CPU claim. 64-MiB unlock work is on the member device, not per connected
member on the BBS host.

Candidate screening used complete GitHub recursive trees (not truncated):
Picomemo tree `835d8277108b96c691fb6a992512f37315cb52d7`, 50 blobs, ISC;
libomemo-c tree `1f0b37fee3ebefc46c32f882623481899a1ba568`, 204 blobs, GPL-3.0.
No forbidden JS/TS filenames were present in either tree. These are **tree object
IDs**, not release signatures, lockfiles, transitive dependency/build-tool reviews,
audit evidence or native runtime/interoperability results. Do not treat permissive
license/small source count as sufficient evidence to adopt unaudited cryptography.

## Contradictions and corrections

The first local filesystem tests rejected macOS's symlinked `/var` temp prefix.
Retain full path-component NOFOLLOW checks and use the fixture's canonical
physical path; do not weaken symlink checks. The first busy-vault PTY assertion
incorrectly expected a password prompt after lock failure; the correct behavior
is to deny before prompting. Test compilation also required the actual shared
Key::Hotkey decoder variant and explicit OwnedFd-to-File conversion. All final
runtime/PTY observations use the corrected sources. The final native run also
includes the 1..2^31-1 OMEMO device-ID restriction from XEP-0384, not arbitrary u32.
These were development failures, not silently discarded contrary capacity trials.

## Remaining work and confidence

High confidence in these tested local behaviors, not a production crypto audit.
Keep #90/#84 open for a reviewed native OMEMO-2/SCE adapter, actual two-home exchange
with an independent implementation, bounded reconnect/archive/dedup and atomic
ratchet+message durability. Prove real device enrollment/removal/history recovery
without home-password resets enrolling an attacker. Old backups retain old
secrets and pins; no rollback or remote-revocation guarantee is claimed.

The [adapter boundary](../native-mail-vault.md) advances #91 by distinguishing
local address/pin metadata, authenticated device possession, scoped online home
proofs and trusted connection-bound local approval. The local address cannot bypass
BBS approval. Native certificate issuance remains blocked by #92. Ordinary local
BBS features must continue during home outages; simultaneous native inbox/BBS
operation remains an integration gate, not a claim from this offline preview.
Keep encrypted attachment metadata/keys inside E2EE envelopes and blobs encrypted
before upload under #88; do not reuse sysop-readable file areas as private mail.

## Primary sources retrieved 2026-10-21

- [RustCrypto chacha20poly1305 0.11.0](https://docs.rs/chacha20poly1305/0.11.0/chacha20poly1305/)
  and [Argon2 0.6.0](https://docs.rs/argon2/0.6.0/argon2/): actual APIs, licenses,
  historical audit/processor caveats and password-derived key usage.
- [Libsodium password hashing](https://doc.libsodium.org/password_hashing/default_phf):
  established native comparison, not adopted runtime.
- [XEP-0384 0.9.1](https://xmpp.org/extensions/xep-0384.html): Experimental OMEMO-2
  device-ID bounds and full Curve25519-format public identity-key verification.
- [Picomemo upstream README](https://github.com/mierenhoop/picomemo) and
  [security policy](https://github.com/mierenhoop/picomemo/security/policy): claimed
  version paths/API/ISC and explicit absence of a professional security audit.
- [Dino libomemo-c](https://github.com/dino/libomemo-c): actual newer-protocol
  reuse path and declared GPL-3.0 obligations, not inferred permissive licensing.
- [Prose native manifest](https://raw.githubusercontent.com/prose-im/prose-core-client/master/crates/prose-core-client/Cargo.toml):
  client MPL-2.0 and native git Signal protocol dependency.

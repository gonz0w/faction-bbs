# 0059: Historical attachment byte proof

[Issue #250](https://github.com/gonz0w/faction-bbs/issues/250) tests a narrow question from the [attachment profile comparison](0058-encrypted-attachment-profile.md): can a native adapter and an unchanged reference exchange the same historical AES-256-GCM ciphertext and tag for complete, bounded inputs? On the recorded macOS host, the selected cases passed. This is a laboratory observation, not an attachment feature or a choice of wire format.

The experiment uses empty associated data and ciphertext followed by a 16-byte tag. It does not implement the proposed XEP-0447/0448 descriptor, SCE envelope, transport, checkpoint or storage contract. In-memory quarantine delays accepted plaintext until authentication succeeds; it is not a durable save or recovery design. The broader attachment issue [#88](https://github.com/gonz0w/faction-bbs/issues/88) remains open.

## Qualified inputs

The native side uses OpenSSL 3.5.7 at source commit `8cf17aaeb4599f8af87fefd810b5b5fee90fe69e`, built as a static library in an isolated owned directory with no host installation. The upstream CLI identified the default provider and AES-256-GCM. The reference is the unchanged Gajim 2.4.7 `aes.py` at commit `c9b40e5e220ea59b0e387ac3e753eac1cfe215a1`, SHA-256 `79036e4d5f72cf8696b7ee44a5223ff6f5214a18b4e8809bf9b22167128c10e4`. Its GPL-3.0-only source and license remain outside Git.

The isolated Python 3.14.7 environment had no pip and contained cryptography 50.0.2, CFFI 2.0.0 and pycparser 2.23. The selected cryptography wheel SHA-256 is `fa8f5efb344d6908a1ce62f4a24e2e5780f825d6f53f5f50ec5ffacac72936cb`. Its 39 Rust SBOM components matched the source lock versions, and it reported bundled OpenSSL 4.0.3. The two sides therefore share OpenSSL lineage; their comparison is not independent primitive validation. Published wheel hashes and metadata do not establish the wheel compiler or independent binary reproducibility. System dyld shared-cache bytes were not independently hashed. CFFI wheel metadata says MIT while its retained license text identifies MIT No Attribution; both identities were preserved.

The initial source-archive check rejected an assumption that every Git tree file appeared in the archive. This setup failure was preserved; it was not a behavior RED. The corrected qualification accounted for 57 export-ignore omissions and matched all 5,750 included blobs to the pinned Git tree. The qualification's 63 frozen file hashes remained unchanged after the proof. The actual host was macOS 27.2 ARM64 with Apple Clang 21; no NetBSD or VM result follows from this run.

## Test history and observations

The stdlib test source was run before `proof.py` existed and failed on the missing sibling module using `-s -B`. An earlier `-I` invocation excluded the script path and is a separate setup failure. The direct Gajim API baseline produced 16 provisional bytes before a boundary assertion failed; that baseline caught a general exception and did not establish an exact authentication class. Before the C producer existed, the explicit native suite separately failed on its missing binary, covering 20 tests and subcases. The first implementation passed 16 stdlib tests and the native-enabled suite of 20 total tests: the same 16 stdlib tests plus four native contract groups. No post-green fix is claimed.

The first integration run exited zero. Both implementations matched two NIST CAVS 14 known-answer cases and exchanged eight fixed complete inputs in both directions. Plaintext lengths were 0, 1, 15, 16, 17, 4095, 4096 and 4097 bytes. The laboratory caps were 8192 plaintext bytes and 8208 ciphertext bytes, not product limits.

| Observation | Recorded result |
| --- | --- |
| Complete inputs | Eight matching ciphertexts and reciprocal decryptions |
| Negative groups | 47 rejected by both sides; no accepted plaintext or spool mutation |
| Authentication groups | 43 raised `cryptography.exceptions.InvalidTag` on the reference and native exit 3 |
| Framing groups | Four short-tag inputs rejected as missing authentication framing |
| Chunk observations | 40; source predictions matched all observations |
| Reference chunk decryption | 14 successes and 26 retained `ValueError` failures |
| Local spool | Eight regular ciphertext objects, 12,465 bytes total, with recorded hashes |

Chunk sizes were 1, 15, 16, 17 and 4096 bytes. Native results matched for these fixtures, but that does not establish general streaming support. The reference failures reflect its retained-tag-cache segmentation and are not patched away. All keys, IVs and data were fixed public laboratory values; the spool inspection is not a privacy test of real secrets or members.

The frozen C source, Python harness and tests had SHA-256 values beginning `30c4e886`, `97a90feb` and `9873d5e3`, respectively; the native binary began `a19a7047`. A separate reviewer ran the same 20-test native-enabled suite and audited the actual spool; this was not a second integration run. The owned fixture was retained for lead review at the time of these observations, so cleanup must be established separately.

## Reading and using the result

The [selected saved evidence](../evidence/attachment-bytes/README.md) records the observed inputs and results. The [experiment instructions](../../experiments/250-attachment-bytes/README.md) explain the supplied test interfaces and prerequisites. The saved evidence checker and stdlib harness check recorded evidence and unit contracts; they do not download dependencies or rerun real cryptography. A fresh comparison needs the exact qualified external inputs and an owned disposable fixture. Missing inputs are a reason to stop and record the gap, not to substitute an unqualified dependency.

This result does not establish an authenticated descriptor, verified peer binding, private remote storage, download authorization, durable keys, safe destination selection, crash recovery, or interoperability with a full current XMPP attachment profile. Nor does it establish production readiness, endpoint behavior, NetBSD support, or an independent cryptographic primitive. Those remain separate questions under the existing project and privacy contracts. No listener, provider, HTTP service or host configuration was changed for this proof.

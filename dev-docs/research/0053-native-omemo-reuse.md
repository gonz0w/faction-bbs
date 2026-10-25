# Native OMEMO-2 reuse qualification

Date: 2026-10-24. [Issue #199](https://github.com/gonz0w/faction-bbs/issues/199),
bounded prerequisite of [#90](https://github.com/gonz0w/faction-bbs/issues/90).

## Decision and scope

Continue qualification of **Picomemo 1.3.1**, not product adoption. Its permissive
C implementation can supply the cryptographic layer beneath a Rust adapter. It
does not supply Faction's trusted-device policy, XMPP/SCE envelope, encrypted
transactional store or inbox. Keep the existing local vault and ordinary board
operation unchanged. No new product dependency or installed service is selected.

The falsifiable next gate is [#200](https://github.com/gonz0w/faction-bbs/issues/200):
actual two-way independent interoperability, identity representation vectors and
native NetBSD execution. Failure of those controls rejects this candidate for
adoption. Host compilation and upstream tests alone cannot satisfy #90.

## Pinned comparison

| Candidate | Observed source contract | Result for this increment |
| --- | --- | --- |
| Picomemo 1.3.1, `835d8277108b96c691fb6a992512f37315cb52d7` | ISC core, included HACL notices for MIT/Apache-2.0, OpenSSL or MbedTLS backend. `-DOMEMO2` selects `urn:xmpp:omemo:2`; generated `omemo2` API avoids mixing legacy and current structure definitions. | Leading native candidate; no complete client or independent audit established. |
| Prose, `1ef3752575e6d9cf0684dfec3b44f916258e0e83` | MPL-2.0 wrapper; current native module uses legacy OMEMO. Signal dependency tag v0.41.2 resolves to `e465b77cb63f4f61bb9df919ada60e5a298b1828`, with AGPL-3.0-only protocol/core/crypto crates. Additional git dependencies and no committed root lockfile. | Does not deliver the requested OMEMO-2 profile; no acceptable complete resolved native source/license graph established. |
| libomemo-c, `1f0b37fee3ebefc46c32f882623481899a1ba568` | GPL-3.0 with a narrow Apple App Store permission, not general permissive relicensing. Version-4 ratchet/protobuf support, caller crypto/store/trust callbacks and protobuf-c generation. XML namespace/SCE selection is outside this library. | Useful protocol/API comparison, not the selected permissive native component or proof of complete interoperability. |

Picomemo advertises the OMEMO 0.9 family; its header points to XEP-0384 0.9.1.
That identifies the inspected profile, not conformance with every revision sharing
the namespace. SCE envelope construction/validation is outside this library.

These are source-based screening results. Prose/libomemo-c were not built, and
no legal compatibility opinion or complete resolved audit is claimed. Signal's
upstream tree includes JS/TS bindings; excluding them from one native build would
not by itself establish a reviewed graph acceptable to Faction. No such dependency
or browser code was adopted or executed.

Primary sources retrieved on the report date:

- Picomemo [core/API](https://github.com/mierenhoop/picomemo/blob/835d8277108b96c691fb6a992512f37315cb52d7/omemo.h),
  [implementation](https://github.com/mierenhoop/picomemo/blob/835d8277108b96c691fb6a992512f37315cb52d7/omemo.c),
  [license](https://github.com/mierenhoop/picomemo/blob/835d8277108b96c691fb6a992512f37315cb52d7/LICENSE),
  [HACL notices](https://github.com/mierenhoop/picomemo/blob/835d8277108b96c691fb6a992512f37315cb52d7/hacl.c),
  [OpenSSL backend](https://github.com/mierenhoop/picomemo/blob/835d8277108b96c691fb6a992512f37315cb52d7/openssl.c)
  and [security policy](https://github.com/mierenhoop/picomemo/blob/835d8277108b96c691fb6a992512f37315cb52d7/.github/SECURITY.md).
- Prose [manifest](https://github.com/prose-im/prose-core-client/blob/1ef3752575e6d9cf0684dfec3b44f916258e0e83/crates/prose-core-client/Cargo.toml),
  [legacy module](https://github.com/prose-im/prose-core-client/blob/1ef3752575e6d9cf0684dfec3b44f916258e0e83/crates/prose-xmpp/src/mods/omemo.rs),
  [license](https://github.com/prose-im/prose-core-client/blob/1ef3752575e6d9cf0684dfec3b44f916258e0e83/LICENSE.md)
  and exact Signal [protocol manifest](https://github.com/signalapp/libsignal/blob/e465b77cb63f4f61bb9df919ada60e5a298b1828/rust/protocol/Cargo.toml).
- libomemo-c [license](https://github.com/dino/libomemo-c/blob/1f0b37fee3ebefc46c32f882623481899a1ba568/LICENSE),
  [public API](https://github.com/dino/libomemo-c/blob/1f0b37fee3ebefc46c32f882623481899a1ba568/src/signal_protocol.h)
  and [protobuf build](https://github.com/dino/libomemo-c/blob/1f0b37fee3ebefc46c32f882623481899a1ba568/protobuf/meson.build).

## Actual host observations

[Saved results](../evidence/native-omemo-host/results.md) pin the archive,
sixteen inspected source files, comparison files, reference wheel hashes, compiler,
commands, outputs and artifact sizes. All upstream C/Python test sources remained
unchanged. No new executable harness, cryptography or product code was authored.
Research validation uses source review and actual existing upstream commands;
there is no invented behavioral TDD cycle for these unchanged tests.

Host: macOS ARM64/Darwin 27, Apple Clang 21, OpenSSL 3.6.3, Python 3.13 reference
fixture. Compiled only the generated OMEMO-2 library plus HACL/OpenSSL drivers;
no legacy library, Lua generation, Docker, XMPP server or browser target was needed.
The archives compile the generated prefixed API. Upstream tests instead include
monolithic `omemo.c` with `-DOMEMO2`; they do not call the linked generated archive
or a Rust FFI wrapper. Those interfaces need their own #200 controls.

Default build documentation requires newer GNU Make than the host's 3.81;
individual compiler commands avoid assuming the complete upstream build is portable.
A single-object dry-run is recorded, not a successful full Makefile build.

| Observed artifact/control | Result |
| --- | --- |
| OMEMO-2 + HACL/OpenSSL static archive | 154,936 bytes; upstream test exit 0. |
| OMEMO-2 + alternative c25519/OpenSSL archive | 88,416 bytes; upstream test exit 0. |
| HACL/OpenSSL AddressSanitizer + UndefinedBehaviorSanitizer test | Exit 0 with no reported diagnostic; not an exhaustive memory/security analysis. |
| Upstream Twomemo 2.1.0 fixture → Picomemo | First prekey payload decrypt passes; reply bytes are produced but not independently decrypted. |
| HACL test linked with `libcrypto` alone | Exit 0; `otool -L` shows libcrypto and system C, without libssl. |
| One HACL test process cost observation | Maximum RSS 5,259,264 bytes. Short host test only; not whole native client, SBC RAM or throughput. |

The unchanged upstream runner prints seventeen named groups. Some contain
legacy-only conditional sections; this is not seventeen independent OMEMO-2
conformance cases. It covers parsing, encryption/tamper checks, signatures,
serialization and delayed-session behavior, not complete XMPP/SCE or Faction trust.
See the exact [test source](https://github.com/mierenhoop/picomemo/blob/835d8277108b96c691fb6a992512f37315cb52d7/test/omemo.c)
and [fixture rules](https://github.com/mierenhoop/picomemo/blob/835d8277108b96c691fb6a992512f37315cb52d7/test/build.mk).

The reference fixture uses fifteen Python packages, all pinned in the record.
They are research dependencies, not a product runtime. Bootstrap pip contained an
unused Emscripten JS worker; it was removed with pip before the final reference
run. The final reference runtime had no JS/TS filenames, and no JavaScript was
executed. This is an explicit exclusion, not a claim that every upstream optional
build tool or example is JS-free. An initial nonexistent Oldmemo version request
failed package resolution; the record preserves that setup mistake and correction.

Archive sizes omit dynamic dependencies. The measured libcrypto file is 4,828,048
bytes; libssl is unnecessary for this crypto-only test. Those are file sizes,
not resident memory or a complete OpenSSL build/license supply-chain audit. The
existing member vault's unlock KDF cost and a future whole-client build must be
measured separately. Server session/credential/resource defaults did not change.

## Reproduce the bounded control

Use a new disposable directory outside the checkout, fetch the recorded archive,
and verify its archive/source hashes before executing it. The structured data appendix contains
all exact compiler/link commands and reference wheel URLs/hashes. Recreate only
that pinned Python reference environment; bootstrap pip is not part of the tested
runtime. Do not run upstream network/Docker targets or install a system library.

From the verified source directory, with `o` created and the isolated reference
Python available, the executed control has this shape (platform include/library
paths come from the selected OpenSSL pkg-config entry):

```sh
clang -O2 -DOMEMO2 -I. test/generate.c omemo.c hacl.c openssl.c \
  $(pkg-config --cflags --libs libcrypto) -o o/generate2
./o/generate2 o/store2.inc o/bundle2.py
PYTHONPATH=o /path/to/reference-python test/initsession.py bundle2
clang -O2 -DOMEMO2 -I. test/omemo.c hacl.c openssl.c \
  $(pkg-config --cflags --libs libcrypto) -o o/test-omemo2
./o/test-omemo2
```

The generator creates synthetic private keys; never use real member data here.
Keep generated stores, messages, Python packages and binaries outside Git and
remove the owned directory after testing. Recorded warnings and a failed initial
package-resolution request remain visible; they were not recast as library defects.

## Adapter contracts exposed by source review

1. **Typed verified identities.** OMEMO-2 wire public identities are Ed25519 bytes;
   Picomemo converts them for Curve25519 DH. The vault currently documents
   Curve25519-format pins. Equal length does not make those encodings equal.
   Establish independent vectors and an explicit enrollment format before mapping
   or migrating pins. Never silently reinterpret existing local records or accept
   a server-added device.
2. **Transactional state.** Session wrappers roll back their own structure on
   failure, but skipped-key callbacks can mutate external state before key-message
   authentication. Stage the session, skipped keys, consumed prekeys, authenticated
   message and retry/outbox changes together. Pin, payload or SCE rejection must
   discard the whole proposed transaction; session rollback alone is insufficient.
3. **Bounds and concurrency.** Registering a skipped-key callback replaces the
   default 1,000-key guard. Enforce aggregate/per-session caps and expiry. Global
   callbacks have no per-instance userdata; serialize ownership deliberately.
   Check every FFI buffer/length/error, including narrowing into OpenSSL int lengths.
4. **Persistence and plaintext handling.** Serializers require caller-sized buffers
   and are not encrypted/versioned vault records. Deserialize into zeroed temporary
   objects: store decoding can partially write before failure. Validate identity,
   counts and schema before committing; discard output on every error. Success
   authenticates bytes, not their JID/device policy or SCE sender/recipient binding.

These are integration obligations, not validated vulnerabilities in a deployed
Faction adapter. Picomemo's security policy states that it has no professional
audit; verified primitive provenance does not certify the full library or client.

## NetBSD and adoption boundary

The source explicitly recognizes NetBSD and calls `getentropy()` through
`unistd.h`. The selected OpenSSL driver requires OpenSSL 3 APIs. This makes a
native qualification plausible; it is **not a NetBSD execution result**. Actual
compiler/libcrypto/package versions, symbol/link behavior and FFI tests remain
#200's software-VM gate. HACL's architecture assumptions and the alternative
backend also need port-specific testing; no all-NetBSD-architecture promise follows.

No mailbox, credentials, device pins, Rust/Cargo dependencies, listeners or owner
playground state changed. Disposable build/reference stores and synthetic keys
are removed after saving sanitized observations. #90 still owns a reviewed native
adapter, atomic encrypted persistence, bounded send/offline/archive/retry/dedup,
recovery/history and trusted local UI. #91 home-proof binding and #88 attachments
remain separate. Hardware tests and deferred hardening are not prerequisites to
this bounded source/host result.

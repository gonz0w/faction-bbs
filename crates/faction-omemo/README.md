# Experimental member-device OMEMO adapter

This library reuses the qualified generated C library and opaque ownership shim
from [the interoperability proof](../../experiments/200-omemo-interop/README.md).
It remains experimental and has no transport, publication, inbox UI or raw-state
CLI. [Issue #203](https://github.com/gonz0w/faction-bbs/issues/203) records the
source-adoption choice. It does not establish a cryptographic audit or release.

The default feature set is empty. Enable `native` only on a member device that
needs local checkpoints. A disabled build returns before checking source hashes,
discovering a compiler/OpenSSL, or emitting native link flags; its Rust library
exports no native or SCE methods. Ordinary client SSH/files stay independent.

Native builds require HOST=TARGET, Rust 1.96, a C11 compiler, `ar`, system OpenSSL
major 3 headers/libraries, and `shasum` (`sha256 -q` on NetBSD). Discovery uses
`pkg-config --cflags/--libs libcrypto`. Alternatively set **both**
`OPENSSL_INCLUDE_DIR` and `OPENSSL_LIB_DIR` to the installed system library paths;
this bypasses pkg-config. `CC` and `AR` select native tools. The adapter checks
OpenSSL's major at compilation and before device allocation at runtime. Native
fresh/import/clone entry sets both process core-dump limits to zero and refuses
private-state allocation if the OS rejects that change.

```sh
cargo test -p faction-omemo --no-default-features
cargo test -p faction-omemo --features native
cargo clippy -p faction-omemo --all-targets --features native -- -D warnings
```

Only [six byte-pinned files](vendor/SHA256SUMS) compile. Builds never download,
generate, patch, or accept source/profile overrides. [Provenance and update
requirements](VENDOR.md) and [third-party notices](NOTICE.md) accompany the files.

`Device::try_clone` stages a complete committed context for an outer durable
checkpoint. `open_pending` retains an isolated whole-state candidate, including
store, session and skipped keys. Validate SCE, then commit that candidate in the
outer transaction. Rejecting a candidate requires abort/drop. Do not publish a
sealed message until the encrypted checkpoint and outbox are durable. Raw
`export` snapshots contain private keys; store them only inside the encrypted
member container. Returned snapshots/plaintext and SCE strings use `Zeroizing`;
the XML parser's temporary normalized allocations are not guaranteed wiped.

The native context holds one fixed verified peer/session. Explicit `ed25519:`
wire identity pins cannot change. Limits are plaintext 8192, ciphertext 8208,
key message 256, snapshot 32768 bytes, 32 skipped keys and 64 isolated prekey
verification derivations. Retained prekeys serve only the enclosing fixed-peer,
bounded session lifetime enforced by the checkpoint owner. They do not implement
live bundle publication, prekey rotation, or an unlimited delayed-mail lifecycle.

SCE accepts `urn:xmpp:sce:1` envelopes with exactly content/from/to/time/rpad,
`jabber:client` subject and body, exact address bindings and inert escaped text.
Subject/body must be nonempty and at most 128/1024 UTF-8 bytes. Total received
plaintext is at most 8192 bytes with a parser cap of 128 nodes; larger received
padding is accepted within that total bound. Send padding contains 128 random
bytes encoded as hex. Its fixed 256-character size provides bounded random
content, with limited length hiding compared with broader OMEMO padding
recommendations; it does not hide subject/body length across arbitrary sizes. DTDs, custom entities, comments, PIs, CDATA, nested or unknown
content/attributes and foreign namespaces reject. Built-in and numeric XML
escapes are allowed. Times use the initial XEP-0082 subset
`YYYY-MM-DDTHH:MM:SSZ`, years 1970 through 9999, with calendar validation and no
fractions, offsets or leap seconds. They must agree within 300 seconds of the
enclosing **sending** time; receipt time is irrelevant. This is a deliberately
bounded SCE subset, not a claim of complete XMPP/SCE extension support.

The native-only [experimental attachment metadata codec](../../dev-docs/attachment-metadata.md)
handles one bounded descriptor inside authenticated SCE. The existing text-only
SCE APIs and current checkpoint reject that extension. It does not provide blob
verification, storage or attachment delivery; default features remain empty.

The `interop_probe` Cargo example is a laboratory-only, uninstalled raw-state
channel for the unchanged [independent reference](../../experiments/200-omemo-interop/reference.py).
It requires the explicit `native` feature and accepts no arguments or file/TTY
input/output: duplicated stdin/stdout descriptors must both be FIFOs or sockets.
It bounds and drains 65,568-byte command lines and zeroizes line, scratch and
reply buffers. It exposes synthetic raw snapshots only through those private
pipes, so do not install it or use it for member data.

The unchanged fixture binds native@a.test/device 101 and
reference@b.test/device 202, with sending time `2026-10-24T12:00:00Z`. In addition
to native authentication, this example runs the actual product SCE decoder
against those fixed inbound address/time bindings before returning candidate
plaintext. Invalid SCE aborts the candidate immediately; valid SCE still requires
the fixture's independent validation and explicit commit. The laboratory `seal`
command deliberately accepts arbitrary raw bytes so the independent matrix can
produce authentic malformed-SCE rejection controls. Product mail uses its own
bounded queue/checkpoint API.

```sh
cargo +1.96.0 test -p faction-omemo --features native --example interop_probe
cargo +1.96.0 build -p faction-omemo --features native --example interop_probe --release
```

The independent Python runtime and its exact 15-wheel hash/license closure stay
in an owned disposable laboratory directory, outside the product dependency graph.
Its AGPL research components are not product dependencies. Execute the unchanged
reference runner with `--native` naming this example binary, private pipes, and
`--disposable-synthetic-fixture`. Host/native interoperability observations and
source identities belong in a new evidence record; earlier proof results remain
unchanged.

# Offline OMEMO-2 interoperability proof

This standalone Rust/generated-C experiment answers [#200](https://github.com/gonz0w/faction-bbs/issues/200)
and qualifies the explicit correction in [#202](https://github.com/gonz0w/faction-bbs/issues/202).
It is outside the product Cargo workspace. It adds no product dependency or mail
UI. Read the [report](../../dev-docs/research/0054-omemo-interop-netbsd.md) and
[saved record](../../dev-docs/evidence/omemo-interop/results.json) before using it.

The original Picomemo candidate fails independent reciprocal authentication.
Only `ad-order-v1` passes the unchanged Twomemo reference matrix. Build selection
is explicit; unknown selectors and source mismatches fail. Neither result is a
production cryptographic audit or complete XMPP client qualification.

## Build the pinned candidates

Use a new owned disposable directory outside Git. Obtain Picomemo revision
`835d8277108b96c691fb6a992512f37315cb52d7` from the
[pinned archive](https://codeload.github.com/mierenhoop/picomemo/tar.gz/835d8277108b96c691fb6a992512f37315cb52d7).
Its SHA-256 must be
`7b1519bb1edfd6a86d6595f6357270b6999a0393cafdb63d76257beb95647301`.
Verify it before extraction. Keep separate original and corrected source trees;
apply `picomemo-ad-order.patch` to the latter with `patch -F0 -p1` from its root.
Require clean application without offsets/fuzz. The patch SHA-256 is
`8bc5d7560065865c81947099c7dabd401cdb21de3c3002d4466e3ce77f395465`.
`build.rs` checks all ten selected upstream inputs against the corresponding
original/corrected map. It does not fetch, patch, install or fall back for you.
Retain [upstream notices](NOTICE.md).

From this experiment directory, the corrected host commands have this form:

```sh
PICOMEMO_SOURCE=/absolute/owned/corrected PICOMEMO_CANDIDATE=ad-order-v1 \
  CARGO_TARGET_DIR=/absolute/owned/target-corrected \
  cargo +1.96.0 test --release --locked --offline
PICOMEMO_SOURCE=/absolute/owned/corrected PICOMEMO_CANDIDATE=ad-order-v1 \
  CARGO_TARGET_DIR=/absolute/owned/target-corrected \
  cargo +1.96.0 build --release --locked --offline
```

Use `PICOMEMO_CANDIDATE=baseline`, the untouched source and a separate target for
the rejected baseline. Rust has no Cargo dependencies. The native compiler builds
`gen/omemo2.c`, `hacl.c`, `openssl.c` and the local C shim with C11/O2; it links
`libcrypto`, not `libssl`. Host discovery needs `pkg-config libcrypto`.
On NetBSD use both `OPENSSL_INCLUDE_DIR=/usr/include` and
`OPENSSL_LIB_DIR=/usr/lib`; no pkg-config is required. The measured native build
used Rust 1.96.0, GCC 12.5.0 and base OpenSSL 3.5.7. Build natively: this script
selects OS hash tools and does not qualify cross compilation or all NetBSD ports.

## Run the independent matrix

The Python reference is research tooling only. The record's `reference_packages`
contains all fifteen exact wheel URLs, SHA-256 hashes and license metadata; use
those artifacts in a private isolated Python 3.13 environment, with no dependency
resolution or alternate wheels. Verify hashes and screen archive entries before
installation. Remove bootstrap pip before the final run and verify the resulting
runtime contains no JS/TS paths. The recorded macOS ARM64 wheels do not imply
NetBSD Python support. The NetBSD proof used the host reference over private SSH
pipes to an unprivileged disposable guest account. No live XMPP service is used.
The reference package code remains unchanged; the fixture asserts both actual
skipped-key limits are 32.

With that isolated Python and the corrected proof binary:

```sh
/absolute/owned/reference-python reference.py \
  --native /absolute/owned/target-corrected/release/faction-omemo-proof \
  --output /absolute/owned/corrected-results.json --disposable-synthetic-fixture
```

For the original binary, also pass `--rejected-baseline`. The baseline mode
requires both first messages to decrypt and their replies to reject without
committing state; it reports `rejected`, never a qualification pass.
For remote execution or `/usr/bin/time -l`, use `--native-argv-json` instead of
`--native`. That owner-only JSON file is a trusted argument array, not a shell
command or member-controlled input. The private transport must preserve stdin and
stdout as pipes. `--rss-unit bytes|KiB` labels the platform's raw time output;
without time instrumentation, RSS is not measured. Never publish private argv,
transport keys, endpoints, snapshots or error files. The harness removes synthetic
state/diagnostics on exit; a failure may leave a mode-0600 `.private-error` beside
the chosen output for owner inspection and removal.

## Private proof protocol and bounds

The CLI accepts no arguments and requires pipe/socket stdin and stdout. TTY or
regular-file routes fail before private output, and core dumps are disabled. It
must never become a BBS forced command or general network service. Hex fields are
lowercase; public replies are `ok [fields...]` or `err`. An incomplete final line
returns only `err` and exits cleanly. Invalid protocol input cannot supply a path,
command or dynamic library.

| Command | Purpose |
| --- | --- |
| `new` | Create synthetic identity/prekeys; return public bundle fields. |
| `trust ed25519:HEX` | Enroll one exact Ed25519 wire identity; changes/legacy labels reject. |
| `start IK SPK SIG SPK_ID PK PK_ID` | Verify/start from the peer's public bundle. |
| `seal SCE_HEX` | Return key message, prekey flag and encrypted payload. |
| `open KEYMSG_HEX PREKEY_FLAG PAYLOAD_HEX` | Stage full candidate state and privately return plaintext. |
| `commit` / `abort` | Accept only after trusted SCE validation, or discard all staged state. |
| `export` / `import SNAPSHOT_HEX` | Private research restart snapshot; import only into an empty process. |
| `convert ED_PUBLIC_HEX` | Call the existing HACL identity conversion oracle. |

A native context holds one device/verified peer. A Rust process mutex serializes
upstream global callbacks. Clone-and-commit includes session, store and skipped
keys; authentication, device, payload or SCE failure discards the entire proposal.
Pending SCE acceptance blocks other operations/export. The caller must bind JID,
device ID and typed Ed25519 pin before opening; the C library cannot supply that
application authority.

Limits: plaintext 8192 bytes, ciphertext 8208, key message 256, snapshot 32768,
32 retained skipped keys and 64 transient verification derivations. CLI line cap
is 65568 bytes. The isolated fresh-session check for an established prekey wrapper
verifies its claimed identity and discards its transient state. This is not a new
ratchet/curve implementation. Heartbeat routing rejects; synthetic prekeys are
retained for delayed test cases only.

Snapshots contain **unencrypted synthetic private material**. Original snapshot
magic is `F200S001`, corrected is `F200A001`; cross-candidate imports reject.
Corrected local session state requires final canonical field 16 (`80 01 ROLE`),
with role 0/1/2 validated against initialization. This is a private format, not an
XMPP wire extension, migration or crash-safe keystore. [#203](https://github.com/gonz0w/faction-bbs/issues/203)
owns encrypted transactional persistence before any transport use.

## Offline repository checks

From the repository root:

```sh
python3 -m unittest discover -s tools -p 'test_omemo_interop*.py'
python3 tools/check_omemo_interop_evidence.py
```

These need only Python's standard library. They validate the SCE/identity/protocol
helpers and saved-source/result contracts; they do not rerun cryptography, Python
reference packages or a VM. Full native reruns need the pinned external sources
above. Remove only your owned disposable directory and its synthetic keys after
exporting sanitized results. Leave the saved owner playground stopped and intact.

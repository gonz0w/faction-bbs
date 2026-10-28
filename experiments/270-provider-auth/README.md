# Provider authority experiment

This isolated Rust experiment is outside the product workspace. It binds synthetic
certificate digests to explicitly approved attachment principals and connects a
trusted TLS adapter to the existing ciphertext store. It is not activated by SSH,
native home mail or ordinary board operation. See the
[authority contract](../../dev-docs/provider-authority.md) for identity, lifetime,
resource and recovery boundaries.

The registry and connection handles are in-memory experimental authority, not a
persistent enrollment system or durable anti-rollback. The loopback TLS fixture
uses synthetic in-memory keys and private owned stores; its intended verified-peer
boundary proves TLS client-key possession only, not possession of the member's
OMEMO identity key. It does not implement a provider or two-home delegation.

On Darwin arm64 (macOS 27.2), the isolated experiment ran 22 tests: 12
authority/core and ten TLS test functions, including multiple variants. The
selected binary reported OpenSSL 3.6.3 using the explicit OpenSSL 3 libraries.
Handshake and frame socket stalls reached approximately five and ten seconds with
zero dispatch. Separate controlled post-BIO scheduling delays and a combined
TCP/TLS admission delay also denied without dispatch; the owned handler joined within five seconds and its closed endpoint
refused further connections. No fixture roots remained. These host observations
do not establish a NetBSD, hardware or capacity result.

[Host evidence](../../dev-docs/evidence/provider-authority/README.md) records those
original observations. A separate [native NetBSD record](../../dev-docs/evidence/provider-authority-netbsd/README.md)
records serial and default-parallel runs of 23 tests each: the original 22
controls and one TCP-observer regression. The native source starts from accepted revision `39bfd2c` with a
test-only closure-observer correction; production Rust and default paths are
unchanged. The historical host record remains bound to its original source.

## Reproducing the isolated checks

Use Rust 1.96.0, an existing OpenSSL 3 development installation and an already
available locked source cache. The host observation selected the existing
OpenSSL 3 include and library directories explicitly with `OPENSSL_INCLUDE_DIR`
and `OPENSSL_LIB_DIR`; the host default OpenSSL 4.1 was not selected. Choose
those paths for the intended installation before invoking Cargo. No installation
or service activation is part of this experiment.

From the repository root, run the source check with
`RUSTUP_TOOLCHAIN=1.96.0 python3 tools/check_provider_auth_dependencies.py`.
Set `CARGO_TARGET_DIR` outside the experiment source tree, for example to
`$PWD/local/build/provider-auth`, so the source guard scans source only. The
serial test command is `cargo +1.96.0 test --manifest-path
experiments/270-provider-auth/Cargo.toml --release --locked -- --test-threads=1`.
The recorded host runs also used `--offline` and the explicit OpenSSL paths.

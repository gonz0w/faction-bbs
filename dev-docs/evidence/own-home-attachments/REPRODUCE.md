# Native evidence reproduction notes — draft

These are commands for a future independently authorized run, not a claim that
the private runtime helper or its fixtures can be replayed. Use the current
[operator guide](../../../docs/provider-operator.md) for independently approved
homes, credentials and policies. The selected runtime, build and helper source
identities match commit `9fddcb151c2b94116f849408305f5c9a35ac5fe9`. The separate
9,358-file retained inventory includes unowned non-runtime guidance excluded from
that commit; its hash does not assert whole-tree identity.

Prepare a fresh, marked, owned private root with mode `0700`, an absolute shared
target directory beneath it, and an existing qualified Rust 1.96 Cargo installation
with dependencies already available offline. Substitute your own paths below. Use
OpenSSL 3 and its local headers and libraries. These commands do not install tools,
provision homes, change VM resources or grant permission to run services.

```sh
export PATH="/absolute/path/to/qualified/rust-1.96/bin:$PATH"
export CARGO_HOME="/absolute/path/to/qualified/offline-cargo-home"
export CARGO_TARGET_DIR="/absolute/path/to/owned-private-root/target"
export CARGO_BUILD_JOBS=1 CARGO_NET_OFFLINE=true
export OPENSSL_DIR=/usr OPENSSL_INCLUDE_DIR=/usr/include OPENSSL_LIB_DIR=/usr/lib
export OPENSSL_NO_VENDOR=1

python3 tools/check_provider_dependencies.py
python3 tools/check_delegation_codec_dependencies.py
python3 tools/check_network_mail_dependencies.py
python3 tools/check_provider_auth_dependencies.py

cargo build --manifest-path crates/faction-provider/Cargo.toml --bin faction-provider --features operator,delegation-verifier --release --locked --offline
cargo build -p faction-client --features own-home-provider --bin faction-client --example device_provider_console_fixture --release --locked --offline
cargo test --manifest-path crates/faction-provider/Cargo.toml --features operator,delegation-verifier --no-run --release --locked --offline
cargo test -p faction-client --features own-home-provider --all-targets --no-run --release --locked --offline
```

The guards take no `--offline` argument; the environment above selects offline
Cargo behavior. This phase compiles test executables and runs no acceptance tests.
The observed serial compilation used 2 GiB and two CPUs. **Stop here:** restore
and independently observe 1 GiB and two CPUs before executing the tests below.
This document grants no machine or VM authority and performs no resource changes.
Preserve failures and independently record the resource settings before each phase.

```sh
cargo test --manifest-path crates/faction-provider/Cargo.toml --features operator,delegation-verifier --release --locked --offline -- --test-threads=1 --nocapture
cargo test -p faction-client --features own-home-provider --all-targets --release --locked --offline -- --test-threads=1 --nocapture
```

The original client test encountered SUN_LEN with a long temporary path; the
subsequent run used a fresh short private `TMPDIR`, retaining the original test
and source.

After separately preparing an approved fixture, the maintained probe accepts these
explicit paths. The root must be mode `0700`, owned by the current user, and contain an `OWNED` file
whose exact contents are `Owned authenticated-provider console proof` followed by
a newline. The output is private and
must not be published without review. Replace every placeholder with a real local
path, including the output path.

```sh
python3 tools/probe_device_provider_console.py --route own-home \
  --client "$CARGO_TARGET_DIR/release/faction-client" \
  --fixture "$CARGO_TARGET_DIR/release/examples/device_provider_console_fixture" \
  --operator "$CARGO_TARGET_DIR/release/faction-provider" \
  --owned-root /absolute/path/to/marked-owned-private-root \
  --openssl /usr/bin/openssl --output /absolute/path/to/private-output.json
```

Record actual controls, calls, durations, verified size, private new save and
no-overwrite result, keeping original limits. Retain failed attempts and inspect
selected stores without publishing secrets. Record unavailable exact token values,
process and port checks, and complete teardown separately.

[native-results.json](native-results.json) records hashes of retained
stage results, results, custody receipts and corpora, plus selected lifecycle raw
receipts. Hashes identify private artifacts; they do not themselves prove behavior,
public reproducibility, complete privacy or an audit. The later
[BBS/SFTP outage and snapshot closure](bbs-outage.json) complete that separate
software qualification. Final review and exact-head CI accompany the closing PR.

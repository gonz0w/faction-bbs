# Reproducing issuer and wire checks

These are normalized reproduction steps, not a claim that a new execution has
occurred. Preserve exact selected source, commands, environment, logs and binary
hashes before and after execution. Use an independently owned disposable fixture;
never start the owner playground. Match the recorded Rust/Cargo 1.96 and explicit
OpenSSL 3 installation in [tools.json](tools.json). On macOS select the installed
OpenSSL 3 include/library paths deliberately; NetBSD used its base OpenSSL 3.5.7.

Confirm the source against the frozen commit and inspect differences before running:

```sh
git show --stat e66ba011e7dfb63c9e04bb02dede238dd5806f64
git diff e66ba011e7dfb63c9e04bb02dede238dd5806f64 -- crates/faction-provider crates/faction-delegation-codec
```

Run the current source and dependency guards in the documentation workflow after
warming its locked metadata graphs for all listed Linux, NetBSD and macOS targets,
including wire with and without durable policy. Source qualification precedes
compilation; do not execute upstream tests or CI. Use the current workflow as the
command authority for exact metadata invocations:

```sh
python3 tools/check_provider_dependencies.py
python3 tools/check_delegation_codec_dependencies.py
```

For recorded-file verification, obtain the manifest path and expected SHA-256 from
the canonical evidence index and pass those exact values to
`python3 tools/check_recorded_files.py MANIFEST SHA256`. This checks bytes and
membership, not execution or provenance by itself.

Use a short private `0700` temporary directory marked `OWNED`, with separate build
and fixture paths. Set `CARGO_BUILD_JOBS=2`, `CARGO_TARGET_DIR` to that private
build directory, and the explicit OpenSSL 3 environment. After the source gate,
run the provider's current release, locked, offline tests serially for each of the
14 feature profiles in [results.json](results.json), and strict Clippy for the
same profiles. For example:

```sh
cargo clippy --manifest-path crates/faction-provider/Cargo.toml --all-targets --release --locked --offline --features delegation-wire,durable-policy -- -D warnings
cargo test --manifest-path crates/faction-provider/Cargo.toml --release --locked --offline --features delegation-wire,durable-policy -- --test-threads=1
cargo test --manifest-path crates/faction-delegation-codec/Cargo.toml --release --locked --offline -- --test-threads=1
```

The wire tests include an actual device session, its closure, relay issuance and
fresh introspection over separate verified TLS connections; do not replace them
with codec-only parsing. Record the actual test and compile-fail results separately.
Native reproduction uses a disposable NetBSD 11 ARM64 software environment with
two CPUs. Small profiles built and ran at 1 GiB; `blob-iq` and `operator` compiled
at 2 GiB and their existing binaries ran at 1 GiB. Check source and executable
identity across phases; compile-fail doctests still invoke rustc. Do not infer a
fresh 1 GiB build failure.

For host console controls, build the feature-gated client and
`device_provider_console_fixture` from the same source after applicable source
gates. The harness requires an absolute private `0700` owned root with the exact
marker bytes `Owned authenticated-provider console proof\n` in its `OWNED` file:

```sh
python3 tools/probe_device_provider_console.py --client "$CLIENT" --fixture "$FIXTURE" --owned-root "$OWNED" --openssl "$OPENSSL" --output "$RESULT"
```

Use new synthetic credentials, retain hashes and sanitized results, verify child
reaping and private fixture removal. The recorded console run was on the host;
no fresh NetBSD production-client console journey is claimed. A private TLS BIO
gate, separate plaintext kernel retry fixture and synthetic clock tests have the
distinct limits described in the [evidence summary](README.md).

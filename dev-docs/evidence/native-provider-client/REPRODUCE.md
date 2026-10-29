# Reproducing the recorded software layers

Use the source snapshot and overlays in [inputs.json](inputs.json), rather than
substituting current working files for an earlier observation. Source objects are
identified by SHA-256 and repository-relative references. Original broader captures,
executable bytes, private fixture credentials and raw logs remain in verified
private retention; they are not needed by the offline recorded-file guard.

## Offline record check

Run the repository's `tools/test_native_provider_client_evidence.py` unittest. It
uses the existing `check_recorded_files.verify` function with the exact manifest
hash and reference count. It checks source/record bytes and package membership,
without network, compilation, runtime fixtures or VM execution.

## Dependency and build checks

Before compiling, run the existing provider, authority, network-mail and OMEMO
source guards against the exact locked profile. Their inventories qualify complete
selected archives/source trees, licenses, platform features and no-JS constraints.
Default/native-mail/network-mail graphs must exclude the provider. Device-provider
uses the single optional local provider edge with default features disabled.

The recorded toolchain is Rust/Cargo 1.96.0 and an already installed OpenSSL 3.
Keep compilation targets and a short marked private TMPDIR outside the source
checkout. Use the recorded compiler-job and guest-memory envelope; do not install
packages, boot saved owner images or change host/network policy for reproduction.

With the verified Cargo 1.96.0 executable, representative commands are below.
Use the recorded release/debug profile; add `--release` for the native release
artifacts. These are normalized examples, not private captured argv.

```sh
cargo test --manifest-path crates/faction-provider/Cargo.toml --locked --offline -- --test-threads=1
cargo test --manifest-path crates/faction-provider/Cargo.toml --features blob-iq --locked --offline -- --test-threads=1
cargo test -p faction-client --features device-provider --test mail_device_provider --locked --offline -- --test-threads=1
cargo test -p faction-client --features device-provider --lib mail_device_provider::tests --locked --offline -- --test-threads=1
```

For the recorded NetBSD test-child setup, create the private 0600 file selected by
`OPENSSL_CONF="$OWNED/openssl-fixture.cnf"` with exactly these static bytes:

```ini
[req]
distinguished_name=dn
[dn]
```

Its SHA-256 is `a1c45aa35abd0050b556a5b709603b4b9278886b8c8254ac7ef5a5e9d8eb638d`.
The already installed Python executable was named `python3.13`. A private 0700
test-only `python3` alias in a 0700 `$TEST_BIN` directory contained exactly:

```sh
#!/bin/sh
exec /usr/pkg/bin/python3.13 "$@"
```

Its SHA-256 is `d2a8150c06c697cbbab23d1aadfd82d0f4dae772c69fa9a15fe419868228a3af`.
Prefix only test-child PATH with `$TEST_BIN`; do not install a system alias or
change global OpenSSL configuration. These are setup prerequisites, not credentials.
The production console proof did not inject these fixture environment overrides.

The finite test layers are:

1. Provider default and `blob-iq` suites, including compile-fail privacy controls,
   framing/bounds/partial/error/flush tests and current authority/lifecycle checks.
2. Native `device-provider` integration tests and private current-thread connection,
   worker and composed synthetic-write controls. The actual pending-TCP backlog
   precondition must hold on the tested platform; arbitrary setup errors are not
   success. macOS's original unavailable precondition remains recorded.
3. Existing default/native/network client, codec, storage, checkpoint/recovery and
   ordinary BBS/file compatibility controls appropriate to the selected snapshot.
4. The production-console proof with synthetic private credentials, loopback TLS
   provider sessions and a separate maintained encrypted-mail relay. Run classic
   and Unicode modes with a 1 MiB attachment and original 60-second/65-call bounds.

`tools/probe_device_provider_console.py` and the explicitly feature-gated
`device_provider_console_fixture` are the maintained journey harness. Follow their
public command interface with newly created owned fixtures, for example:

```sh
python3 tools/probe_device_provider_console.py --client "$CLIENT" --fixture "$FIXTURE" --owned-root "$OWNED" --openssl "$OPENSSL" --output "$RESULT"
```

Do not reuse private recorded argv, ports, paths, credentials or member content. Record actual input and
executable hashes before starting. Keep successful serve counts distinct from
attempt counts and connection-loss observations distinct from guaranteed reply loss.

## Primary native qualification

The native record uses an owned disposable NetBSD 11 ARM64 software clone after
asset/package/cache/source preflight. Compiler and runtime envelopes differ, and
warm rebuilds are identified separately from initial builds. Test-child PATH and
OpenSSL configuration prerequisites are recorded as setup facts, without global
host changes. Every native pass must be tied to its actual source and executable;
older 447-input observations do not become 448-input executions.

Preserve failed variants and original binaries before rebuilding. Stop/reap owned
helpers and verify fixture, controller/VM, forward and temporary build cleanup as
recorded in [teardown](teardown.json). Retain private originals before deleting
scratch. The saved owner playground remains stopped and intact.

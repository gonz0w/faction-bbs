# Reproducing the recorded layers

Use frozen commit `57020dcb2c04b62a58596b1e07c2310c6b17701a` for the current
product and the base-plus-source-set mappings in [inputs.json](inputs.json) for
earlier observations. Apply each exact retained source variant before its check;
substituting current code cannot reproduce an earlier RED. The selected 797-input
map is broader than execution. Binary bytes, credentials, raw logs and private
orchestration remain in private retention.

## Offline recorded-byte check

The new `tools/test_durable_provider_operator_evidence.py` unittest calls the
existing `check_recorded_files.verify` helper with the actual final manifest
hash and reference count. It checks saved bytes and selected folder membership;
it does not rerun tests, a VM or the production journey.

## Source qualification and host checks

Select and observe the already installed Rust/Cargo 1.96.0 executables before
checks. Set `RUSTUP_TOOLCHAIN=1.96.0` and use `cargo +1.96.0`. Keep compilation
outside the source tree, offline and within the recorded two-job team ceiling.
The final host stages used one compiler job and installed OpenSSL 3 with explicit
include/library directories and `OPENSSL_NO_VENDOR=1`. Do not install packages or
change host security, services or global OpenSSL configuration.

Run the existing provider, network-mail and provider-authority dependency guards
before compilation. The recorded qualification checks all 24 selected metadata
graphs and 167 complete existing registry archives/extracted sources, licenses and
no-JS constraints across Linux, NetBSD and macOS. Ordinary default/native/network
client graphs exclude the optional operator/durability selections.

Representative normalized commands follow; these are public recipes, not copied
private argv. Choose the recorded release or debug profile when comparing an
artifact: host provider library/TLS suites used release, and the maintained host
operator/client journey used debug.

```sh
python3 tools/check_provider_dependencies.py
python3 tools/check_network_mail_dependencies.py
python3 tools/check_provider_auth_dependencies.py
cargo +1.96.0 test --manifest-path crates/faction-provider/Cargo.toml --all-targets --release --locked --offline -- --test-threads=1
cargo +1.96.0 test --manifest-path crates/faction-provider/Cargo.toml --features durable-policy,blob-iq --all-targets --release --locked --offline -- --test-threads=1
cargo +1.96.0 test --manifest-path crates/faction-provider/Cargo.toml --features operator --test operator_cli --locked --offline -- --test-threads=1
python3 -m unittest discover -s tools -p test_probe_provider_operator.py
```

Run strict Clippy for the same selected profiles with `--all-targets -- -D warnings`
and `cargo +1.96.0 fmt --manifest-path crates/faction-provider/Cargo.toml -- --check`.
Keep source preflight before each changed graph; no registry-version addition is
needed. Compare counts by feature variant, without adding overlapping suites or
counting the no-environment subprocess helper as independent behavior.

## Production console journey

Create new marked owned 0700 fixtures and materialize the explicit operator,
native `device-provider` client and member-seed example. Supply synthetic,
independently preapproved public policy and separate private TLS files. The
member seed creates device-local fixture material; the maintained operator is
the actual provider. Do not substitute a fixture server or inject received mail.

```sh
python3 tools/probe_provider_operator.py --client "$CLIENT" --fixture "$MEMBER_SEED" --operator "$OPERATOR" --owned-root "$OWNED" --openssl "$OPENSSL" --output "$RESULT"
```

The probe drives real classic/Unicode production consoles, an owned loopback
listener and a separate maintained encrypted-mail relay. It checks 1 MiB byte
equality, release only after commit, incomplete-upload restart/exact retry,
offline policy lock refusal, revocation and explicit reset/history behavior.
Use new private credentials and actual ephemeral endpoints; never reuse private
recorded paths, keys, ports, profiles or member values. Keep the original
60-second/65-call console bound, four-chunk SDK calls and ten-second upload cycle.

Stop/reap owned children and verify fixture removal. Preserve source, executable,
result and failure identities before editing or rebuilding. Available-value
privacy scans and retained originals do not establish coverage of secrets that
were never captured. Native software qualification needs its separate recorded
preflight, build/runtime envelopes and verified disposable-clone teardown. The
saved owner playground remains stopped and intact.

## Fresh NetBSD software layer

Use an owned disposable NetBSD 11 ARM64 clone after actual package/cache/source
preflight. Verify all 797 frozen selected inputs, 167 existing registry archives
and extracted source trees, observed tool versions and installed libraries before
compilation. The recorded build/check envelope was two CPUs, 2128732160 physical
memory bytes and at most two compiler jobs; the two existing narrow C helpers
used one job. Runtime restarted at two CPUs/1054990336 bytes, with no compilation
in the materialized-test or production phases. Private orchestration and direct
C command captures are retained privately rather than published as project code.

The test-child setup used these exact static OpenSSL configuration bytes:

```ini
[req]
distinguished_name=dn
[dn]
```

Their SHA-256 is `a1c45aa35abd0050b556a5b709603b4b9278886b8c8254ac7ef5a5e9d8eb638d`.
An owned 0700 test-only interpreter alias contained exactly:

```sh
#!/bin/sh
exec /usr/pkg/bin/python3.13 "$@"
```

Its SHA-256 is `d2a8150c06c697cbbab23d1aadfd82d0f4dae772c69fa9a15fe419868228a3af`.
Keep those settings scoped to the test children that need them. The production
console did not inject these test-only environment overrides. Preserve the
original three-pass/one-error Python command and its separate one-case replay;
do not report an unexecuted full rerun. Use the explicit source/binary identity
for each default/native/network/device profile; their counts overlap.

The separate eight-control home-mail outage journey checks pending independent
BBS approval, unchanged encrypted state while the home is unreachable, store
locking and same-picker restricted SFTP/return. It does not turn provider
approval into BBS membership. Preserve private originals, then verify stopped
owned services/VM, reaped children, closed forwards, fixture/root removal and
unchanged saved owner assets before claiming teardown.

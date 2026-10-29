# Reproducing the recorded layers

Use maintained source `358e35ceec242763d8ea2442a1cfc2cfc390cb7c` for the final suite
and the exact source-set mappings in [inputs.json](inputs.json) for earlier measured
variants. The mappings preserve captured bytes; replacing them with current code
cannot reproduce a historical RED. Runtime bytes are unchanged from `49735783`; the final source changes only the
unsafe-directory fixture test. The broad 810-input selection is larger than
execution and the readable Git anchors retain supporting public context. Original
private argv, logs, credentials, binaries and orchestration remain private.

## Source and tool qualification

Observe the installed Rust/Cargo 1.96 executables before checks; set
`RUSTUP_TOOLCHAIN=1.96.0` and use `cargo +1.96.0`. Run offline in a new owned private
capture/target directory, with at most two compiler jobs team-wide. The accepted
host lane initially had two exclusive jobs and later one alongside the operator's
one-job lane. Use an existing qualified OpenSSL 3 installation with explicit include
and library directories and `OPENSSL_NO_VENDOR=1`; install no package or service.

Warm the selected metadata graphs before the dependency guards inspect them.
The source qualification uses four root profiles (default/native/network/device)
and five provider profiles (default/IQ/durable/preparation/operator), each for Linux,
NetBSD and macOS, for 27 graphs. Preserve locked metadata/setup and graph-ordering
failures separately; a zero-test setup failure is not behavioral RED.

Run the maintained provider, network-mail and provider-authority dependency guards,
which verify the exact graphs, local edges, complete existing archive/extracted
sources, licenses and no-JS scope. The accepted graph change needs no new registry
version. The source-qualification receipt gives the original and supplemental gate
identities and distinguishes their counts.

## Public API and maintained-command controls

Representative commands below are normalized public recipes, not recorded private
argv. Build without executing first, then materialize each reported product/test
executable into a new private stage, hash it and record its actual library links
before executing it. Preserve every original input, failed candidate and executable
before edits, replacement or rebuilding. Use recorded debug/release profiles when
comparing binary identities; these host preparation consumers used debug.

```sh
cargo +1.96.0 test --manifest-path crates/faction-provider/Cargo.toml --features preparation --test preparation --locked --offline --no-run --message-format=json
cargo +1.96.0 test -p faction-client --features device-provider --test device_provider_preparation --bin faction-client --locked --offline --no-run --message-format=json
cargo +1.96.0 test --manifest-path crates/faction-provider/Cargo.toml --features operator --all-targets --locked --offline --no-run --message-format=json
```

Select the actual retained test executables from those Cargo artifact records.
Run the preparation API executable and client binary tests with `--test-threads=1`.
The client command/PTTY consumer requires an existing owned 0700 capture directory
and the absolute path of the separately materialized current client:

```sh
env FACTION_SETUP_CLIENT_EXE="$CLIENT" FACTION_SETUP_CAPTURE_DIR="$PRIVATE_CAPTURES" "$CLIENT_SETUP_TEST" --test-threads=1
```

The consumer creates marked random owned fixture children, uses actual hidden
unlock and the maintained commands, retains bounded private transcripts, and proves
fixture removal. Wrong secrets, cancel keys, expired/history/revoked state, each
profile binding mismatch, ambiguous or mismatched credentials, unsafe modes/links,
output collision and actual DER bytes are substantive checks. Existing helper
entries are invoked by their parent tests and are not extra independent passes.

Run strict Clippy for the same selected profiles with `--all-targets -- -D warnings`,
format checks, repository checks and the maintained Python consumers. No failing
check is waived and ordinary/default/native/network behavior remains independent.

## Prepared production composition

Create fresh marked owned 0700 fixtures. Materialize/hash the actual device-provider
client, operator and existing member-seed example before running the maintained
probe. Supply new synthetic credentials and a complete independently preapproved
proposal; never reuse recorded keys, endpoints, member values or output paths.

```sh
python3 tools/probe_provider_operator.py --client "$CLIENT" --fixture "$MEMBER_SEED" --operator "$OPERATOR" --owned-root "$OWNED" --openssl "$OPENSSL3" --output "$PRIVATE_RESULT" --prepared
```

The prepared mode calls both new client commands with independent hidden unlocks,
then the operator check using a separately selected public certificate and complete
approved proposal. It checks unchanged checkpoint/policy/blob bytes before a
**separate deliberate init/apply**. It then drives actual classic and Unicode
production-console upload, encrypted receipt, fetch, verification and private save,
plus restart, exact retry, wrong pin/key, revocation and reset-history controls.
The payload is 1 MiB, each SDK call remains at most four chunks/ten seconds including
mail release, and each console action remains bounded to 65 calls/60 seconds.

Run the same maintained probe in a fresh fixture **without** `--prepared` for the
independent 26-control legacy path. Do not count an old saved result as a fresh run.
Stop/reap owned children and verify fixture teardown. Protect retained captures and
retain the available private-value corpus before deleting ephemeral values. A scoped
available-value scan cannot prove coverage of in-memory or previously deleted data.

## Fresh NetBSD layer

Use an authorized marked disposable NetBSD 11 ARM64 clone after source/package
preflight. The accepted run verified the original 810 source inputs and all 167
existing registry archives/complete source trees before compilation; the separately
reviewed test supplement replaces only the operator CLI fixture test. Linux/macOS
metadata qualification remains the distinct host 27-output gate. Native records
three actual source-guard invocations before initial and supplemental compilation;
do not claim 27 separately retained native raw metadata files.

The observed tools are Rust/Cargo 1.96, OpenSSL 3.5.7 and Python 3.13.14. Build/checks
used two CPUs/2128732160 memory bytes and at most two compiler jobs; runtime used
two CPUs/1054990336 bytes with no compilation. The two existing narrow C helpers
and the later test-only supplement used one compiler job. Preserve all actual
ELF/source/library identities before execution or overwrite; count 69 original
stable ELF copies/63 distinct hashes, 12 product roles, 8 libraries plus the loader,
and two separate corrected-test artifact copies precisely.

The guest lacked the expected default OpenSSL configuration and an unversioned
`python3`. Owned test-child-only inputs supplied the following exact static bytes:

```ini
[req]
distinguished_name=dn
[dn]
```

Configuration SHA-256:
`a1c45aa35abd0050b556a5b709603b4b9278886b8c8254ac7ef5a5e9d8eb638d`.

```sh
#!/bin/sh
exec /usr/pkg/bin/python3.13 "$@"
```

Alias SHA-256:
`d2a8150c06c697cbbab23d1aadfd82d0f4dae772c69fa9a15fe419868228a3af`.
Keep these inside owned fixtures and scoped to the relevant test children. The
production console received neither test-only environment injection. No system
configuration, alias, package, service or owner-playground change is authorized.
The native fixture used a short owned temporary path; do not substitute a long
path that changes existing UNIX-socket setup preconditions.

Retain the initial 18-stage command failure and the 21 previously unrun continuation
stages separately. An added host invariant exposed the umask-dependent fixture;
explicit chmod and the same invariant/refusal assertion then passed. The corrected
native case passed once with 11 other cases filtered. A subsequent metadata-wrapper
permission failure happened after that passing test; it was not a product failure
or an excuse to replay/relabel the completed case. Preserve the missing original
metadata-alias limitation and use retained raw stdout/per-role logs as the original
execution evidence.

## Recorded-byte verification

The final evidence unittest uses the existing `check_recorded_files.verify` helper
with the actual immutable manifest SHA-256 and reference count. It checks saved
bytes and selected-folder membership; it does not execute Cargo, a provider, a VM
or a production journey. Normalize curated prose/JSON to exactly one final LF before
computing that manifest; preserve measured source bytes without trimming them.
The guard's first real missing-manifest RED must precede publication, and the same
guard must then pass against the exact reviewed records and source objects.

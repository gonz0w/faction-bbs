# Reproducing composed policy and publication checks

Use the [documentation workflow](../../../.github/workflows/documentation.yml)
as the command and feature-matrix authority. Preserve exact selected source,
fixtures, tool versions, executable identities, arguments, logs and failures before
changing measured bytes. Use a private marked disposable root and leave the owner
playground stopped. Follow the workflow's locked metadata and source gates before
compilation, the 16 separate provider feature selections, standalone codec tests,
strict host Clippy and serial test settings. Do not replace the matrix with a
broader feature combination or infer a client result from a provider test.

The recorded native runtime was NetBSD 11 AArch64 with Rust 1.96 and OpenSSL 3.5.7,
two CPUs and 1 GiB. Fourteen provider profiles compiled at 1 GiB; blob-iq and
operator were compiled at 2 GiB and their tests ran at 1 GiB. Treat build and
runtime envelopes separately. For example, after the workflow's source and
metadata gates, the following normalized commands select the provider profiles;
the workflow contains the complete matrix and host Clippy selections:

```sh
cargo test --manifest-path crates/faction-provider/Cargo.toml --release --locked --features delegation-verifier -- --test-threads=1
cargo test --manifest-path crates/faction-provider/Cargo.toml --release --locked --features delegation-verifier,durable-policy -- --test-threads=1
```

For native client fixture prerequisites, including the private OpenSSL
configuration and test Python alias, see the existing [native client reproduction
guide](../native-provider-client/REPRODUCE.md). Preserve the Cargo-reported client
executable before other builds can replace an alias, and verify the exact client
and fixture identities used for console checks. Do not pass test fixture environment
overrides into the production console.

For the component positive, retain evidence of actual
authenticated sender publication, Store reopen and recipient retrieval in both
directions without SetupAccess. Preserve failed attempts and distinguish synthetic
clock or scheduling controls from live kernel behavior.

Use only authorized disposable fixtures; retain sensitive raw material privately
and verify cleanup separately. Reproduction does not establish provider deployment,
independent interoperability, physical hardware or supported capacity. Review the
separate results and remaining gates in the [record](README.md).

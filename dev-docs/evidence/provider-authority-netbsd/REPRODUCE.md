# Reproduce the native provider-authority experiment

This procedure describes a bounded software experiment, not provider deployment or
production support. Use accepted revision `39bfd2cecf5d073665d5f8a73349ad37fe9de0f2` with the
recorded test-only closure-observer correction, and a preflighted locked Cargo
cache prepared offline from the existing qualified sources. Install and fetch nothing.
The [earlier host record](../provider-authority/README.md) and its sources remain
unchanged. The native results and identities belong in this directory's evidence
record; source and library identities do not attest the complete loader or a
hermetic build.

Use an owned disposable NetBSD 11.0 evbarm/aarch64 guest and an owned persistent
working directory outside `/tmp`, which NetBSD clears on restart. Restrict guest
external networking and use guest-local loopback for the synthetic TLS fixtures.
Keep keys, raw transcripts and private endpoints out of Git. The saved owner
playground must remain stopped and intact. Record actual source, toolchain, cache,
CPU and memory identities before execution; the preflight for this run reported
Rust and Cargo 1.96, cc 12.5, base OpenSSL 3.5.7, and two CPUs with 2,128,732,160
bytes of physical memory for the build. `rustfmt` and `clippy` were available.

From the accepted repository root in the guest, with a private owned working
root already prepared, set the paths below to that root. The source checkout and
cache must match the accepted inputs; do not overwrite the historical evidence.
Use the native Cargo installation directly; these commands do not assume rustup.

```sh
umask 077
export PATH=/usr/pkg/bin:/usr/bin:/bin:/usr/sbin:/sbin
export CARGO_HOME="/path/to/owned/persistent/cargo-home"
export CARGO_TARGET_DIR="/path/to/owned/persistent/provider-target"
export CARGO_BUILD_JOBS=2
export OPENSSL_INCLUDE_DIR=/usr/include
export OPENSSL_LIB_DIR=/usr/lib
export OPENSSL_NO_VENDOR=1
python3.13 tools/check_provider_auth_dependencies.py
cargo test --manifest-path experiments/270-provider-auth/Cargo.toml --release --locked --offline --no-run
cargo fmt --manifest-path experiments/270-provider-auth/Cargo.toml -- --check
cargo clippy --manifest-path experiments/270-provider-auth/Cargo.toml --all-targets --release --locked --offline -- -D warnings
```

Limit the build to two CPUs, 2 GiB and two jobs. Shut down and restart the owned
clone for runtime observations at two CPUs and approximately 1 GiB, retaining
the persistent source, cache and target directory. Restore the environment above,
then run the corrected experiment both serially and with its default parallelism:

```sh
cargo test --manifest-path experiments/270-provider-auth/Cargo.toml --release --locked --offline -- --test-threads=1 --nocapture
cargo test --manifest-path experiments/270-provider-auth/Cargo.toml --release --locked --offline -- --nocapture
cargo test --manifest-path experiments/270-provider-auth/Cargo.toml -p faction-files --all-targets --release --locked --offline -- --test-threads=1 --nocapture
```

The corrected native serial run passed 23 tests (12 authority/core, ten original
TLS functions and one TCP-observer regression), with none ignored or filtered:
53.85 seconds reported by the harness and 53.992171 seconds for the whole command.
The default-parallel run also passed all 23, with none ignored or filtered: 53.88
seconds reported by the harness and 54.030299 seconds for the whole command.
The current store run passed 18 tests in 0.492898 seconds; one existing helper
was ignored by the harness and invoked by its parent. The original build took 15.8058 seconds at
two CPUs and 2,128,732,160 bytes of physical memory. Observer RED and GREEN
recompiles used two CPUs and 1,054,990,336 bytes. See the [native evidence](README.md)
for source and binary identities, the observer correction and remaining checks.
Preserve failed or unavailable checks separately. Teardown must verify that owned
listeners, handlers, processes, scratch keys and stores have stopped or been
removed and that the saved assets remain unchanged.

Interpret observed socket stalls separately from controlled post-BIO scheduling
delays; neither establishes certificate-verifier performance. The original shared
five-second admission and ten-second cycle remain unchanged. Filesystem deadlines
are cooperative and cannot preempt blocked I/O. Registry and connection authority
are in memory and do not provide durable anti-rollback. This software experiment
does not establish physical hardware, capacity, enrollment or provider support.

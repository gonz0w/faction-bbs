# BBS-004 native session probes

Disposable local research, **not a deployable BBS**. `ALLOW` and `REVOKE` are unauthenticated
synthetic harness controls. POST/CHAT return canned responses; no durability or fan-out.
Unix sockets are created in private temp directories, never on IP interfaces.

```sh
python3 tools/measure_native_sessions.py --run
python3 tools/measure_native_sessions.py --check
```

Run from root. Fresh execution needs cc/rustc/cargo/go/ps and initial crate-registry access;
all compilation/sockets/binaries/caches are temporary. Mio is pinned by Cargo.toml/lock.
`--check` reads saved evidence only and needs no native compiler/dependency fetch.
The tool sets its own FD soft limit for 250 local connections; no host configuration.

See [scope, actual measurements and recommendation](../../dev-docs/research/0004-runtime-comparison.md).
Root crate code is BSD-3-Clause; selected third-party packages retain their recorded
MIT/Apache terms. No dependency source/binary is redistributed in this repository.

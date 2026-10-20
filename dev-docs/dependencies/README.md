# Local application dependencies

Reviewed 2026-10-20. [ADR 0008](../decisions/0008-local-core-and-credentials.md)
adopts the initial core dependencies; [ADR 0009](../decisions/0009-safe-terminal-menus-and-themes.md) adds a pinned TOML parser for declarative themes. [ADR 0011](../decisions/0011-private-export-restore.md) enables rusqlite backup/hooks/limits for private snapshot tools without adding registry packages. [ADR 0012](../decisions/0012-restricted-ssh-and-terminal-handoff.md) enables rustix fs/net and uses existing cc 1.9.0 to build a small NetBSD-only native CMSG bridge; no new registry package or runtime SSH library is added. `Cargo.lock` pins
versions/checksums. [The inventory](local-core.json) records selected dependency
versions, declared license expressions and features for NetBSD ARM64 and Linux
x86-64, including build dependencies. Current selected graphs contain 39 / 41 registry packages respectively; all added declarations offer MIT/Apache-2.0 choices except winnow (MIT). This source inventory alone is not runtime evidence; [native application execution](../research/0024-restricted-ssh-netbsd.md) is recorded separately. The lock may also contain unselected platform packages.

Run `python3 tools/check_rust_dependencies.py` after fetching/building. It asks
Cargo for each named selected graph, scans the fetched package trees for prohibited
JS/TS source/Node manifests/directories, and compares the reviewed inventory.
An intentional update uses `--write`, followed by source/license/feature review;
do not regenerate blindly to suppress a policy failure. Cargo may fetch sources.

Current declared crate terms offer MIT or Apache-2.0 choices (some add the LLVM
exception); rusqlite/libsqlite3-sys are MIT. Bundled SQLite carries its upstream
public-domain dedication. Dependency manifests/license files and SQLite's source
notice were inspected; retain these notices when packaging. Faction's BSD license
does not relicense dependencies. Release packaging/license collection remains work.

This is a filename and declaration check, not a legal opinion or a vulnerability,
unsafe-code, build-script, generated-code or runtime security audit. It covers the
named graphs, not every Rust target/feature or downstream distribution. No browser
JS backend, async runtime, embedded SSH library or external service is included. Base
OpenSSH is an OS dependency for the development gateway. The small faction-unix C/FFI
ownership boundary is explicit; other application crates still forbid unsafe Rust. The new
TOML dependency uses only std/parse/serde with default features disabled; some
unselected derive/proc-macro packages can remain in the lock without compilation.

ADR 0013 extends the same in-tree native helper with a small C11 lock-free signal
flag on all build hosts. It uses the existing C compiler/`cc` build dependency;
selected registry graph/versions are unchanged. Native descriptor ownership and
signal/process lifecycle require independent positive tests, not just compilation.

# 0023: Rust dependencies on the chosen NetBSD target

Date/retrieval: 2026-10-19 local; guest observations 2026-10-20 UTC.
Issues: #4/#9/#18/#23/#31. Status: measured compatibility, research-stage choices.
Authority: owner explicitly chose Rust and NetBSD as primary target on 2026-10-19.
Decision: [ADR 0006](../decisions/0006-rust-and-primary-netbsd.md).

## Question and declared constraints

Can the proposed native readiness networking, local SQLite and SSH-client access
path actually compile/run on NetBSD without JS/TS or a public HTTP service?
Preserve sysop approval, local data ownership and optional private-network access.
The owner target is a hobby SBC/Pi or VM, 1–2 cores / about 1 GiB host RAM and
100–250 connected members; this experiment measures compatibility, not that load.
NetBSD support for a machine does not imply Rust or our dependencies support it.

## Environment, provenance and method

An official NetBSD 11.0 evbarm-aarch64 disk and GENERIC64 kernel were booted in
QEMU 11.1.1/HVF on an ARM64 macOS developer host: 2 vCPUs, 1024 MiB configured,
1054990336 physical bytes reported by the guest. A disposable 12-GiB virtual disk
held native build tools; this was the stock image, not a size-optimized appliance.
Virtio block/net/RNG, no bridged network or host filesystem sharing; the only
host inbound forward bound loopback. No HTTP, overlay or public board was deployed.

Image and kernel SHA-512 matched the release-wide signed manifest; GPG reported a
good signature for fingerprint `271133262BF35CB50D4F504689261E17F5EF49FF`.
The public key came from official HTTPS, without independent web-of-trust checks.
The adjacent image-directory SHA512 list omitted our chosen image; the global
manifest supplied coverage. Exact URLs/hashes are in the saved evidence.

The first HVF boot panicked with IllegalInstruction in `vmt_probe`/`vm_cmd`.
Booting with kernel userconf and disabling `vmt` allowed this guest to run; it is
a recorded hypervisor-specific limitation. PMU initialization also warned.
Neither observation establishes behavior on real SBCs or other hypervisors.

Guest compiler: pkgsrc rust-bin 1.96.0 (LLVM 22.1.2), host triple
`aarch64-unknown-netbsd`; native C compiler GCC 12.5.0. Crates were resolved/locked
on the developer host, bundled outside the checkout, then built offline **inside
NetBSD** with Cargo `--frozen`. Both SQLite variants used the same source/lock.
See the [executable probe/reproduction](../../experiments/023-netbsd-dependencies/README.md)
and [sanitized observations](results/023-netbsd-dependencies.json).

## Observed results

| Component | Exact candidate | Observed evidence | Limit |
| --- | --- | --- | --- |
| Rust/OS | Rust 1.96.0, NetBSD 11.0 ARM64 | Native release build and execution passed | One guest/triple; not every NetBSD port |
| Networking | Mio 1.2.4 / libc 0.2.191 | TCP and Unix-stream readiness, second ready peer while another stalled, bounded command input passed | Loopback IPv4 and Unix only; no TLS/DNS/IPv6/mesh or write-pressure claim |
| SQLite binding | rusqlite 0.40.1 / libsqlite3-sys 0.38.2 | Native FFI build, bound parameters, FK rejection, FTS5, WAL/FULL, busy error, approval, backup and record round trip passed | Trusted fixtures; not power-loss, full disk, corrupt import or final schema |
| Bundled DB | SQLite 3.53.2 | Same oracles passed; no dynamic SQLite dependency in ldd | Embedded C/FFI must be tracked and rebuilt for updates |
| pkgsrc DB | sqlite3 3.53.2 | Same oracles passed after explicit runtime path; ldd selected /usr/pkg/lib | Package/runtime path and update policy still need appliance design |
| Base SSH gateway | OpenSSH 10.3 NetBSD 20260408-hpn13v14-lpk; OpenSSL 3.5.7 | Nine negative/positive gateway oracles passed | Synthetic single account; no complete auth/registration, 250-session or security audit claim |

Mio's pinned source selects its kqueue backend for NetBSD. Its runtime probe
passed; no polling scan, Tokio, async runtime, separate network service or JS
dependency was needed for this readiness adapter.

The initial non-bundled binary silently loaded **base SQLite 3.45.1**, even though
pkgsrc headers/link search paths were set. Its small selftest passed, but that did
not verify the intended runtime. An explicit root-owned pkgsrc rpath corrected
selection; both `ldd` and runtime `sqlite_version()` then reported the intended
library/version. Reject ambiguous library selection in the next appliance slice.
Pinned rusqlite 0.40.1's README describes default 3.34.1 bindings; do not confuse
today's moving upstream README with this pinned version or infer a vulnerability
from the base version string alone. NetBSD backports require separate review.

Stripped probe binaries were 1855752 bytes bundled and 464304 bytes pkgsrc-linked.
This comparison excludes shared libraries, OpenSSH processes and all BBS features;
it is not RAM or whole-install footprint. The 100-ms SQLite busy setting returned
DatabaseBusy after about 220–230 ms in this VM. We asserted a broad <2-second
compatibility bound, not an exact 100-ms latency promise; scheduling/storage and
SQLite retry behavior need measurement under representative load.

## SSH protection goals and boundaries

An attacker may possess a transport key while unapproved/revoked, send arbitrary
exec/subsystem/forwarding requests or oversized input, or stay connected through
approval changes. The fixture allows key authentication to a forced synthetic
program and PTY, rejects original exec/subsystem commands, disables forwarding,
X11, tunnels and user rc files, and rechecks a local DB flag for each READ/POST
transaction. The member key also uses `restrict,pty`; the results test these
controls together, not each independently. System administrator compromise and
arbitrary code execution in the probe are outside this scoped oracle.

Observed nine oracles: pending read/post denied; unknown key denied; host exec
rejected without executing the supplied sentinel; SFTP returned failure/no protocol
output; direct TCP, remote TCP and remote Unix forwards rejected; oversized input
disconnected; one live PTY allowed read/post after approval then denied both after
revocation. QUIT closed the test session. SSH shell-channel requests deliberately
enter the forced probe, never an interactive host shell. No registration flow,
password authentication, reconnect token, resize/ANSI behavior, full terminal
fuzzing or prompt idle timeout was implemented. Agent/X11/tunnel denials were
checked in effective settings, not separately exercised protocol attacks.

OpenSSH authenticates one test OS account; Faction member identity mapping remains
open. The fixture's administrative approval subcommand is unreachable through its
member command parser, but the DB is owned by the probe process. This is no proof
of role isolation after code execution. No claim of a sandbox/chroot or general
host-file confidentiality follows from ForceCommand. The production design needs
an authorization boundary, bounded queues/work, file/terminal safety, connection
limits, listener/firewall policy and cleanup under hostile disconnects.

No public HTTP dependency was needed. Loopback forwarding into a private VM does
not test an overlay invitation workflow, a private-interface firewall or
accidental-public-listener prevention. Those remain required before deployment.

## Embedded SSH alternative and dependency screen

Base OpenSSH provides a practical **comparison gateway**, with crypto and updates
handled by NetBSD, but its per-session process costs and OS-account mapping may
conflict with tiny total budgets. Do not adopt it for 250 members on this test.

The exact russh 0.64.1 published manifest was inspected: Apache-2.0, Rust >=1.89,
Tokio/futures, default flate2/aws-lc-rs/RSA, several crypto prerelease pins, and
getrandom's `wasm_js` feature. That feature does not mean JS executes on NetBSD;
it requires a targeted dependency/source-policy review before any adoption under
our strict no-JS rule. No embedded-SSH build, complete transitive audit or memory
measurement was performed. It remains an unverified comparator, not a fallback
we can claim works. Do not add its defaults blindly to the small Mio prototype.

The compiled NetBSD probe graph has 14 registry crates, with exact declared
licenses and a no-JS/TS filename/manifest scan in the saved inventory. Source
SHA-256 ties observations to the experiment; Cargo.lock pins registry checksums.
Mio/rusqlite/libsqlite3-sys declare MIT; other selected crates use MIT/Apache-2.0
alternatives. Bundled SQLite is public domain. OS/libc/C/SQLite and dependency
unsafe boundaries remain; `forbid(unsafe_code)` only covers our probe. No security
audit/CVE clearance is claimed. Conditional Windows/WASI lock entries were not
compiled or policy-audited; this is a NetBSD-selected graph, not every target.

## Decision, confidence and next falsifiable work

Proceed with Rust on **NetBSD 11 ARM64 as first validation target**, per owner
decision. Mio plus rusqlite/bundled SQLite is a workable dependency slice.
Use base OpenSSH as the measured restricted-gateway comparator; production SSH
backend selection remains open until whole-session resource/identity tests pass.
Keep domain behavior separate from adapters and support scope per triple/device.

High confidence that these pinned components compile/run together in this VM;
moderate confidence in this narrow gateway behavior; low confidence in unmeasured
hardware, complete security, durability and 100–250-session application capacity.
Reversal cost is low while these remain disposable experiments. SQLite bundling
versus pkgsrc changes update/linking obligations without requiring a new data model.

Next #11/#20/#35 should define a useful local read/post/chat slice with real local
identity/approval/revocation. #23 must compare whole OpenSSH session costs versus an
audited embedded candidate, including 100/250 connected clients, slow readers,
handshake limits, identity mapping, default private binding and cleanup. #18
must name a real SBC/Pi model and test toolchain availability, minimal image,
installation/update/restore and total RAM/CPU. #31 must define dependency updates,
advisory review and policy enforcement beyond the selected-source scan. #9 still needs native crash/full
disk/read-only/media tests and versioned export/search/attachment reconciliation.
Do not close those broader issues from this compatibility result.

## Primary sources (retrieved 2026-10-19)

- [NetBSD 11.0 release](https://www.netbsd.org/releases/formal-11/NetBSD-11.0.html),
  [official disk](https://cdn.netbsd.org/pub/NetBSD/NetBSD-11.0/evbarm-aarch64/binary/gzimg/),
  [kernel](https://cdn.netbsd.org/pub/NetBSD/NetBSD-11.0/evbarm-aarch64/binary/kernel/),
  [signed hashes](https://cdn.NetBSD.org/pub/NetBSD/security/hashes/NetBSD-11.0_hashes.asc),
  [official signing key](https://cdn.NetBSD.org/pub/NetBSD/security/PGP/security-officer@netbsd.org.asc).
- [NetBSD QEMU ARM guide](https://wiki.netbsd.org/ports/evbarm/qemu_arm/),
  [pkg_add](https://man.netbsd.org/pkg_add.1),
  [versioned pkgsrc package directory](https://cdn.netbsd.org/pub/pkgsrc/packages/NetBSD/aarch64/11.0_2026Q2/All/).
- [Rust NetBSD target policy](https://doc.rust-lang.org/rustc/platform-support/netbsd.html),
  [Mio 1.2.4](https://docs.rs/mio/1.2.4/mio/),
  [rusqlite 0.40.1](https://docs.rs/rusqlite/0.40.1/rusqlite/).
- [NetBSD sshd_config](https://man.netbsd.org/sshd_config.5),
  [OpenSSH licenses](https://www.openssh.com/openbsd.html),
  [russh 0.64.1 source manifest](https://docs.rs/crate/russh/0.64.1/source/Cargo.toml).
- [SQLite WAL](https://sqlite.org/wal.html), [SQLite public-domain status](https://sqlite.org/copyright.html).

Published Cargo manifests/locked crate contents supplied the exact versioned
dependency/license and backend-selection evidence; moving documentation is not a
substitute for this run's native runtime observations.

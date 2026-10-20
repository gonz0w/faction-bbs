# NetBSD dependency and restricted-gateway probe

Issue #23, with #4/#9/#18/#31. Disposable compatibility research, not a BBS,
production SSH configuration, hardened appliance, or resource benchmark.
See the [report](../../dev-docs/research/0023-netbsd-dependencies.md) and
[saved observations](../../dev-docs/research/results/023-netbsd-dependencies.json).

## Scope and safety

The saved run used NetBSD 11.0 ARM64, QEMU 11.1.1/HVF, two vCPUs, 1024 MiB RAM,
a private copy-on-write disk, user-mode networking, and a temporary host SSH
forward bound explicitly to loopback. No bridge, host directory share, public
listener, HTTP service or real member data was used. The image auto-expands its
root partition; allocate 12 GiB of virtual disk for compiler/build space. This is
not a minimal image. The stock image starts services unrelated to Faction.

Download the report's exact official image/kernel, verify SHA-512 against the
signed release-wide manifest, and verify that manifest with the official release
signing key. The saved key was retrieved through official HTTPS; independent
identity verification was not performed. A nearby image-directory `SHA512` file
did not include the chosen disk image; do not treat that as verified coverage.

Direct QEMU kernel boot used the decompressed GENERIC64 `.img`, virtio block,
network and RNG devices, `root=NAME=netbsd-root -c`, `-smp 2 -m 1024`, no display,
and a private serial socket. On this HVF host the initial boot panicked in the
VMware probe. At kernel userconf, `disable vmt` then `quit` allowed boot. This
workaround is an observed VM-specific limitation, not advice for physical Pis.
The guest auto-reboots after first disk expansion; repeat userconf then if needed.
See the official [QEMU ARM guide](https://wiki.netbsd.org/ports/evbarm/qemu_arm/).

Use temporary keys, pin the guest host key in a private known-hosts file, and
disable password authentication for the test member. Never commit keys, disk
images, connection details, raw SSH logs or a live board's data. Stop the guest
and QEMU, remove forwards and delete credentials when finished.

## Native build and local selftest

The official NetBSD aarch64 `11.0_2026Q2/All` repository supplied
`rust-bin-1.96.0` and `sqlite3-3.53.2` through `pkg_add`. Record exact package
versions when repeating. The saved source graph is pinned by Cargo.lock.
The selected NetBSD crate sources were scanned for JS/TS before use.

For an offline guest build, run `cargo vendor --locked` on this manifest into a
temporary bundle, add the Cargo source replacement config that command emits,
and transfer the source/lock/vendor bundle through the private SSH channel.
Vendor sources and compiled targets stay outside the repository. No Node tooling.

Inside the NetBSD guest, with `/usr/pkg/bin` on PATH:

```sh
cargo build --release --frozen
./target/release/faction-netbsd-probe selftest /tmp/faction-fresh-bundled
```

Use a fresh directory per run. Assertions fail the process on a violated oracle.
The default `bundled-sqlite` feature compiles SQLite 3.53.2 from libsqlite3-sys;
it needs the native C compiler, but no runtime SQLite shared library. Capture the
binary before switching feature sets.

The pkgsrc alternative was built with:

```sh
export SQLITE3_LIB_DIR=/usr/pkg/lib
export SQLITE3_INCLUDE_DIR=/usr/pkg/include
export RUSTFLAGS='-C link-arg=-Wl,-rpath,/usr/pkg/lib'
cargo build --release --frozen --no-default-features
ldd ./target/release/faction-netbsd-probe
./target/release/faction-netbsd-probe selftest /tmp/faction-fresh-pkgsrc
```

Check both `ldd` and the printed runtime SQLite version. Omitting the runtime path
selected `/usr/lib/libsqlite3.so` (3.45.1), despite the pkgsrc compile/link search
path. The explicit root-owned pkgsrc runtime path selected 3.53.2. This fixture
does not prescribe a distributable binary's final linking/update policy.

## Disposable gateway fixture

Create a synthetic OS account `bbstest` with a normal shell solely for OpenSSH's
forced-command dispatch. Install an ephemeral public key in its authorized_keys
using `restrict,pty`; ensure permissions/ownership meet OpenSSH requirements.
Install the **bundled** probe root-owned at
`/usr/local/libexec/faction-netbsd-probe`, not under a member-writable home.
Create `/var/db/faction-probe` owned by the synthetic account, and initialize
`gateway.db` there using the probe's `init` command as that account.

Append [gateway-match.conf](gateway-match.conf) after the global daemon settings.
Validate with `/usr/sbin/sshd -t`, inspect `/usr/sbin/sshd -T -C ...`, then reload
the test daemon. Keep the separately keyed root administrative control path
available only inside the isolated fixture. The fragment does not set a listener,
firewall, host key, connection/rate limits, account mapping or recovery policy.
It is deliberately insufficient as a production configuration.

Run the host driver with private connection arguments (shell variables below are
provided by the operator, not deployment defaults):

```sh
python3 check_gateway.py --host "$GUEST_HOST" --port "$GUEST_PORT" \
  --identity "$TEST_KEY" --known-hosts "$TEST_KNOWN_HOSTS" --output "$RESULT_FILE"
```

The driver requires root administrative access to mutate the synthetic approval
flag out of band. It checks key authentication, pending denial, unknown-key
denial, rejected exec/SFTP/direct TCP/remote TCP/remote Unix forwards, oversized
input disconnect, and one live PTY across approve/read/post/revoke/read/post.
Outputs include oracle names only; preserve unexpected failures privately for
investigation, never publish raw credentials/transcripts. SFTP's observed denial
returned status 1 with no protocol output, without a diagnostic message.

`approval` is an **administrative test mode**, never an allowed member command.
The forced server accepts only fixed READ/POST/QUIT strings; it never executes
client commands. The single synthetic identity, database and approved flag are
not account registration, authentication mapping, role separation or the final
domain schema. All actions use a write transaction for a simple serialized
revocation oracle; this is not an efficient production read architecture.
Backup and binary record round trips use trusted synthetic data. The export is
neither the proposed versioned product export nor a safe arbitrary import format.

## Selected source policy screen

`python3 audit_sources.py` resolves this locked manifest for NetBSD ARM64, inventories
selected crate versions/licenses and fails on JS/TS files, JS manifests or
node_modules. It needs Cargo plus crate sources and may fetch missing metadata.
It does not scan the compiler distribution, all OS packages, generated code,
vulnerability advisories or conditional targets. Compare its JSON output with the
saved dependency inventory before refreshing observations.

## Saved evidence check

```sh
python3 verify_evidence.py
```

This verifies hashes, locked versions, result shape and expected observed passes.
It does not start a VM, rebuild dependencies, rerun SSH, establish security or
measure capacity. Run the native/SSH probes deliberately to obtain new evidence.

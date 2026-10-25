# Native XMPP transport software observations

Issue [#208](https://github.com/gonz0w/faction-bbs/issues/208), 2026-10-25.
[Saved results](results.json) qualify the standalone
[research profile](../../native-xmpp-transport.md), not an integrated mailbox.
No product dependency, provider account, public listener, full OMEMO wire codec,
full-XMPP conformance, audit, physical SBC or capacity claim follows.

```sh
python3 tools/check_xmpp_transport_evidence.py
```

This checks immutable recorded results/source bytes. It does not run a client,
TLS fixture or VM. The [comparison](../../research/0055-native-xmpp-transport.md)
and [ADR 0079](../../decisions/0079-native-xmpp-research-transport.md) explain the
choice and costs; [#209](https://github.com/gonz0w/faction-bbs/issues/209) owns the
first deliberately adopted home-server adapter.

## Actual final runs

Both host Darwin ARM64 and native NetBSD 11 evbarm/aarch64 passed:

- 17 Rust 1.96 unit/AsyncRead/process controls, with no ignored tests.
- 18 real loopback TLS/XMPP controls: verified identity and account authentication,
  opaque synthetic payload exchange, fragmentation/coalesced SASL restart, wrong
  hostname/CA/changed pin, rejected account/mechanism, malformed/deep/large XML,
  whitespace/event/interval bounds, lost connection, trickling timeout and cancellation.
- Exact fixed pin/CA/name checks occur before XML/SASL credentials. Failure output
  contains only enum/counters, not wire data or private fixture content.

The macOS native backend observed TLS 1.2 only. NetBSD OpenSSL observed TLS 1.2
and TLS 1.3. A forced TLS 1.2 positive case runs on each route. The guard validates
actual per-platform observations and the qualified union; it never invents macOS
TLS 1.3 support. A minimal independent Python TLS peer authenticates the generated
account and exchanges stanzas. This is not maintained-home-server interoperability
or an encrypted-mail proof: the opaque fixture payload lacks the complete OMEMO
header/key/SCE semantics. Existing crypto qualification stays separate.

The final runtime VM had two cores, 1024 MiB configured RAM and 1054990336 observed
RAM bytes, Rust 1.96, base OpenSSL 3.5.7 and GCC 12.5. The final guard was rebuilt
incrementally there using dependencies compiled in the separate larger-builder
phase. The native process artifact/hash and linked platform libraries are recorded.

## Compiler cost is separate

A complete fresh release build in the 1 GiB guest failed with rustc SIGKILL at two
jobs and again serially. UVM out-of-swap diagnostics were captured for the two-job
phase; fresh serial dmesg was not captured before reboot, so the record does not
invent serial kernel confirmation. No swap, optimization, allocator, kernel or
application limit was changed to hide these failures.

The unchanged dependency code built successfully using one job in a temporary
2 GiB NetBSD builder; its 16-test pre-review suite passed. The small reviewed guard
correction then rebuilt and passed all 17 tests and the full transport/resource
matrix after restoring the VM to 1 GiB. This demonstrates an incremental/runtime
result, **not** a successful complete fresh 1 GiB source build.

One complete final native research-client process, including TLS/authentication and
one synthetic payload exchange, peaked at **9992 KiB RSS** according to NetBSD
`time -l`. The observed wall time was 0.44 seconds; reported user/system times
rounded to 0.00 seconds and do not imply zero CPU cost. The fixture server,
certificate generation, compiler, integrated vault/UI, BBS/home service and any
multi-user load are excluded. RSS does not apportion shared physical pages or
predict integrated capacity. [NetBSD documents the RSS unit](https://man.netbsd.org/getrusage.2).

## Source and failed-candidate preservation

Twelve own crate/fixture/native-runner inputs were SHA-matched to the actual guest.
Thirty-five preserved intermediate source files include actual red/pre-refactor/old
guard inputs, plus their [stage ledger](../sources/208-xmpp/frozen-inputs.json). The selected
Linux/NetBSD/macOS dependency graph is checked separately against locked archive
checksums and extracted source, complete selected licenses/features and no-JS policy.
No upstream CI or unselected JavaScript source was adopted or executed.

The final review added a checked immutable pin for the retained rejected observation.
The [original record](initial-record.json) is preserved byte-for-byte; the correction
guard permits only that additional observation digest and provenance field. No
runtime/source hash or measured outcome changed.

The [intermediate observation](intermediate-observations.json) preserves the earlier
16-test/18-control native result and its original cost before review found the
whitespace-cap defect. It is explicitly rejected as final qualification. Its old
bytes/results are not refreshed to the corrected guard. All preceding project
checkpoint/interoperability/capacity measurements remain unchanged.

Actual tests failed before fixes for missing guard/driver modules, non-UTF8 argv,
coalesced unconsumed-byte replay, typed connection loss and a 16385-byte stanza
accepted after root whitespace. The cap fix counts whitespace conservatively;
a positive exact combined-budget case remains. A whitespace TLS flood can hit the
16 KiB interval before the 64 KiB lifetime cap; the separate Rust stanza-boundary/
restart test proves the lifetime budget. No constant or negative assertion was
relaxed.

Initial native setup also retained two distinct fixture failures: an incomplete
directory-vendor subset could not resolve an unselected target manifest, and base
OpenSSL lacked a global request config. Verified Cargo source/index cache enabled
offline resolution without downloading unselected JS source. Certificate requests
now use an explicit private fixture config, with a failing regression before the
fix. No host OpenSSL file or trust store was installed/changed.

The executed [builder](run-build.sh) and [runtime](run-runtime.sh) scripts are
archived laboratory inputs, not services or generic setup tools. The builder's
historical unused failed-directory branch contains `exit99`; its actual directory
change succeeded. Preserve those bytes. Use the copyable commands below rather
than treating that archived branch as a tested failure path.

## Reproduce

Use the repository root, Rust 1.96, a native compiler, OpenSSL 3 development files
on NetBSD/Linux and Python 3. Set base OpenSSL paths on NetBSD. Use an adequately
sized builder; the tested complete builder had 2 GiB.

```sh
OPENSSL_INCLUDE_DIR=/usr/include OPENSSL_LIB_DIR=/usr/lib \
  cargo +1.96.0 test --manifest-path experiments/208-xmpp-transport/Cargo.toml \
  --all-targets --release --locked
OPENSSL_INCLUDE_DIR=/usr/include OPENSSL_LIB_DIR=/usr/lib \
  cargo +1.96.0 build --manifest-path experiments/208-xmpp-transport/Cargo.toml \
  --release --locked
python3 tools/probe_xmpp_transport.py \
  --binary experiments/208-xmpp-transport/target/release/faction-xmpp-probe
```

On the qualified NetBSD image the executable Python name is `python3.13`; use that
name if `python3` is absent. Other native build hosts select their normal TLS
backend and omit the NetBSD OpenSSL assignments. The binary opens only a generated
loopback peer; no real account/provider is required. The resource collector is
NetBSD-only and strictly parses `time -l` diagnostics; it rejects unknown stderr.
The ten-second timer is cooperative after trusted stdin setup. Only generated
regular CA files are qualified; arbitrary blocking local paths/OS-provider work
are not a hard real-time guarantee.

## Teardown

The actual owned QEMU PID/overlay/controller were verified, shut down and removed;
its actual loopback administrative forward was verified closed. TLS fixture
listeners/children stopped, generated keys/accounts/temp roots were removed, and
analyst/implementer/reviewer roots were removed after source capture. A separately
verified public Cargo source/index cache is retained for checks. The saved owner
playground stayed stopped: saved-file hashes and disk size/mtime matched before
and after. No public listener, real provider contact, deployment or host-policy
change occurred.

# Reproducing the bounded software checks

Use the exact [input map](inputs.json), source objects and existing locked archives.
The saved-data unit guard validates retained hashes; it does not compile or rerun
NetBSD. Original raw captures, executed binaries and failed candidates are retained
privately, separately from this public record.

## Source and dependency qualification

The original and refreshed archives are identified separately. The refreshed
committed archive changed only `crates/faction-provider/tests/embedding.rs` in the
346-input selection. Match all input hashes before compiling. Preserve any new
failure and its actual source/binary before changing a producer or observer.

Use Rust/Cargo 1.96.0 and existing OpenSSL 3 development files. Set Cargo's target
outside the source tree, `CARGO_BUILD_JOBS=2` and `CARGO_NET_OFFLINE=true`. On the
host select the existing OpenSSL 3 include/library paths explicitly, with
`OPENSSL_NO_VENDOR=1`; do not use a different default installation or vendor one.
Run the provider, network-mail and historical provider-auth source guards before
compilation. Their inventories cover Linux, NetBSD and macOS graphs, complete
selected sources/licenses/checksums and the no-JS rule. The provider's default and
`blob-iq` profiles are distinct; no new registry version was introduced.

## Host and native execution

The actual host commands, selected environment, test names and copied executed
binaries are in [host.json](host.json). Run provider tests without and with
`--features blob-iq`, including compile-fail docs, plus formatting and all-target
Clippy with warnings denied. Existing codec, transfer, file-store and default
client regressions are retained separately in the historical host record.

For native work, preflight an existing clean standalone owned software clone,
existing hypervisor/compiler/OpenSSL and qualified offline cache. Compare saved
owner assets and confirm their images are closed. Boot only the marked disposable
clone with two CPUs, restricted networking and one fresh loopback admin forward;
stop inherited BBS/file workers inside that clone. Do not expose BBS/provider
ports, install packages, change the host or start the saved playground.

Build the explicit provider default/IQ profiles, standalone protocol, network and
default clients and file store with two CPUs/2 GiB and at most two compiler jobs.
Retain original built artifacts before restaging revised inputs. Restart the same
clone at two CPUs/1 GiB; verify selected source, binary, library and cache hashes
again before actual execution. Run provider tests both serially and with default
parallelism, compile-fail docs per profile, protocol/client/store all-target tests.
Do not invoke ignored crash helpers directly; their parent tests supply the fixture
and recovery protocol. [Native results](native.json) contain actual commands,
per-stage costs and helper distinctions. No new terminal/UI behavior is claimed.

Run fixed bounded controls for actual mTLS acceptance/refusal, binary request/reply,
current policy/lifecycle/route checks, malformed/oversized input, attempt/deadline/
cancellation bounds, quota, exact retry and recovery. The IQ proof is typed local
composition after real TLS; it is not ordinary native-client XMPP connectivity.
The two deliberate source mutants test assertion adequacy only and are never
installed as runtime code. Keep unsuccessful setup/zero-selection trials distinct
from actual product failures and passing tests.

Capture final actual binaries, library/tool identities, logs and failure variants;
verify no owned stores/test processes/non-admin listeners remain. Stop/reap the VM
and controller, confirm the forward and saved images are closed, compare owner
assets, verify private capture retention, then remove only the marked owned roots.
Record facts and limits separately from production enrollment, deployment or
independent interoperability. Finish separate nonauthor review, committed-source
checks and required current-head CI before merging the paired closing PR.

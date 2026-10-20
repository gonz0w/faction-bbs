# 0012: Restricted OpenSSH with a separate terminal UID and local worker handoff

Status: implemented/tested development gateway; production gates remain
Date: 2026-10-20
Owner: project owner requested restricted SSH and native NetBSD application validation
Issues: [#23](https://github.com/gonz0w/faction-bbs/issues/23), #18/#19/#20/#30/#31/#35
Supersedes: no prior product decision; implements ADR 0004/0006 direction
Evidence: [native report](../research/0024-restricted-ssh-netbsd.md), [plan 0012](../plans/0012-restricted-ssh-netbsd.md)

## Question, scenarios and stopping condition

Provide ordinary SSH access to the current BBS while keeping member OS access out
of the private database and owner recovery/export entry point. Test two real remote
members, approval/revocation, ANSI/resize/chat and attempted host access in a fresh
NetBSD VM. Stop at a bounded, reviewable development gateway with current user docs
and native evidence. No public deployment, 250-member acceptance or physical SBC
claim follows. Preserve local data, no-JS/TS, optional private networking/HTTP absence
and Rust/primary NetBSD choices.

## Gate screen and alternatives

| Alternative | Approval / local data / owner separation | Dependency and portability | Resource / operations | Decision |
| --- | --- | --- | --- | --- |
| Force the owner console under the SSH member UID | Fails: recovery/export/state authority mixed with member entry | Already runs | Easy but wrong authority | Reject |
| Separate member binary with direct DB rights under the terminal UID | Parser separation helps, filesystem owner rights remain shared with frontend | Smallest source change | Lower IPC work, weak OS separation | Reject for this increment |
| Local domain RPC broker, unprivileged presentation client | Credible distinct authority | Needs typed/versioned operations and client abstraction for all room/chat/UI behavior | Good future shared core candidate; larger current change | Defer comparison for scale |
| Embedded SSH server in Rust | Credible if current auth/output controls validated | Earlier russh manifest review left runtime/crypto/wasm_js source gates unresolved | Whole-session footprint unknown | Keep comparator, do not add defaults |
| Base OpenSSH -> separate terminal UID -> protected Unix terminal handoff -> trusted member worker | Current BBS auth/approval remains local; no DB/owner command path in frontend | Existing OS SSH, rustix fs/net features; narrow native ABI bridge needed on NetBSD | Process per member plus SSH; fixed eight-worker development cap, native tests | Implement current gateway |

Mandatory privacy/local authority/provenance/no-JS gates pass the scoped tests and
source screen. Complete hostile-code isolation, crash/output enforcement and
small-host 100/250 capacity remain unknown, so this is not production adoption.
Qualitative criteria: minimal remote parser, explicit authority separation, reuse
of tested UI and measurable native behavior. No unsupported weighted performance
ranking is justified by a two-member process sample.

## Decision and trust boundaries

OpenSSH admission authenticates an operator-installed transport key to `factionterm`.
It grants no BBS identity/approval. Force only `faction-ssh`, with no exec/original
command, forwarding, user rc/environment, SFTP, tunnel or host-shell authority.
The root-controlled account home/key policy and exact dedicated daemon matter;
another permissive SSH daemon admitting that account invalidates the boundary.

The frontend has no core/SQLite/password dependency. It passes one already-open
TTY plus bounded TERM over a protected local Unix socket and waits for completion.
The backend is a distinct non-root owner UID. Socket group/path checks and kernel
TTY ownership gate local transport; they are not a peer-credential/member proof.
The trusted broker starts a fixed owner-configured member executable with cleared
environment, fixed state/theme arguments and the received terminal as stdio.
No arbitrary client path, command or owner RPC is parsed. BBS password, status,
role and epoch checks still run in the core; opaque sessions stay in the worker.

Extract shared UI into the CLI library and duplicate admitted stdin, so workers do
not require a controlling `/dev/tty`. The owner binary alone parses bootstrap,
recovery/export/restore. Trusted workers retain database-owner authority; this is
not a sandbox for compromised BBS code. Frontends share a terminal UID and can
affect each other after a compromise of that OS identity. Keep these limits explicit.

Default cap eight workers/handoffs, 128-KiB supervisor stacks, burst eight / one new
launch per five seconds, two-second receive deadline, 120-second prompt reads and
30-minute total session. Admission serialization protects the receive-to-spawn
descriptor interval on platforms without atomic CLOEXEC. NetBSD/Linux also request
atomic descriptor CLOEXEC. Disconnect gets a short graceful interval then kill/reap;
normal chat cleanup succeeds where possible, abrupt lease expiry remains bounded.
Thread creation failure drops that connection/slot without panicking the listener.

## Native ABI discovery and implementation exception

The first real NetBSD handoff failed although all Rust code compiled and negative
tests passed. Native headers reported `cmsghdr` size 12, data offset 16 and one-int
space 32. Pinned libc 0.2.191 NetBSD/aarch64 uses int-size `_ALIGNBYTES`, producing
offset 12/space 16. The Rustix ancillary path therefore did not transfer/interpret
the intended descriptor. Negative-only tests were insufficient.

Use a small `faction-unix` crate on NetBSD: an 81-line native C helper calls the
target's CMSG macros; a documented Rust FFI wrapper transfers exactly one owned fd.
Buffers are fixed at 256 bytes, excess/truncated rights are closed, success marks
CLOEXEC, and the wrapper documents pointer/ownership lifetimes. Other application
crates retain `forbid(unsafe_code)`; do not claim all project code is memory-safe
Rust. Existing cc 1.9.0 builds the helper; no new registry package is added.

An independent native positive test proves actual socket data crosses the received
descriptor and the original stays live. Real SSH tests additionally reject 1/2/80
nonterminal/extra rights and show unchanged broker descriptor rows. Keep a dedicated
provenance/regression gate; remove/reduce this bridge only after a pinned upstream
replacement passes the native positive and failure tests. No registry source patch
or fork is silently applied. Broader ABI effects remain outside this finding.

## Evidence, confidence and primary sources

Retrieved 2026-10-20:

- [NetBSD sshd_config(5)](https://man.netbsd.org/sshd_config.5), ForceCommand,
  DisableForwarding, key/environment/TTY/listener and timeout directives; and
  [sshd(8)](https://man.netbsd.org/sshd.8), `restrict,pty` and execution context.
  Configuration semantics are documented; effective settings plus selected wire
  negatives were measured together, not proved individually.
- [NetBSD recvmsg(2)](https://man.netbsd.org/recvmsg.2) and the guest's shipped
  `/usr/include/sys/socket.h`: native ancillary layout and descriptor semantics.
  Pinned libc 0.2.191/rustix 1.1.5 sources inspected directly. The native C layout
  probe and positive transfer test support the narrow observed mismatch.
- [NetBSD QEMU guide](https://wiki.netbsd.org/ports/evbarm/qemu_arm/) and official
  signed NetBSD 11 image/manifest/key sources preserved in the new evidence.
- Native Rust 1.96 full workspace tests/build, four actual local PTY workflows,
  20 actual application SSH oracles and a two-member process sample. Details/hashes
  in the report; CI hash checks tie saved evidence to current relevant sources.

High confidence in named native behavior, moderate confidence in this restricted
gateway's limited threat oracles, low confidence in unmeasured production crash,
load, independent-network/firewall, hardware and hostile-code scenarios. An empty
CLI command was found to select a shell request in the OpenSSH client; the suite
uses a real blank exec request instead. Process/fstat oracles verify a real PID
and live baseline, avoiding false passes on NetBSD formatting/name differences.

## Reversal, costs and next work

Moderate reversal cost: owner/core schema unchanged and shared UI stays reusable;
replace the transport supervisor/handoff with a measured broker/RPC/embedded
alternative later. The private database/snapshot format remains local. The native
bridge adds a small C/FFI review/update obligation; switching to a corrected pinned
upstream implementation requires native tests. No user data migration is needed.

Next measure complete OpenSSH/backend/private/shared memory and hostile failure
behavior before increasing eight-worker cap; implement maintained rc.d lifecycle,
output/broker-crash policies, account/key management and remaining local lifecycle.
Per-account/IP fairness and user-shell/OS confinement require further work. Physical
SBC support remains deferred. Lua menus/isolated doors follow useful core work;
do not treat a prototype boundary as a production game sandbox.

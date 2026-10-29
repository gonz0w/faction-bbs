# Authenticated provider embedding and native IQ integration

Software observations for [#276](https://github.com/gonz0w/faction-bbs/issues/276)
and [#277](https://github.com/gonz0w/faction-bbs/issues/277), 2026-10-29 UTC.
The optional standalone provider library and shared native blob-IQ codec were
qualified on the macOS host and NetBSD 11 evbarm/aarch64 with Rust/Cargo 1.96.0.
The provider remains outside ordinary workspace builds and starts no service.

## Observed behavior

Actual TLS 1.3 device sessions authorize the existing ciphertext store. The public
runtime owns authority; weak controls and sessions cannot retain a dropped owner.
Current typed policy, revocation, reset, stop, certificate validity and routing
checks gate operations. Binary requests and replies also traverse the same verified
TLS connection. The optional IQ adapter composes maintained native codec elements
with that authenticated session **in process**. It implements no XMPP stream,
server, SASL mechanism or IQ wire carrier. Ordinary native PLAIN sessions do not
acquire device authority through this API.

The final host suites passed **12 authority + 17 external + four compile-fail**
controls without the IQ feature and **12 + 26 + four** with it. Formatting and both
Clippy profiles passed. [Host results](host.json) retain actual test names, commands,
log/binary/tool identities and limits. Earlier host observations are separately
retained in [host-before-review.json](host-before-review.json); their weaker route
controls and then-pending native qualification are historical, not final acceptance.

Fresh [NetBSD results](native.json) passed the same authority/external controls
serially and with default parallelism, plus four compile-fail controls per profile.
The standalone protocol suite passed 25 tests, the network client 231, default
client 48 and file store 18. Eight existing client subprocess-helper entries and
one existing store death-helper entry remain ignored as direct harness entries;
the parent recovery tests invoke them. No provider control was skipped. These
per-profile counts overlap and are not a count of unique product behaviors.

Native runtime used two CPUs and 1,054,990,336 observed physical-memory bytes.
Compilation used two CPUs, 2,128,732,160 bytes and at most two compiler jobs.
These are guest envelopes, not peak-process memory or capacity measurements.
[Identities](identities.json) distinguish actual binaries, libraries, loader and
tools. Earlier original-source builds are retained separately and are not relabeled
as final runtime passes. No package was installed or upgraded for this qualification.

## Separate review and actual chronology

The [TDD history](tdd.json) preserves observed missing-runtime, missing-wire-method,
missing-handler, profile, preservation and CI-gate RED/GREEN stages. Initial
development binaries were not all individually retained; exact source/log variants,
identified later binaries and final executed binaries are retained privately.

Separate review found that forged actor-free reads originally queried nonexistent
objects. The added controls commit actual Alice-readable and Bob-only objects,
prove rightful Status/Get reads, then reject forged routes across dispatch, actual
binary TLS I/O and maintained IQs. Both deliberately bad private source variants
passed the three old controls and failed all three strengthened controls: six
actual distinguishing failures. This characterizes unchanged producer behavior;
it is not a claimed producer-fix RED. The zero-test stale-timestamp trial is
explicitly rejected. Frozen mutant source identities are included in the manifest.

An optional post-response-encoding check added after GREEN lacked its own RED and
was rejected before shipment. Its source/history remains retained. The accepted
IQ producer was restored to its previous GREEN behavior, with accepted assertions
intact, before the separate review added stronger coverage. Attempts are checked
before decoding; current authority and the deadline gate actual store dispatch.
Bounded local result construction cannot undo completed authorized store work or
guarantee retroactive suppression of that typed result. Binary transport I/O
retains its deadline/cancellation checks. No owner waiver or retrospective RED is used.

## Exact source and teardown

[Inputs](inputs.json) binds all 346 selected source/manifest/lock/guard/inventory
inputs from commit `a8da3cf553e5a2c7379e0cf4023af3ab56632b82`, including the
explicit test-only delta from `bd120cef`. Broad input identity does not mean every
selected file executed. Frozen bytes reuse existing evidence objects where exact;
new unique objects are retained separately. The manifest pins saved data/source
bytes and selected-record membership. Its unit check verifies those bytes without
rerunning or attesting a VM. [Reproduction](REPRODUCE.md) describes actual layers.

[Teardown](teardown.json) records stopped/reaped disposable VM/controller, closed
forward, removed owned stores/build/clone roots, private retention and unchanged
saved owner assets. All 86 saved owner assets remained unchanged and six images
remained closed. The owner playground stays stopped and intact. No keys, raw
private captures, real endpoints or member data are published.

The one-active-session, eight-attempt, five-second shared admission, ten-second
accepted cycle, 8192-byte binary and 16384-byte IQ bounds remain. Filesystem work
is cooperative and cannot be forcibly preempted. Normal BBS admission/credential/
helper/drain limits and classic/default-off paths remain unchanged. This is not
provider deployment, production enrollment/recovery, own-home delegation, OMEMO
private-identity possession, independent modern-client interoperability, hardware,
a security audit or 250-member support. Those original outcomes remain under
[#88](https://github.com/gonz0w/faction-bbs/issues/88).

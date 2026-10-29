# Durable provider and private operator software evidence

Issues [#283](https://github.com/gonz0w/faction-bbs/issues/283) and
[#284](https://github.com/gonz0w/faction-bbs/issues/284) add explicit restart durability
for preapproved public provider policy and a private foreground operator. The
frozen product source is `57020dcb2c04b62a58596b1e07c2310c6b17701a`.

The durable runtime retains validated registry generation, reset epoch,
availability and all lifetime certificate-to-exact-principal history. Every
policy control mutation validates, publishes and installs under the dispatch mutex.
Invalid proposals preserve the current policy. Storage failure or publication
uncertainty stops that runtime until explicit reopen. Ordinary stop/drop does
not persist a policy revocation. Reset clears all active bindings; explicit
reapproval needs a higher generation and the retained exact principal.

Initialization requires an empty private root. Reopen requires the original
permanent lock and complete current state, without bootstrap or pending fallback.
Owned 0700 directories, regular single-link 0600 files, descriptor-relative
no-follow access and exclusive locking bound the store to current/pending
snapshots of at most 65536 bytes each plus a permanent empty lock. This provides
ordinary restart durability and monotonic proposals against retained validated
state. It does not detect restoring an entire older valid directory or defend
against its owner.

[Host results](host.json) record observed Rust 1.96 default and durable/IQ suites,
six operator process controls, parser/probe checks and the maintained production
console journey. The latter passed 26 controls, thirteen each in classic and
Unicode modes, using a 1 MiB attachment, actual device TLS keys, durable policy,
separate encrypted-mail release/receipt and explicit verified no-overwrite save.
Restart after incomplete upload, exact retry, current revocation and offline lock
refusal were exercised. Possible durable progress is not guaranteed reply loss.

[Fresh NetBSD 11 ARM64 results](native.json) distinguish 27 build/check stages
at two CPUs/about 2 GiB from 32 materialized runtime commands at two CPUs/about
1 GiB. Five empty harnesses are not new controls. The production console passed
the same 26 controls, and separate host/native home-mail outage journeys each
passed eight approved-BBS/restricted-SFTP controls. The first native Python probe
command had three passes and one setup error; only the unchanged errored case was
replayed successfully with a private test-child interpreter alias. This is not a
full four-case rerun. No producer or resource limit changed.

[TDD](tdd.json) preserves genuine API, live-state and parser failures before their
corrections, initial feature/command absence, fixture-only TLS failures and the
actual CI marker consumer failure. Earlier Rust 1.98 variants remain separate.
The interrupted pre-gate Rust 1.96 run remains nonacceptance despite passing
output. Feature-profile counts overlap; the direct `death_child` entry is a
helper whose parent actually terminates five publication-boundary subprocesses.
Disabled operator targets run zero tests. Three early failed operator executable
byte copies are unavailable, and two lack historical executable digests. Their
exact sources and original logs/results survive; later qualified executables are
retained separately. This historical retention gap is not waived by this record.

[Inputs](inputs.json) selects 797 tracked current inputs, broader than execution,
and retains exact development variants anchored to the readable base commit.
Source references reuse existing immutable objects where possible. Private
captures containing ignored metadata or untracked orchestration are excluded.
[Identities](identities.json), [costs](costs.json), [routing](routing.json) and
[teardown](teardown.json) distinguish observed tools/binaries/libraries, combined
build/test or wall-only costs, unverified backend routing and actual cleanup.
No old record, resolver or source object is refreshed. [Offline checks](offline.json)
retain the original 89-command block (88 passes/one failing discovery with six
failures), the scoped intermediate timeout-fixture failures and the separate
584-entry passing run after five Python CI test-fixture corrections. That overlay
is distinct from the frozen native/product source.

The provider stays explicit, default-off and same-home. Certificate possession
and routing approval do not prove OMEMO key possession, enroll real devices,
recover mail keys or grant BBS membership. Ordinary mail/BBS/files remain
independent. Provider limits remain one session, eight attempts, five-second
TCP/TLS admission, ten-second cycle, 8192-byte payload, 16384-byte IQ, 32 policy
entries and eight handles. Each SDK call retains four chunks and its ten-second upload bound including
mail release; the foreground console retains 65 calls/60 seconds. Normal budgets remain eight
sessions, five-second admission, two credential permits/kernel slots, three
helpers and five-second drain. No deployment, independent
interoperability, physical hardware, security audit or 250-user capacity claim
follows. See [reproduction](REPRODUCE.md).

The owned native services, VM/controller and children stopped; three forwards
closed, four guest roots and the disposable clone/run root were removed after
verified capture. All 86 saved owner assets matched and six saved images stayed
closed. The temporary host execution root is absent after atomic relocation;
private originals and build cache are intentionally retained. Separate host/native
available-value scans found no matches in the proposed package. Coverage excludes
successful-fixture values already deleted and secrets never retained in memory.

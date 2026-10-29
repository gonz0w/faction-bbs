# Offline provider preparation software evidence

Issues [#286](https://github.com/gonz0w/faction-bbs/issues/286) and
[#287](https://github.com/gonz0w/faction-bbs/issues/287) add deliberate local
preparation from an existing usable member checkpoint and independently approved
operator data. Final source is `358e35ceec242763d8ea2442a1cfc2cfc390cb7c`. Its only change
from frozen `4973578362c2429ef28f81e5af5eb97888717889` is a separately reviewed
unsafe-directory test-fixture correction; runtime producers remain identical.
The baseline is merged `cfa7c4a782b865fea00687ef23d0f0cab7974663` from PR #285.

The member's separate commands prepare the existing private version1 provider
profile and export an inert version1 public device card. Account, device and full
public identity come from the actual unlocked checkpoint. The canonical existing
publication gate rejects history-only, expired or revoked state and imports the
actual device snapshot. Normal hidden unlock, cancellation and terminal cleanup
are reused. An existing private lock is required before opening the checkpoint;
preparation creates no checkpoint or spool member and starts no connection.

The card has exactly version, account, device, full public identity and SHA-256 of
the explicitly selected public certificate's DER. The bounded parser refuses
unknown, duplicate, malformed and noncanonical values. The read-only digest helper
accepts one unambiguous public certificate without opening a private key. A card
or certificate is self-asserted data, not identity-key possession, membership,
enrollment, independent approval or an adopted network protocol.

The operator's separate offline command compares that untrusted card and selected
public certificate with a **complete independently preapproved proposal**. It
preserves all bindings, flags, generation and reset epoch in canonical checked
output. It never infers approval, enables a device, exports current authority or
opens retained policy/blob state. Existing separate `init` or `apply` remains the
state-changing authority boundary. A stale prepared candidate can still fail apply.

Each output is an explicit independent publication: owned private no-follow paths,
regular single-link 0600 completed files under owned 0700 directories, bounded
writes, no replacement and atomic final-name publication. A failure after attempting
publication may leave the complete output and reports uncertainty; it does not
promise rollback, a two-file transaction, erasure or power-cut durability. The
member profile reloads through the maintained loader against the same vault.
Local credential consistency checks parse the existing material and verify the
actual certificate/key pair. Remote chain, purpose, transport-certificate time and
current provider-authority checks still occur at their existing TLS/operation
boundaries. Successful preparation establishes no remote readiness or approval.

[Host results](host.json) preserve actual API/command RED and GREEN, adverse identity,
credential/path controls, seven publisher failure phases and the maintained
classic/Unicode production journey. The prepared journey passed 32 controls,
16 per display mode, using newly generated profile/card/candidate files, a
separate explicit init/apply, a 1 MiB attachment with actual ciphertext upload, encrypted mail
receipt, authenticated fetch/verify, private save and no overwrite. Fresh legacy
mode passed 26 controls without selecting the preparation flow. Counts overlap
across feature suites and helper entries are not independent tests. The original
host results remain distinct from the later umask077 fixture invariant RED and
corrected focused/full 12 host CLI GREEN; no product behavior or refusal assertion
changed.

[Fresh NetBSD results](native.json) record the same production bytes on NetBSD 11
ARM64, with two CPUs/1054990336 memory bytes for materialized test/runtime work.
Compilation used two CPUs/2128732160 bytes and at most two compiler jobs; the
separate test-only correction used one job. Actual prepared and legacy production
journeys passed 32 and 26 controls respectively. Fresh ordinary home-mail/BBS/
restricted-SFTP outage controls passed eight native-client and eight host-client
checks against the same owned guest, with distinct synthetic members.

The original native operator command remains recorded as 11 passes/one fixture
failure: its inherited private umask made an intended 0755 directory actually 0700,
which the product correctly accepted. The reviewed test-only correction explicitly
sets 0755, checks the actual mode and preserves the refusal assertion. Only the
affected native case was replayed and passed; the earlier eleven passes were not
relabelled as a rerun. Original execution stdout/per-role logs remain retained,
but an overwritten runtime metadata alias is unavailable. Hash-verified derived
indexes describe those logs and do not replace the original metadata bytes.

[Source qualification](source-qualification.json) separates the original 27-graph
Rust 1.96 source/license/no-JS gate from supplemental prepared-mode CI qualification.
All 167 existing registry versions/checksums remained unchanged. Ordinary default,
native and network client graphs exclude preparation, operator and durability.
The device-provider client selects preparation without the operator/durable feature.

[Exact inputs](inputs.json) resolve the broad 810-input source/config/inventory
selection and bounded measured development variants through immutable public source
objects. Membership does not assert every file executed. The two readable Git
anchors supply supporting context; private whole-repository captures, raw commands,
logs, credentials, endpoints, member values and executable bytes are not published.
Previous provider records and source objects remain unchanged.

[Executable identities](identities.json), [TDD chronology](tdd.json),
[measured costs](costs.json), [privacy scope](privacy.json),
[actual cleanup and private retention](teardown.json), and [routing limits](routing.json) distinguish observed
software results from configured roles, compiler work and whole-harness costs.
The host pipeline RSS includes synthetic peers, clients, operator and harness,
not product-only memory. Runtime model/effort telemetry was unavailable.

Use the [normalized reproduction recipes](REPRODUCE.md) and the maintained
[member preparation guide](../../../docs/provider-preparation.md) and
[operator guide](../../../docs/provider-operator.md). Normal service/provider/SDK
and console bounds remain unchanged. This record does not establish physical
hardware, capacity, production deployment, certificate issuance, real enrollment,
own-home delegation or independent modern-client interoperability. Parent #88,
hardening #106–#111 and deferred advanced/game work retain their separate scope.

The owned native VM/controller stopped normally, three former forwards closed and
four marked guest roots were removed. All 86 saved owner assets stayed exact and
six owner images stayed closed; the playground was never started. The host evidence
root moved atomically into ignored private retention. Original failed-fixture and
live roots are absent while original captures, binaries, corpora and build caches
remain private; no data-erasure claim follows. Available-value scans distinguish
raw public/static matches from classified sensitive matches and expressly exclude
uncaptured or previously deleted in-memory values.

# Deliberate same-home provider enrollment

Issues [#293](https://github.com/gonz0w/faction-bbs/issues/293) and
[#292](https://github.com/gonz0w/faction-bbs/issues/292), observed 2026-10-30.
The [member/operator guide](../../../docs/provider-enrollment.md) combines the
maintained native client, offline operator and dedicated SSH forward. The new
[driver](../../../tools/probe_enrolled_provider.py) qualifies those existing
interfaces; it is not a new enrollment authority or online issuer.

## Observed software controls

| Current corrected source | macOS 27.2 arm64 | NetBSD 11.0 evbarm/aarch64 VM |
| --- | --- | --- |
| Enrollment/transfer, classic and Unicode | 40 controls; 34.08 seconds | 40 controls; 89.16 seconds |
| Focused provider Python contracts | 40 pass | 40 pass |
| Same-picker unavailable home → approved BBS → restricted SFTP | 8 controls | 8 controls |
| Current-policy Rust runtime/lifecycle controls | 7 individually selected tests pass | Not rerun in this batch |

These counts overlap behavior and are not unique test-suite totals. See
[host](host.json), [native](native.json), [outage](outage.json), and
[source/executable provenance](provenance.json), and [retention/teardown](retention.json). Enrollment uses a 1,048,576-byte
plaintext and actually encrypts, uploads through authenticated SSH and mTLS,
receives encrypted XMPP mail, fetches, verifies and explicitly saves a new private
file. A second save refuses replacement. Public principals from the member seed
are never installed into the provider. Account authentication and full public
identity inspection precede the independent operator record and complete policy
proposal; the actual TLS operation demonstrates transport-key possession.
It does not prove OMEMO private-key possession or correctness of human approval.

The test isolates credential failures under a long-valid policy and first
observes successful authorized progress. Separate generations test expiry,
transport rotation, revocation and reset. Interrupted upload retains the exact
pending ciphertext; restarted operator and SSH owner resolve the persisted retry
before releasing mail. Missing/copied/mismatched local profiles, wrong pin/key,
unapproved certificates and revoked/expired/reset authority leave unauthorized
store/mail bytes unchanged. Operator changes use stop/apply/restart; the seven
separate host Rust tests cover live Runtime policy checks, not an invented online
operator reload.

The dedicated SSH fixture authenticates one current OS user with three distinct
keys: forwarding, BBS-role and SFTP-role. The ordinary-role keys refuse forwarding;
the dedicated key refuses another destination, shell and subsystem channels.
Generic channel refusal alone is insufficient: session-denial checks require
fresh same-attempt server evidence of successful key authentication and
`MaxSessions 0` refusal. **This proves per-key restrictions under one UID, not
separate forwarding-account UID isolation or deployment on two machines.** A
separate owned NetBSD board supplies actual distinct-UID approved BBS/SFTP access
for the outage journey. Its normal eight-session/5000-ms configuration is intact.
No forwarding, credential or capacity limit was enlarged.

The outage helper uses a closed TCP home and real preseeded encrypted history,
cursor and pending state. One picker process survives failed mail operations,
lock/unlock, local board registration and explicit sysop approval, restricted
SFTP upload/review/download, and return to the picker. Mail-state bytes remain
unchanged. The two platforms use fresh different synthetic member registrations.
This complements the enrollment driver's provider-outage/ordinary-mail control;
it is not a federation availability or load test.

## Source, build and failures

The final producer is SHA-256
`5dec0a4a92cd208ed43012fd6032ea72b48897db4328e38adaaf9d3a0fb5c871`;
its 13-test contract file is
`3062d21b4d24a1755e14d5f18254ba3e135c9c0c0ad280f9fe60f202613065cd`.
The initial stage map preserves actual missing-interface RED, subsequent positive
and negative GREEN stages, causal-policy/log-oracle RED and every executed source
variant. The rotation eight-test GREEN was observed in the tool transcript; it
has no separately redirected log. The later CSR map retains an actual failing
missing-`-config` regression against the previously executed producer, followed by
40 passing relevant tests. Map identities are in provenance; private retained
captures resolve their source/test/log entries. No missing log was reconstructed.

The first host full journey passed against producer `7c1a8ad1…`; it is not a pass
for the final source. The first NetBSD runtime then failed at CSR verification:
`openssl req` tried the absent base-image system configuration. The corrected
command explicitly selects the already created private request config. Both full
journeys above were rerun on the corrected source, with unchanged assertions.
The native unit preflight separately lacked the source Cargo manifest and the
unversioned `python3` used by an older mock executable. Supplying the manifest and
a unit-only private interpreter alias yielded 40 passes. No alias or OpenSSL
configuration was injected into the actual production-console run or installed
system-wide.

Host client/operator/fixture builds used Rust/Cargo 1.96.0, two build jobs, locked
offline inputs and explicit Homebrew OpenSSL 3.6.3 include/library paths. The first
host build found LibreSSL and failed the existing OpenSSL-major guard; the guard
was not weakened. The failed build executable survives, but the full initial log
is available only in the tool transcript. OpenSSH is 10.5p1; Python is 3.14.7.
Provider, provider-auth and network-mail dependency preflights passed without new
dependencies or a JS/TS exception.

**No fresh native compilation is claimed.** The retained native ELFs from build
`4973578362c2429ef28f81e5af5eb97888717889` were executed afresh after comparing
810 selected inputs: 808 match, with only two test files different. All 734
selected non-test inputs, including production, build, lock and fixture inputs,
match; the new Python driver is separately identified. Old test executables are
not represented as current native test runs. The two host outage seed fixtures
are also explicitly reused; the host production client was freshly built.
Native OpenSSL is 3.5.7, OpenSSH 10.3 and Python 3.13.14. Executable hashes are
recorded, rather than inferred from tags or version strings. The VM had two CPUs
and 1,054,990,336 bytes of physical memory. This is not a 250-member resource or
physical SBC qualification.

## Privacy coverage and retention

Before deletion, the driver scanned each provider subtree (54 files per mode)
with bounded regular-file reads for selected plaintext body/subject/name,
passwords, a plaintext payload prefix and complete member transport keys. The
maintained fixture audit separately checked actual device-held descriptor
key/IV/hash bytes and encodings against the provider snapshot and opaque mail.
The provider legitimately retains public accounts/devices/full identities,
certificate digests, policy/history metadata, ciphertext size/hash and timing.
This scan is an exclusion check within those files, not a memory, system-log,
whole-disk or forensic audit.

Successful per-mode member/provider stores and the exclusion-value corpus are
deleted by the driver. The retained private evidence has path/length/hash scan
coverage, console captures, opaque mail, SSH-denial records and transfer progress;
it does **not** contain the deleted stores or enough inputs to independently
replay the exact privacy scan. The audit's success was asserted, but its stdout
was not retained. Public JSON is a bounded projection of retained raw results,
whose original hashes are recorded. Outage fixture state and selected BBS store/
logs are privately retained; those scans found no selected private-mail markers.
Keys, member content and raw fixture configs are not published.

Rotation/reset does not recover lost OMEMO keys, revoke already saved recipient
copies or erase messages from recipients. Policy/ciphertext backups and retention
remain operator responsibilities; password reset creates no device approval.

## Teardown and limits

Owned host/guest SSH and BBS/file listeners and product children were verified
stopped. The disposable VM and serial controller were stopped, all three VM
forwards closed, and both attempted owned overlays/kernel copy removed. All 86
saved playground files were rehashed exactly; all six saved images were closed.
The owner's playground was never started. The clone's initial boot omitted its
established `-c`/`disable vmt` recipe; that failed attempt was retained before a
fresh overlay used the existing workaround. Setup and teardown precondition
failures remain private observations, not passing test stages.

Private archives retain only evidence for these owned disposable fixtures.
This does not establish a real deployed provider, cross-home delegation,
independent-client interoperability, physical hardware, broad hardening or new
capacity support. The separate [QXmpp preflight](../../research/0062-headless-reference-qualification.md)
is a selected-profile NO-GO; it does not qualify another reference or complete
[#295](https://github.com/gonz0w/faction-bbs/issues/295).

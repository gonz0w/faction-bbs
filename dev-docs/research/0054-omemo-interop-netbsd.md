# Two-way native OMEMO-2 and NetBSD qualification

Completed bounded research, 2026-10-24. [#200](https://github.com/gonz0w/faction-bbs/issues/200)
and controlled correction [#202](https://github.com/gonz0w/faction-bbs/issues/202),
following [0053](0053-native-omemo-reuse.md). Lead owns integration; separate
analyst/source review and final review apply under PROJECT.md. This is a
qualification result, not a product cryptography adoption decision.

## Result and decision

**Reject unchanged Picomemo 1.3.1 for the adapter.** Its self-tests pass, but an
independent peer's reciprocal reply fails authentication. An explicit isolated
`ad-order-v1` correction passes 92 unchanged-reference controls on both macOS ARM64
and NetBSD 11.0 aarch64. It is a viable research input for the next vault increment;
installation/support/update review and encrypted transactions remain adoption gates.
No product Cargo dependency, existing vault or member transport changed.

The simpler alternative was to keep the library unchanged. The required two-way
control disproved that option. A minimal source correction is testable and
reversible here; adopting a different complete crypto stack would reopen licensing,
profile and dependency qualification. No numerical technology score is justified.

| Gate | Observation and decision |
| --- | --- |
| Exact original two-way interoperability | Fail on host and NetBSD; reject original candidate. |
| Explicit corrected generated library + independent reference | Pass in this bounded matrix on both software routes. |
| Typed identities, SCE and whole-state rejection | Pass synthetic trust/binding/replay/error controls within stated caps. |
| Rust/NetBSD/no-JS selected execution graph | Pass selected builds/reference closure; no all-port claim. |
| Product store, client UI, live XMPP, library support/audit | Not delivered/established; blocks product adoption. |

## Pins, licenses and protocol profile

[Saved results](../evidence/omemo-interop/results.json) pin the original archive,
ten original/corrected upstream files, patch, all executed proof inputs, fifteen
reference wheels/licenses and thirty frozen intermediate inputs. Picomemo revision
is `835d8277108b96c691fb6a992512f37315cb52d7` (1.3.1); patch SHA-256 is
`8bc5d7560065865c81947099c7dabd401cdb21de3c3002d4466e3ce77f395465`.
ISC core/HACL MIT and Apache notices/OpenSSL terms remain; see the
[notice](../../experiments/200-omemo-interop/NOTICE.md). Nothing is relicensed by
Faction's BSD-3-Clause license. The experiment has no Cargo dependencies and does
not vendor or install the upstream library.

Independent reference: unchanged Twomemo 2.1.0, release commit
`9a9c76183e8bb3bdd677df27004464a5d4954e52`, wheel SHA-256
`dc799851d85f8badde8632511b5ad1091d9de95588a8d9c4d555764238f17f1a`.
Its Python dependency closure includes AGPL-licensed research components such as
Oldmemo; this isolated fixture is not a permissive product dependency decision.
The reference runs in host Python 3.13, not in the NetBSD product. Bootstrap pip's
unused Emscripten JS worker was excluded by removing pip before the final runtime
scan/run. No JS/TS selected runtime path or JavaScript execution was used.

Primary protocol/source inputs retrieved 2026-10-24:

- [XEP-0384 0.9.1](https://xmpp.org/extensions/attic/xep-0384-0.9.1.html),
  `urn:xmpp:omemo:2`, especially §4.2 initiation-associated data.
- [XEP-0420 0.5.0](https://xmpp.org/extensions/attic/xep-0420-0.5.0.html),
  `urn:xmpp:sce:1`; [XEP-0082 1.1.1](https://xmpp.org/extensions/attic/xep-0082-1.1.1.html).
- Pinned Picomemo [implementation](https://github.com/mierenhoop/picomemo/blob/835d8277108b96c691fb6a992512f37315cb52d7/omemo.c),
  [public API](https://github.com/mierenhoop/picomemo/blob/835d8277108b96c691fb6a992512f37315cb52d7/omemo.h)
  and [license](https://github.com/mierenhoop/picomemo/blob/835d8277108b96c691fb6a992512f37315cb52d7/LICENSE).
- Twomemo [release source](https://github.com/Syndace/python-twomemo/tree/9a9c76183e8bb3bdd677df27004464a5d4954e52)
  and [OMEMO storage implementation](https://github.com/Syndace/python-omemo).
- [RFC 8032 §7.1](https://www.rfc-editor.org/rfc/rfc8032#section-7.1)
  and [libsodium conversion API](https://doc.libsodium.org/advanced/ed25519-curve25519).

## Baseline defect and controlled correction

For either initiator, the unchanged library and Twomemo decrypt the first message.
The independent reply then fails native key authentication with `-3` / `ECORRUPT`,
with committed state unchanged. A diagnostic-only reversed-AD reference reply
passes key, payload and SCE checks. That deliberately modified diagnostic isolates
the cause and is **not** counted as interoperability. The original is rejected
on both routes; self-tests alone had missed this cross-implementation mismatch.

The specification fixes AD as active initiator identity followed by passive
responder identity. Original Picomemo chooses message-sender order for each MAC.
The [patch](../../experiments/200-omemo-interop/picomemo-ad-order.patch) retains an
OMEMO-2 initiation role and uses that fixed order in both directions. It changes no
primitive, KDF or ratchet algorithm. Monolithic/generated OMEMO-2 files agree with
Lua regeneration; generated legacy OMEMO-0 files remain byte-identical. No automatic
fallback or upstream installation/contact occurred.

Role 0/1/2 means uninitialized/active/passive. Corrected local serialization requires
canonical final field 16 bytes `80 01 ROLE`, checks bounds/state/role before writes,
and uses a different proof snapshot magic. Old/corrected snapshots reject each
other. An initial generic field-16 formatter attempt hit its `<16` assertion and
aborted; the exact rejected source/patch is retained. The accepted correction
leaves generic wire parsing/formatting unchanged. There is no vault migration.

## Identity, envelope and transaction oracles

Two independent existing-library vectors cover both Ed25519 sign bits. First,
Ed `d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a`
converts to Curve `d85e07ec22b0ad881537c2f44d662d1a143cf830c57aca4305d85c7a90f6b62e`
(sign 0). Second, Ed `03a107bff3ce10be1d70dd18e74bc09967e4d6309ba50d5f1ddc8664125531b8`
converts to Curve `4701d08488451f545a409fb58ae3e58581ca40ac3f7f114698cd71deac73ca01`
(sign 1). XEdDSA and independently executed libsodium 1.0.22 agree. Both signs of
a point map to the same Curve identity; that does not make full Ed pins interchangeable.
The proof requires exact `ed25519:` enrollment and rejects legacy Curve labels or
changed Ed sign. Existing 32-byte vault records are not silently reinterpreted.
The HACL conversion API is an encoding operation, not arbitrary-key validation.

SCE is a strict proof subset: UTF-8 subject/body with XML escaping, exact namespace,
sender/recipient, no duplicate/unknown shape, random-content padding and a zoned
XEP-0082 time within 300 seconds of the enclosing stanza's sending time. Delayed
receipt is allowed; receive-now is not its freshness oracle. Subject/body limits
are 128/1024 bytes, total plaintext 8192. Valid received padding may exceed the
sender's normal padding choice within that total cap. This does not prove every
SCE extension, XMPP stanza/routing rule or client implementation interoperable.

Rust uses an opaque generated-C handle and serializes process-global callbacks.
Each receive stages store/session/skipped keys; the private-pipe caller validates
SCE before commit. Rejection discards all staged changes, not just the ratchet
struct. Established prekey wrappers receive a bounded isolated fresh-session
identity check; its temporary derivations are discarded. The fixture binds exact
JID/device/pin outside the C library and never trusts a remote server-added device.

## Actual execution and TDD

The [experiment instructions](../../experiments/200-omemo-interop/README.md) describe
source verification, both build selectors, native OpenSSL paths and the private
reference runner. Final builds use Rust 1.96.0, release profile and C O2. Actual
NetBSD runs execute Rust/generated-C as unprivileged UID 1006 through private SSH
pipes in a marked disposable clone; the unchanged Python peer runs on the host.
There is no live XMPP listener, account, real mail or BBS-server decryption.

| Executed control | Host | NetBSD |
| --- | --- | --- |
| Original baseline reciprocal matrix | Rejected, 2 explicit controls | Rejected, 2 explicit controls |
| Corrected unchanged-reference matrix | Pass, 92 controls | Pass, 92 controls |
| Common final baseline Rust/FFI tests | 17 pass | 17 pass |
| Corrected Rust/FFI tests | 17 pass | 17 pass |
| Native CLI process/FD/EOF/bounds controls | Separate host controls | 7 pass |

The matrix covers both initiations and chained replies, Unicode subjects/bodies,
2/0/1 delivery, retained earlier-chain keys across reply ratchets and restarts,
actual native export/process-restart/import, reference backing-store restart,
replay, forged signatures, key/payload mutation/truncation, wrong namespace/JID/
device/identity, first and established prekey wrapper identity mutation, authentic
but invalid SCE, whole-state rollback and valid successors. Over-32 skip attempts
reject without committing, followed by valid low-counter receipt. Four stdlib
fixture tests and seven evidence-guard tests pass. Strict corrected Rust 1.96 clippy
passes. Neither native self-tests nor duplicated check labels replace independent
reference assertions.

Observed red: missing native interface compile errors before implementation
(initial cargo exit not separately captured), missing reference module, actual
reciprocal rejection, and a role-serialization test failing `None != Some(1)`
(cargo exit 101) before correction. The formatter assertion candidate is retained.
Evidence tests exposed missing checker/CI entry and false acceptance of product
status/patch substitution before those fixes. A native CLI helper initially
expected nonzero exit on incomplete EOF; the documented contract is `err` and
clean exit, so its oracle was corrected in #200 before changing that helper.
No implementation or refusal/privacy boundary was relaxed. Initial newer-toolchain
clippy found three codec style lints; actual prior source was preserved before
style-only refactoring. Both OS builds/matrices were rerun afterward on Rust 1.96.

## Measured costs and limits

NetBSD 11.0 evbarm/aarch64, two vCPUs, configured 1024 MiB RAM (observed
1,054,990,336 bytes), GCC 12.5.0 nb3, Rust 1.96.0, base OpenSSL 3.5.7.
Host: Darwin ARM64, Clang 21, Rust 1.96.0, OpenSSL 3.6.3.

| Cost observation | Host | NetBSD |
| --- | --- | --- |
| Corrected unstripped proof binary | 579,664 bytes | 624,368 bytes |
| Corrected proof process maximum RSS across recorded runs | 5,586,944 bytes | 6,044 KiB (6,189,056 bytes) |
| Native linked libcrypto shared object | Not remeasured in this increment | 6,244,640 bytes on disk |

The record retains every raw RSS observation with platform units and binary/library
hashes. The 27-byte libcrypto symlink is recorded separately from its target size;
the file size is not extra private RSS. NetBSD also links system crypt/C/pthread/
GCC support. RSS is the short-lived proof process only: it excludes the separate
Python reference, existing vault Argon2 unlock, future complete Faction client,
server load and VM overhead. No 250-user, physical SBC, throughput, all-port or
production memory claim follows.

Limits are fixed at one peer/process, plaintext 8192, ciphertext 8208, key message
256, snapshot 32768 bytes, 32 stored skipped keys and 64 isolated verification
probe derivations. Heartbeat routing is rejected. Prekey retention here is for
synthetic delayed tests only, not production lifecycle policy. Raw research snapshots
contain private synthetic state; they are neither encrypted nor crash-safe stores.

## Confidence, cleanup and adoption boundary

High confidence in this finite reproducible corrected-vs-original outcome; low
confidence in untested client/port/production claims, for which no result is made.
A separate read-only source reviewer reproduced 17 Rust/FFI plus four stdlib tests
and found no actionable patch findings. That review is not a cryptographic audit.
Requested PROJECT routing settings are recorded in issues; native model/effort
routing metadata was unavailable, so observed routing remains unverified.

The owned clone, overlay, keys, Python runtime and build directories were removed
after sanitized evidence export. Its loopback forwards are closed. The saved
owner playground remains stopped, with retained owner file hashes and disk
size/mtime unchanged. No historical #199 or earlier evidence was refreshed.
Offline CI validates saved results/source identity and limitations; it never
relabels those checks as new VM/crypto execution.

[#203](https://github.com/gonz0w/faction-bbs/issues/203) owns supported source/build/
update review and encrypted atomic state/message/outbox checkpoints, preserving
v1 vault drafts/recovery and explicit verified Ed enrollment. The correction can
still be discarded without member migration because it is uninstalled research.
[#90](https://github.com/gonz0w/faction-bbs/issues/90) remains open for actual bounded
inbox/send/retry/dedup/history/recovery and trusted local UI. Live transport,
roaming #91, attachments #88 and audit/adoption policy are separate gates. No
server-readable fallback or automatic visited-board membership is introduced.

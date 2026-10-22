# 0041: Broader articles and independent INN validation

Issue #103. Status: implemented and validated, 2026-10-21. Keep hardening #106–#111 later.

## Outcome and steps

- [x] Repair #113 standalone locks and merge after exact-head CI passes (fe6a394).
- [x] Preserve all 95 original execution inputs and unchanged result before edits.
- [x] Add profile-v2 UTF-8 encoded subjects, common numeric-zone date forms and
  bounded text/plain MIME decoding/encoding with exact body round trips.
- [x] Represent long local lines without modifying content; bounded reply pruning
  retains root and recent ancestors without unbounded history traversal.
- [x] Validate emitted and returned articles with original INN inews in offline
  mode, including negative controls; no news service/provider/publication.
- [x] Run host/native NetBSD, real terminal and source/policy checks; update docs/PR.

## Constraints

No live adapter, implicit room export, remote authority, new owner helpers or
credential/session/default changes. Schema 6 and current ASCII composer stay;
UTF-8 wire subjects are codec support, not a hidden storage migration. Independent
INN is an optional offline reference tool, not an adopted product runtime. Review
its exact source/version/license/no-JS graph, use private synthetic fixtures and
never invoke a posting mode. A temporary test-only loopback refusal listener is
allowed; it sends only 400 and verifies zero commands. No product/public listener. Preserve the owner's stopped playground and all ten
source snapshots/twenty-two historical result identities. Software VMs suffice.

## Actual outcomes

133 Rust tests pass on macOS/native NetBSD; host clippy/format/build and 104 Python
tests pass. Twelve original INN controls pass on each system, with actual runtime
hashes and separate Python MIME parsing. Seven shared native UID/PTY subject,
paging/unread/revocation controls pass. A 16-KiB body encodes to 17,775 bytes;
extra peak requested live allocation is 66,277 parse / 32,256 serialize on both
hosts, excluding allocator/stack/RSS/whole-service/concurrency costs. QP appends
directly without per-byte temporary strings. No capacity/hardware claim.

The native reference build required a no-dependency official Perl development
package. Initial hostname lookup caused a timeout even in INN -D mode; final
INN_HOSTNAME=fixture.invalid skips DNS and a temporary owned loopback 400 fixture
proves zero client commands. INN accepts the deliberately invalid Date; Faction
refuses it. These findings are retained, not converted into a full-server claim.

All 100 current inputs were byte-compared with the executed native checkout.
The disposable VM/reference package/synthetic state were removed; original
playground remains stopped and preserved. Ten snapshots and twenty-two old result
identities stay fixed. No new registry packages/features, schema migration,
owner helpers, credential/session/default change, live feed or hardening activation.

The six #103 acceptance criteria are implemented for the explicit bounded local
profile. #101/#104 own real injection, peer/provider transport and consent/history;
#114 separately owns Unicode composing and versioned metadata compatibility.

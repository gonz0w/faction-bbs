# 0040: Authored subjects and local room articles

Issue #103. Status: implemented and validated, 2026-10-21. #112 merged after checks passed.

## Outcome

Add subjects to local/shared posting, sensible reply defaults and retry safety.
Connect schema-6 records to the bounded codec using privacy-safe local author,
group and Path projections with current approval and file permission checks.
Preserve unknown legacy dates rather than fabricate wire dates. Define bounded
reply references and exact duplicate/conflict comparison without an import/feed
runtime. Keep unsupported text/header/MIME handling explicit.

## Steps

- [x] Merge #112 and freeze its 90 actual execution inputs and original result.
- [x] Implement subject entry/default/retry and test both real terminal paths.
- [x] Implement local article projection, references and duplicate/conflict tests.
- [x] Validate software/NetBSD behavior, bounded costs and independent parsing.
- [x] Update docs/status/decision and open a focused PR; keep unfulfilled #103 gates visible.

## Boundaries

No hardening/NPF activation, new listener, room publication, peer import, approval
from headers, changed credential/helper/session limits or physical hardware gate.
Keep ASCII subjects for the current ASCII terminal input; UTF-8 body/unsupported
MIME policy must be explicit. No invented dates, private home/board/session/address
metadata or automatic resource fetching. Use only synthetic private VM fixtures;
keep the owner's saved playground off and unchanged. Preserve nine snapshots and
all twenty-one historical result identities.

## Actual results and remaining gates

127 Rust tests pass on macOS and native NetBSD 11 ARM64; host release build/clippy/
format and 104 Python tooling tests pass. Real local/plain/ANSI screens pass on
both hosts. Seven native shared UID/PTY controls include inherited subjects and
authored overrides, partial/full unread and revocation. Host export/restore and
member-worker checks pass. Independent Python MIME parsing passes on both hosts;
this does not establish independent news-server exchange. For a 16-KiB body the
codec emits 17,123 bytes; extra peak requested live allocation is 65,745 parse /
32,256 serialize on both hosts, excluding stacks/RSS/allocator/whole-service costs.

The final record pins 95 real source inputs. Disposable VM/test state removed;
owner's stopped playground/checkpoint remain preserved. No new registry packages,
owner helpers, credential/default/session changes, live NNTP, remote import,
capacity or hardening adoption. Core now depends on the existing article crate.

#103 stays open for internationalized header entry, wider date/MIME support,
reference pruning and independent news software. The implemented core API is local
projection, not a member export action, network feed or remote insertion. #101/#104
keep consent/injection/peer/provider gates. Hardening remains later #106–#111.

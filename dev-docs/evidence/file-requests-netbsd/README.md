# File request desk: native software evidence

2026-10-23 · Issue #140 · [results](results.json)

A clean offline build of all 18 workspace packages on an owned **NetBSD 11 ARM64**
VM (2 cores/1 GiB) passed **321 Rust tests**, matching the developer host. Both
systems passed six new local/worker request PTYs, eight library regressions and
**18 complete backup/recovery controls**. Native recovery uses the actual CLI under
distinct non-root owner/file UIDs; the developer-host fixture uses its local API.
The host additionally passed 130 Python tests, strict clippy, dependency inventories
and the maintained Rust/terminal workflow.

Eight actual shared-service controls traverse restricted SSH, the host native
client, the guest native client and the existing restricted SFTP service:

- Member request review/cancel/create and persistence across reconnect.
- A sysop cannot fulfill a private file for an unauthorized requester; an eligible
  published file succeeds only after explicit rights/source/YES review.
- Request → native Files populates the recent-link list; zero payload allowance
  defers without creating transfer grants.
- Compact native NetBSD downloads the exact 34-byte synthetic original manual,
  verified by SHA256 through restricted SFTP.
- Plain manual SSH request details/links emit no escapes.
- Revoking an area grant removes a held fulfillment token and provenance.
- Concurrent close rejects a stale review without silently adopting a revision.
- Sysop deletion is explicit; live account revocation expires the member session.

Unchanged unread state is compared before/after the request journeys. Additional
Rust controls cover both actors' file access, pending/removed/missing objects,
foreign sessions, stale revisions, Busy retry, ownership, quotas, hostile text,
exact schema-14 export compatibility and schema-15 request recovery. Long reviews
page in full; resize never posts or implicitly confirms. Existing native gallery,
travel, discovery, catch-up, polls, events, pages, wall, writer, reader, operator,
launcher, directory, Nightwire, offline vault/picker and 18 artwork profiles also pass.

The record pins **288 matched native execution inputs**, **33 host harness inputs**,
13 unchanged dependency/access/credential inputs, five build hashes per OS and
three installed native hashes, including the actual request prompt in the service.
Source snapshot 38 preserves the previous 292 library inputs and complete prior
result before changes. All historical results/hashes remain unchanged. Guard it
without a VM/network using `python3 tools/check_file_requests_evidence.py`.

Normal limits remain eight sessions, 5000-ms admission, two credential permits,
three owner helpers, 32-KiB output and five-second drain. New request limits are
256 retained/8 open per member, four cards, eight text rows and bounded cursors;
there is no new registry dependency, helper, listener or transfer authority.

The disposable VM, private test inputs and temporary vendor cache are removed;
the saved owner playground stays stopped with disk/accounts/keys/helpers intact.
No physical SBC, Ghostty pixel, sustained 250-member capacity, public library,
private-mail runtime/federation, secure erasure or complete-hardening claim.

# 0066: Local file requests share the library and existing catalog authority

Status: accepted for the hobby preview · 2026-10-23 · Issue #140

Use the existing shared Rust library workflow and owner-reviewed pack for an
optional requests-only or combined shelf/request experience. Persist bounded
ordinary community requests in schema 15. Do not store requests in mutable host
manifests or invent another file store/service. SQLite snapshots retain the desk;
external pack configuration remains separately preserved.

Approved members create immutable requests with member-scoped retry nonces. Any
approved member may fulfill an open request only with a current published local
catalog object readable by both fulfiller and requester in the same transaction.
Author/sysop close/delete and exact revision comparisons keep moderation explicit.
A terminal review is not mutation authority. Current approval/epoch/role and source
access are checked again on commit; Busy retains explicit retry state.

Project each offer under the current viewer's file rights before showing its ID,
rights or provenance. Deleted/revoked/unpublished objects leave a generic unavailable
fulfillment, without deleting the request or opening a successor. Store object IDs
without a foreign key deliberately, so removing catalog metadata cannot silently
delete community request history. A fulfilled state describes the past operation,
not a promise that the object remains eligible. Sysops retain ordinary owner access.

Keep 256 retained/8 open-per-member requests, four cards/frame and fixed ASCII/input,
output and cursor caps. Explicit deletion frees retention; no automatic content
expiry or member activity tracking. Existing restricted SFTP and client confirmations
remain the only transfer authority. Reuse the common frame/feed adapter for local,
worker/shared, classic/narrow and native visits; no new dependencies or protocol.

Alternatives: owner-only fulfillment would burden ordinary communities; public
unfiltered link/provenance display would leak private catalog metadata; automatic
uploads/fetches would bypass existing review and broaden scope. Federation/private
mail, richer editing, proposals/acceptance queues and later hardening remain outside
this bounded local desk. None is needed for #140's current acceptance.

Reversal requires a stopped complete older backup and matching binaries in fresh
paths, not dropping tables or relabeling a live database. Software NetBSD validation
is evidence for this preview, not physical SBC/250-member or complete-hardening proof.

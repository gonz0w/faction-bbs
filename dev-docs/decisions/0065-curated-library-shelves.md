# 0065: Curated shelves over existing local file authority

Status: accepted for the hobby preview, 2026-10-23. Issue #140, first increment.

## Decision

A sysop-curated collection should make existing files easier to discover without
copying file data, publication rules or transfer authority. Use a small immutable
owner-managed startup pack, shared by Arc in the service, and one faction-library
workflow across local/worker/shared adapters. Required curator notes, rights and
provenance describe existing canonical local file references. Current core file
resolution precedes captions, notes, shelf enumeration and every selected action.
Hidden/private-only shelves disclose neither names nor counts. Selected entries
retain stable pack IDs; unavailable sources return to a generic list.

Deliberate Files jumps recheck the source and reuse the existing native recent-link
and key-bound SFTP flow. Optional discussion rooms use current core room checks and
the normal reader; previews do not acknowledge unread, actual reads do. A missing
room is independent of an otherwise eligible file. No new schema, registry package,
network/exec authority, helper/listener or resource defaults.

[The guide](../../docs/library-shelves.md) defines configuration, all caps, curation,
rights limits, stopped withdrawal/restart and external-pack recovery. Logical
withdrawal cannot recall prior downloads. No public publication or private-mail
projection. Schema remains 14 and existing complete file recovery is unchanged.

## Alternatives and consequences

Reuse current core authority rather than a parallel library ACL/copying model.
A database-backed request/shelf editor could support live curation, but would need
its own schema, revision/ownership and moderation/recovery work. That remains the
next #140 increment, together with request/create/fulfill/close; do not close the
parent on this shelf increment. Manual owned packs match current themes/galleries
and are sufficient to make a small local collection useful now.

Rollback removes the new host config/flags and uses the previous matching binary;
there is no new DB state. Preserve exact packs separately from database backups.
[Validation](../evidence/library-netbsd/README.md) uses owned software fixtures;
capacity, physical hardware, broader hardening and federation remain separate.

# 0063: Owner-curated terminal gallery packs

Status: accepted for the hobby preview, 2026-10-23. Issue: #144.

## Context and decision

Boards need original creative exhibits without confusing art viewing with public
publishing, arbitrary uploaded controls or bitmap authority. Reuse the existing
canonical CP437/ANSI/SAUCE/iCE importer and original Nightwire cells. A small
immutable owner-managed TOML pack is loaded before admission, shared by Arc in
the service, and passed to one faction-gallery workflow across local/worker/SSH.
It records required provenance, rights/credit and explicit ASCII text editions.
Trusted themes may arrange the host-optional action; classic text remains usable.

Choose manual owned-pack review/restart over a new database/submission/moderation
subsystem. Existing pending Files or offline intake handles submissions; explicit
owner review establishes permission and installs/removes works. Metadata claims
are not license verification. Downloads need separate permission and a currently
resolvable existing local Files token; no copied grant or automatic transfer.
No new schema or registry package, no new listener/helper or relaxed budgets.

[The guide](../../docs/scene-gallery.md) defines all caps, configuration, curator
workflow, removal, fallback, download boundary and external-pack recovery. Canonical
cells/pages are shared immutable startup data, not retained copies in each Desk.
Active sessions deliberately retain the installed snapshot until a stopped/drained
restart; emergency withdrawal requires stopping it. No secure-erasure claim.

## Alternatives and consequences

A new DB-backed gallery/submission queue could make live curation convenient but
would add migrations, storage lifecycle and moderation before proven need. Generic
uploaded image/terminal playback would widen terminal-control authority. Both stay
outside this first usable exhibit. Pack quotas favor small, deliberate shows;
sysops can replace a pack between events. No archive is imported without permission.

Rollback uses the prior binary/config with `gallery`/`--gallery` removed; schema 14
is unchanged. Preserve external packs separately from existing complete backups.
[Validation](../evidence/gallery-netbsd/README.md) uses owned software fixtures;
capacity, physical SBCs, Ghostty pixels, federation, marketplace and hardening
remain separate work.

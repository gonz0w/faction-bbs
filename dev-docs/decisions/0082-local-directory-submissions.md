# 0082: Locally moderated directory submissions

Accepted for [issue #184](https://github.com/gonz0w/faction-bbs/issues/184),
2026-10-26. Extends [the curated directory](0050-curated-local-other-board-directory.md)
without changing its pack format or giving member input configuration authority.

## Decision

Store bounded member drafts, pending reviews and published/expired listings in
schema 19 of the existing SQLite database. Use immediate transactions and exact
revision checks for publication and removal. Keep the curator TOML unchanged.
A trusted startup configuration installs validated curator keys on the Board
before member service listeners exist; session calls cannot supply conflict keys
or enable the feature. Ordinary operation remains independent of this feature.

Use the same inert card field bounds and endpoint normalization as the curated
pack. Permit 32 draft/pending records, four per author, and 32 combined curator
plus published/expired records. Drafts reserve no endpoints. Submission and
approval check slug and canonical host/port/login conflicts. Expired or hidden
published records retain their reservation. Current authorization is checked in
every operation; approval also requires the author's current local approval.

Retention is count bounded: records remain until explicit withdrawal, rejection
or removal. These actions delete the row, with no rejection text or history.
Expiry retains the row and shows OLD, preserving existing advisory date behavior.
Private owner exports include all submission state; curator packs remain external.
Restore requires fresh sessions and compatible trusted configuration.

## Alternatives and consequences

Editing the owner pack from a member flow would conflate editorial configuration
with untrusted submissions and require a second atomic recovery mechanism.
A separate queue database would duplicate authorization, export and migration
work. The bounded SQLite overlay uses the existing transaction and recovery path.
No scheduler, remote registry, contact proof or generic RPC is needed.

Published rows from unapproved authors are hidden, retained and counted until
approval returns or the sysop removes them. Startup rejects conflicting or
oversized curator/published unions instead of dropping either source. A live
catalog change resets browsing selection; an old numeric index never selects a
different endpoint. Nothing resolves, connects, forwards credentials or grants
access to a destination. Curator entries keep owner pack/restart management.

Software verification must cover current authority, exact revisions, duplicates,
size/count boundaries, migration, restart, export/restore and real classic/plain
terminal routes on the developer host and native NetBSD. Evidence describes actual
executed sources and observations; it does not establish production hardening,
physical SBC support or higher capacity.

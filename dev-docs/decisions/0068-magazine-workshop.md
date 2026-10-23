# 0068: Local magazine workshop and exact-issue consent

Status: accepted for this bounded implementation, 2026-10-23
Issue: [#143](https://github.com/gonz0w/faction-bbs/issues/143)

## Decision

Store local manuscripts, copied editorial proposals and versioned issue consent
in schema 16. Approved members author their own source; sysops edit the proposed
issue without rewriting another member's source. Each author approves all of
their selected contributions in the complete current issue. Every header,
selection or proposal change rotates a random token, invalidating all approvals.
Publication requires current submitted sources, approved authors and exact
matching consent in one immediate transaction.

Keep the immutable static-pack reader from ADR 0067 for curated original content.
A separate opt-in common workshop serves local/worker/shared terminals. All
prose is bounded inert ASCII and all release actions require explicit review and
confirmation. No imports from rooms or private mail; authors must provide only
original/permitted material with rights and provenance.

## Alternatives and evidence

Static editor-declared consent alone cannot authenticate a contributor. Editing
authors' sources would obscure provenance. Automatic room extraction could
republish private discussion without intent. Public release, cryptographic
signatures, delegated editors, automated schedule execution and external
syndication add separate policy/identity requirements and remain follow-on work.

Unit and actual terminal evidence are recorded in plan 0073 and the editorial
software record. No dependencies, listeners, thread pools or default resource
limits change. Files keep their existing separate permissions/transfer service.

## Consequences and reversal

Local drafts/reviews are server-readable editorial work, never end-to-end private
mail. Assigned contributors can see the whole review cohort issue after the
author explicitly submits. Published text stays immutable; future changes need
another edition and new approvals. Source withdrawal or consent revocation
withdraws the issue; copies and backups remain. Related Files offers are separate
curator metadata outside the approved article content; the sysop verifies bytes
and manages any file withdrawal separately.

Approval uses trusted local account sessions and database integrity, not a
cryptographic signature resistant to a malicious filesystem owner. Account
revocation immediately hides affected issues; reapproval may restore visibility
unless consent/source was withdrawn. Schema 16 is forward-only for a live board;
use a stopped complete backup before upgrade. Disabling presentation preserves
retained manuscripts/consent; moving back to older software requires the matching
pre-upgrade backup, not a downgrade of a live catalog.

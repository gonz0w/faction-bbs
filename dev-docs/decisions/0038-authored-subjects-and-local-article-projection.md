# ADR 0038: Authored subjects and bounded local article projection

Status: accepted for the local prototype, 2026-10-21. Issue #103; follows ADRs
0036/0037 and merged #112. No injection, peer transport or production adoption.

## Question and constraints

Make rooms more useful through authored subjects and connect existing metadata
with the codec, without turning representation into publication or changing
chronological reading, local approval, file permissions or private authority.
The owner requested continued #103 development and deferred hardening.

## Decision

Keep schema 6. Add a borrowed PostDraft with subject/body/parent to the posting
transaction; keep the existing post API as a default-subject wrapper. An empty
subject chooses a generated label or one bounded inherited Re: prefix. Preview
the reply default, store its resolved value once and include the subject in
idempotent request comparison. Reject non-ASCII/encoded-word/control subjects
until a deliberate header-encoding/composer increment; do not silently transliterate.

Project approved local records with persistent public IDs, per-article reserved
author aliases, UTC from the stored new-post time, local numeric group aliases,
not-for-mail Path and at most sixteen ordered same-room ancestors. Unknown dates,
cycles/deeper chains and unrepresentable lines fail explicitly. Validate current
file permissions and retain exact canonical bodies; no per-viewer redacted body
under the same article ID. Existing normal room reading remains available.

Strict normalized duplicate comparison returns distinct/duplicate/conflict. Any
changed field/body, including Path, conflicts under the same ID; no overwrite,
remote insertion or authorization from headers. This conservative policy can
later be revised for validated relay semantics without rewriting local IDs.

## Alternatives and tradeoffs

Mandatory subjects add friction; optional defaults keep the BBS flow. Retrofitting
all generated labels as authored would falsify provenance. Reusing handles/home
names leaks identity; per-article aliases deliberately sacrifice cross-post author
continuity for this local stage. Real peering identities/groups/Path require #101's
consent and injection policy, rather than enabling them implicitly. Fabricated
legacy dates, silently wrapped bodies and truncated reply ancestry would hide
loss; fail rather than mutate. Broader MIME/encoded headers remain a focused
follow-up instead of accepting an unreviewed permissive parser.

## Validation and reversal

Test subject limits, retry conflicts, approval/revocation, restore stability,
Unicode body round trips, privacy-safe fields, bounded ancestors and denied file
references. Real plain/ANSI and shared NetBSD PTYs must exercise authored/default
subjects. Compare synthetic output with Python's independent email/MIME parser;
this is not news-server interoperability. Measure encoded bytes and requested
parse/serialize allocation separately, never convert them to a capacity claim.
The schema is unchanged; retain subjects on rollback and coordinate stopped
binaries as documented. API/projection policy is reversible without public feeds.

## Sources and confidence

Primary references retrieved 2026-10-21:
[RFC 5536](https://www.rfc-editor.org/rfc/rfc5536),
[RFC 5537](https://www.rfc-editor.org/rfc/rfc5537),
[Python email policy](https://docs.python.org/3/library/email.policy.html).
These separate article representation, injection and MIME parsing. Local reserved
aliases and strict conflict behavior are Faction design choices inferred from our
privacy/bounded-work requirements, not a claim of universal Netnews compatibility.
Confidence is limited to the supported text profile and recorded software tests.

# ADR 0040: Versioned Unicode subjects and a classic display

Status: accepted for the software prototype, 2026-10-22. Issue #114; follows
merged #115 and ADR 0039. Hardening #106–#111 stays later.

## Decision and constraints

Use schema 7 for stored UTF-8 subjects. Preserve schema-6 ASCII validation for
old exports/migration; an old binary must reject the new version. Validate the
exact old metadata catalog and original rows before atomically rebuilding its
subject constraint with BLOB byte length. Preserve IDs, authored/generated labels,
UTC/origin and legacy unknown dates. Stopped complete backup/upgrade/rollback is
mandatory operational procedure, not a claim that owner-open detects all writers.

Use one small dependency-free subject input module in faction-terminal across
local/worker/shared paths: 160 UTF-8 bytes, incremental validation, complete-scalar
and incomplete-tail deletion. Malformed input fails the connection. Reply defaults
clip at character boundaries. Existing approval/retry/session/credential/owner
resource rules stay. No codec profile, networking or owner-helper change.

Keep classic ASCII escapes for input echo, paged labels and reply previews, with
independent cell clipping. This safely supports any valid stored Unicode without
trusting glyph widths, combining/bidi behavior or TERM/brand claims. Full native
glyph rendering is #96. The cost is a less natural display on modern terminals;
that is explicit, not claimed as completed Unicode rendering.

## Alternatives, validation and reversal

Silently widening version 6 lets older binaries accept stores they cannot validate;
reject it. Assuming one character equals one cell breaks wide/combining text and
can leak controls into previews. A new Unicode width/dependency/capability stack
is unnecessary for this compatibility increment and remains separately scoped.

140 host/native Rust tests cover multibyte limits, exact bytes, conflict/retry,
revocation, old catalog/data refusal, rollback and restore. Real local/worker
PTYs pass five controls on each host; native shared PTYs pass twelve. No new
capacity, physical hardware, live peer or hardening claim. Retain original
100-input #115 evidence in sources/pre114 rather than refreshing old hashes.

Reversal restores a matching complete schema-6-or-older backup with old binaries
into fresh paths. There is no in-place downgrade; later posts would be absent.
See docs/unicode-subjects.md and plan 0042 for operational details/evidence.

# ADR 0039: Broader local text articles and original INN validation

Status: accepted for the software prototype, 2026-10-21. Issue #103; follows ADRs
0036–0038 and merged #113. No network runtime/injection/production adoption.

## Question and constraints

Support useful UTF-8/MIME text interchange and long conversations, then validate
with original news software. Keep schema 6, current approval/file rules and exact
bodies. Physical hardware and later hardening must not delay product work.

## Decision

Adopt bounded codec profile v2 as specified in docs/article-profile.md. Parse
UTF-8/US-ASCII B/Q subjects, common numeric-zone dates and single text/plain MIME
7bit/8bit/quoted-printable/base64. Emit canonical UTF-8 B subjects where needed and
quoted-printable bodies. Reject unsupported charsets/headers/parts/controls rather
than fetch/execute or silently discard data. Use the existing reviewed base64
package, with no new registry versions/features. Keep the schema-6 composer ASCII;
#114 owns a deliberate versioned Unicode-composing/storage increment.

Use real MIME final-newline semantics, replacing v1's extra wire terminator. Keep
legacy fixture bytes/results fixed and describe their standard decoded final LF.
No DB/body migration is performed. For bounded reply history, traverse at most
128 ancestors and emit root plus nearest fifteen; reject cycles/deeper work.

Build original INN 2.7.4 inews without installation as a reference, pinning archive,
source no-JS scan and actual runtime artifacts. Its -D mode still resolves a host
and opens NNTP: set a synthetic INN_HOSTNAME and use a temporary owned loopback
fixture that sends only 400 and checks zero client commands. No real service,
provider or posting is involved. Retain its observed invalid-Date acceptance;
Faction's own validator must reject it regardless.

## Alternatives and tradeoffs

Plain 8bit cannot represent long physical lines unchanged. QP costs encoded bytes
but supports exact content and bounded physical lines. Base64 output is compact
for dense Unicode but poor for ANSI-era inspection; accept it as text input and
emit readable QP. Do not introduce a permissive full MIME/HTML parser for text
rooms. Deferring Unicode storage avoids silently widening schema-6 compatibility.

Unbounded ancestor scans violate work limits; rejecting every seventeenth ancestor
blocks normal conversations. A fixed traversal cap plus root/recent pruning is a
reversible local policy. Actual future relay Path/References processing stays #101.
INN inews validates real news processing independently; it is not a full server,
so no live interop/adoption claim. Python email parsing remains a separate control.

## Validation, confidence and reversal

133 Rust tests on host/native NetBSD include encodings, malicious values, exact
whitespace/newlines and 128-ancestor cap. Twelve original INN controls on both
hosts include roundtrip, rejection and the stricter Faction Date rule. Existing
reader/subject/unread/revocation, migration and snapshot tests stay. Measure wire
size/requested allocation separately from RSS/whole-service costs; no capacity
claim. No new owner threads/helpers, credential/session defaults or listener.

Core model/storage reversal preserves schema-6 records; offline codec semantics
must select the version deliberately. Unknown legacy dates remain unknown.
Confidence covers this bounded text profile and offline controls, not arbitrary
providers, all NetBSD ports, physical power loss, independent audit or live peers.

## Primary sources

Retrieved 2026-10-21: [RFC 2045](https://www.rfc-editor.org/rfc/rfc2045),
[RFC 2047](https://www.rfc-editor.org/rfc/rfc2047),
[RFC 5322](https://www.rfc-editor.org/rfc/rfc5322),
[INN inews documentation](https://www.eyrie.org/~eagle/software/inn/docs/inews.html)
and [official INN archives](https://downloads.isc.org/isc/inn/).
The pinned source's frontends/inews.c and lib/getfqdn.c establish the connection/
hostname behavior. Faction's limits, rejected subsets and fixture are design
choices, not a claim that standards require these exact bounds.

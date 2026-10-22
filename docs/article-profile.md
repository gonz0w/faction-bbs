# Local article profile v2 and independent news-software checks

Status: implemented software profile, 2026-10-21, #103. This is local article
formatting and parsing, not a news service or authorization to publish a room.
SQLite schema stays 6; local account/approval, file policy and unread state stay.

## Supported text profile

The Rust codec exposes `PROFILE_VERSION = 2`. Its model holds decoded UTF-8 text;
From stays a bare ASCII mailbox. Message-ID, local numeric group aliases,
not-for-mail Path and bounded References keep their existing identity rules.
Remote From/Path/References never grant membership or sysop authority.

| Area | Accepted input and canonical output |
| --- | --- |
| Subject | Up to 160 decoded UTF-8 bytes, no controls/surrounding whitespace. Parse UTF-8 or US-ASCII B/Q encoded words; preserve whitespace except folding between adjacent encoded words. Emit non-ASCII/ambiguous subjects as UTF-8 B words, each within 75 bytes and split at character boundaries. |
| Date | Numeric-zone dates, optional correct weekday, one/two-digit day and optional seconds. Validate calendar/time/weekday; normalize to `DD Mon YYYY HH:MM:SS +HHMM`. No named zones, obsolete comments or guessed legacy date. |
| MIME | Single text/plain part, UTF-8 or US-ASCII; accept quoted/case-insensitive charset syntax. Parse 7bit, 8bit, quoted-printable and base64 with bounded decoded text. Absent MIME uses text/plain US-ASCII/7bit defaults; MIME fields require version 1.0. |
| Body | LF-normalized UTF-8 text. Canonical output is quoted-printable with at most 76 physical bytes per line and CRLF framing. No logical-line wrapping, trailing-whitespace loss or invented final newline. |
| Limits | 24 KiB total wire, 4 KiB headers, 16 KiB decoded body, 998 physical bytes per input line, four groups and sixteen emitted References. Encoded size may reach the wire cap before decoded-body cap. |

Reject unknown/authority/control headers, HTML, multipart/binary attachments,
unsupported charsets, malformed encodings, decoded terminal controls and oversized
content. The codec opens no files/URLs and executes no terminal graphics. Sysop
presentation remains separate. This is not a universal provider/MIME parser.

## Exact bodies and long conversations

Quoted-printable soft breaks let every ordinary 1024-byte local single-line post
fit without changing its text. Preserve zero/one/multiple final newlines and
trailing spaces/tabs through parse/serialize and the independent reference tool.

Local projection traverses at most 128 descending same-room ancestors. If more
than sixteen references are available, retain the root plus the nearest fifteen,
in order. More than 128 or cyclic/cross-room data fails explicitly; ordinary
chronological reading remains available. A currently inaccessible file reference
or unknown legacy date still prevents article projection rather than creating a
viewer-dependent body or fictitious timestamp. No partial message becomes read.

## Compatibility and scope

Profile v1 appended and removed an extra wire newline. V2 uses normal MIME body
semantics: every decoded newline belongs to the body. The old handwritten 8bit
fixture therefore has a final decoded LF. No stored post, metadata ID, subject,
reply link or database snapshot is rewritten. Historical v1 output/results/source
bytes remain preserved. Do not treat old offline v1 output as an invented date or
authority, auto-detect its writer's intended final newline or mix codec versions
as an unreviewed feed; use the recorded v1 tool when reproducing its old semantics.

The codec now uses the already reviewed base64 crate; no new registry package or
feature is selected. The terminal composer/storage validator stays ASCII under
schema 6. [#114](https://github.com/gonz0w/faction-bbs/issues/114) owns UTF-8 composing
with versioned metadata compatibility; decoded wire-subject support is already here.
A strict normalized duplicate matches all fields/body; changed content under one
ID, including Path, conflicts. Relay-specific equivalence and persistent remote
import/dedup remain #101/#104, alongside room/author/peer/history consent.

## Original INN validation

The optional reference builder compiles original INN 2.7.4 library/inews from the
[official archive](https://downloads.isc.org/isc/inn/inn-2.7.4.tar.gz), with pinned
SHA-256 and no-JS filename review. It installs nothing and starts no news server.
It needs Python 3.12+, a C compiler/make and Perl as upstream build tooling;
NetBSD's reference-only fixture used official perl-5.42.3 with no package dependencies.
These are not new BBS runtime dependencies or an adopted INN deployment.

```sh
python3 tools/build_inn_reference.py --archive /path/to/inn-2.7.4.tar.gz --work-dir /new/private/inn-reference
cargo build -p faction-articles --examples --release --locked
python3 tools/check_inn_articles.py --reference-dir /new/private/inn-reference --examples target/release/examples
```

`inews -D` still opens a connection and resolves its host name. The harness fixes
INN_HOSTNAME/configuration to fixture.invalid and the endpoint to an owned temporary
loopback fixture. It sends only a 400 refusal, verifies zero client commands and
closes on completion. No posting mode, credentials, real board, provider or public
listener is used. Convert CRLF to Unix LF at inews's local-file interface and back
before Faction parsing; canonical article semantics must be identical.

Twelve controls pass on macOS and native NetBSD: eight round trips covering UTF-8
subjects, long lines, final bytes/whitespace, MIME defaults, base64 and Q words;
three INN rejection controls (duplicate Subject, invalid Message-ID and empty
body); and a deliberate invalid-Date control that INN accepts but Faction rejects.
INN acceptance is therefore not a universal RFC validator or peer/server proof.
Future live NNTP acceptance, peering, provider credentials and privacy/retention
policy remain #101/#104. PGP/VM/storage hardening #106–#111 stays later.

See [software evidence](../dev-docs/evidence/broader-articles-netbsd/README.md),
[ADR 0039](../dev-docs/decisions/0039-broader-articles-and-inn-validation.md) and
[research 0048](../dev-docs/research/0048-offline-inn-article-validation.md).

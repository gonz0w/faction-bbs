# 0048: Original INN as an offline article control

Date: 2026-10-21. Issue #103. Outcome: useful reference-only control; no maintained
news runtime/provider/peering stack selected. See ADR 0039 and plan 0041.

## Question and screening

Can original news software independently process our bounded local articles
without publishing or contacting a news provider? Preserve NetBSD, no JS/TS,
small bounded product state, local consent and software-only validation.

| Candidate | Evidence and use |
| --- | --- |
| Python email/MIME | Existing independent syntax/body parser, useful but not news software. Retain as a separate control. |
| INN 2.7.4 inews | Original C news posting/preparation tool with offline -D output. Selected for exact input/output and negative controls; not installed as a server or adopted runtime. |
| TIN reader | Relevant C terminal reader, but display/spool validation adds a UI/terminal fixture. Defer actual reader/peer work to #101 instead of treating a screenshot as a byte-level roundtrip. |
| Full INN feed/server | Required later for independent live peering/injection/privacy policy. No board has yet authorized a feed under #101/#104. |

## Source and build provenance

Official INN source: https://downloads.isc.org/isc/inn/inn-2.7.4.tar.gz .
SHA-256: 80fc7e801e1996cb17bb430d104dd9c5b29aa571fd4b29a9c821b4fadb051d3b .
Pin and filename-scan the complete archive before extraction. INN defaults to its
ISC permissive license, with file-specific notices preserved; no third-party
source/binary is bundled in Faction. This reference uses TLS retrieval and a pinned
content digest, not an independently authenticated PGP release claim. Signing
adoption is later #106/#107.

Configure disables optional Perl/Python runtime hooks, OpenSSL, canlock and SASL.
Build only include/lib/frontends inews; no make install, setuid helper, service,
host tuning or actual board state. Upstream configure still requires a Perl build
tool. The initial NetBSD attempt lacked it; the disposable clone then used the
official perl-5.42.3 package with no dependency packages and no JS/TS filenames.
This is development tooling, not a new BBS dependency. Hash the wrapper, actual
C executable and libinn artifacts, not just a libtool shell wrapper.

## Findings and actual controls

Original code opens NNTP before testing Dump and calls hostname canonicalization.
A reserved non-listening port alone is not a portable fixture; the early native
attempt timed out during hostname setup. Force INN_HOSTNAME=fixture.invalid to
skip DNS and set a literal loopback endpoint. A temporary owned fixture sends a
400 refusal and asserts EOF with zero client commands. It cannot authenticate,
accept POST or publish an article. INN's local interface uses LF; convert physical
line endings only at that boundary and compare canonical Faction output.

Twelve checks pass on macOS/NetBSD: eight round trips, three rejections and one
explicit validation difference. Returned UTF-8 encoded subjects, 1024-byte logical
lines, final non-newline bytes, trailing whitespace/blank lines, ASCII MIME
defaults, base64 and Q subjects preserve the canonical model. INN rejects duplicate
Subject, invalid Message-ID and empty body. INN inews accepts an invalid Date;
Faction refuses it. Do not substitute INN's exit code for our own schema/parser/
authorization checks or call this a full NNTP server test.

V2 normal MIME semantics fix v1's extra decoded-newline convention; no DB/body
changes are needed. QP wire expansion and requested allocation are measured
separately in the current record. No replay/throughput/supported-capacity claim.

## Reproduction and remaining gates

Use docs/article-profile.md and tools/build_inn_reference.py / check_inn_articles.py.
Fixtures are synthetic and private, isolated from the stopped owner playground.
The current evidence pins product/tool inputs and upstream/runtime hashes; all
older source/result identities stay fixed. #101/#104 still need live independent
servers, injection, peering/provider credentials, consent/history/ACL/retention and
resource/reconnect policy. #114 owns Unicode composing/storage compatibility.

Primary references retrieved 2026-10-21: [INN inews](https://www.eyrie.org/~eagle/software/inn/docs/inews.html),
[INN source archive](https://downloads.isc.org/isc/inn/),
[TIN upstream](https://www.tin.org/),
[RFC 2045](https://www.rfc-editor.org/rfc/rfc2045),
[RFC 2047](https://www.rfc-editor.org/rfc/rfc2047),
[RFC 5322](https://www.rfc-editor.org/rfc/rfc5322).

# Discussion articles and optional news connections

Status: codec and local metadata migration implemented, 2026-10-21; network
adapters remain planned. Discussion-room
posts should use a Netnews-compatible article representation. Live IRC channels
and encrypted XMPP private chat/mail keep their respective formats. Keep the
Citadel experience: enter a room, read new posts in order, post and reply.

| Work | Issue |
| --- | --- |
| Local article model, codec and old-post migration | [#103](https://github.com/gonz0w/faction-bbs/issues/103) |
| NNTP-first policy and independent peer interoperability | [#101](https://github.com/gonz0w/faction-bbs/issues/101) |
| Optional provider subscriptions, read-only first | [#104](https://github.com/gonz0w/faction-bbs/issues/104) |

## Available offline foundation

`faction-articles` is a dependency-free Rust library with validated construction,
parse and deterministic serialization for a bounded text-only profile. It accepts
explicit identity/date/group/path metadata; it does not generate missing historical
facts, read a database or open a connection. There is a hand-authored article
fixture and an offline validation example:

```sh
cargo run -p faction-articles --example check_article -- \
  < crates/faction-articles/tests/fixtures/plain.article
```

The initial profile allows one each of the six mandatory article fields, optional
References, and explicit MIME-Version 1.0 / text/plain charset=utf-8 / 8bit headers.
Headers are ASCII; From is a bare mailbox. Date uses `DD Mon YYYY HH:MM:SS +HHMM`
without a weekday. UTF-8 bodies normalize to LF internally; the wire uses CRLF.
Encoded-word subjects, other date/header/MIME variants, unknown/control headers,
HTML, attachments and raw terminal controls are rejected rather than discarded.
This is a deliberately limited profile, not a general-purpose provider parser.

Limits: 24 KiB wire, 4 KiB headers, 16 KiB body, 998 bytes per physical line,
four groups and sixteen references. Header injection, duplicates, invalid IDs,
invalid calendar/time values and self/duplicate reply references fail validation.
Strict local duplicate/conflict comparison is implemented; persistent remote
deduplication, relay Path equivalence and conflict quarantine belong to adapters.

[ADR 0036](../dev-docs/decisions/0036-room-articles-and-reading.md) records this
increment. [Post metadata](post-metadata.md) now stores stable local article IDs,
authored/default subject labels and new-post UTC times in schema 6. The live reader
shows these fields; [local article projection](room-articles.md) now supplies
reserved author/group/Path aliases and bounded reply references through the codec.
Strict normalized ID comparison distinguishes duplicates from conflicts without
inserting remote posts. It does not publish codec output. #103 remains open for
internationalized subjects, broader MIME/date/reference policy and independent
news-software interoperability; real peering mapping is gated by #101.

## Article formatting before network transport

Use the [Netnews article format](https://www.rfc-editor.org/rfc/rfc5536) as the
interchange direction; [NNTP](https://www.rfc-editor.org/rfc/rfc3977) is the
separate network transport. Formatting a post does not publish it, open a listener
or require a news provider. SQLite remains the current store. Define a versioned,
bounded Rust model/codec and a migration before changing storage or wire behavior.
A local record/proto-article and a fully injected wire article have different
requirements; [RFC 5537](https://www.rfc-editor.org/rfc/rfc5537) covers processing.

The profile needs stable article identity, author/date/subject/group/path mapping
and reply references. Separate global Message-ID from local post IDs, per-server
article numbers, room membership and member unread positions. Preserve chronological
reading and scoped local file links. A remote From or Approved header is data,
never proof of local membership or an imported sysop role.

Legacy schema-5 posts have bodies, authors and reply links, but no original
posting timestamp or subject. Schema 6 preserves their contents, order and read cursors, labels derived
metadata and retains unknown historical posting dates. New posts collect an optional
subject; blank replies inherit one Re: prefix with a visible default. Validate old exports, restored boards, stable IDs
and interrupted migration/rollback before adoption. Do not use private rotating
session identity, real email addresses or network addresses as article identity.

Start with bounded text discussions. Specify UTF-8 body/MIME and header encoding,
line folding, CRLF, duplicate/malformed fields, reference depth and encoded-size
limits. Imported content cannot supply trusted terminal escapes, graphics, HTML
execution, local paths or automatic resource fetching. Themes render the ANSI
experience; archive/wire bodies must remain safely readable as text. Defer or
reject unsupported multipart/binary articles explicitly.

## Independent peers and external news providers

NNTP is now the first transport to validate for deliberately shared discussion
rooms. Existing news software should provide interoperability controls before
adopting a service/library. XMPP PubSub, Matrix and ActivityPub remain alternatives
if a named NNTP requirement fails; a broad comparison need not delay the bounded
local article model. See [research 0047](../dev-docs/research/0047-modern-terminal-and-federation-options.md).

An invited Faction peer and a provider reader account are different relationships.
A provider may allow reading or posting to selected groups without accepting a
server feed. Begin provider work with explicitly configured read-only subscriptions
into labeled imported-room views. No automatic mapping of a private room, old
history export or public discovery. Posting is a later disabled-by-default step
with explicit author/sysop destination consent and current local authorization.
Remote control/cancel messages cannot become privileged local operations.

Require authenticated TLS, verified server identity, protected provider credentials,
endpoint/group policy and bounded network work. Specify reconnect/backoff, expired
article gaps, deduplication/conflicting copies, cache retention and text/disk/
bandwidth quotas. A disconnected provider must not block the local board. A
subscription does not authorize contacting a vendor, buying service or publishing
content as part of research. Validate provider capabilities and policies separately
from the selected protocol.

## Local by default, explicit sharing

Each sysop retains local approval and local/private rooms. Shared rooms require
room/author/peer policy, clear destination/history/new-subscriber rules and honest
retained-copy limits. TLS protects a connection; it does not make discussion
replicas server-blind or make public articles retractable. Do not promise permanent
storage from a provider. Keep #42's deliberate permanent/public publishing separate.

Minimize metadata: author identity, group names and path/injection fields can
reveal relationships or infrastructure. Define safe local identities and the
approved outbound metadata profile before peering. No account database, password,
private file, XMPP key/plaintext or IRC channel is exported by an article adapter.
Encrypted mail/attachments remain [member-device work](private-mail.md).

## Build gates

First implement and round-trip the local supported article profile with malformed
header/terminal, old-post migration and export/restore fixtures. Then use two
independently controlled software nodes: one explicitly shared room, one private
room, denied subscriptions/posting, history boundaries, reconnect/gap/dedup/loop
controls and provider-off local operation. Measure complete adapter memory, CPU,
disk and bandwidth on NetBSD. No physical SBC prerequisite or new 250-user claim.

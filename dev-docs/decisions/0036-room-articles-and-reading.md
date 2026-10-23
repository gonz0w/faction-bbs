# 0036: Bounded room article foundation and shared message pages

Status: accepted for this local increment, 2026-10-21.
Issues: #103 and #21/#25; future transports #101/#104.

## Question and constraints

How can we start building the owner's Netnews discussion direction while making
the present BBS easier to read, without fabricating legacy metadata or enabling
publication? Keep Rust/NetBSD/no-JS, local approval, current credential/session/
thread limits, permission-checked files and recoverable SQLite data.

## Decision

Add dependency-free faction-articles as a storage/transport-independent text
profile with strict construction, bounded parsing and deterministic serialization.
Inputs provide identity/date/subject/group/path explicitly. Reject unsupported
headers/MIME, including control/authority headers, instead of silently interpreting
or stripping them. The current profile is intentionally narrower than arbitrary
Netnews. It neither resolves dates/identities nor implements injection/relay/feed,
deduplication, NNTP commands, database migration or private-message conversion.

Use one pure bounded message presenter in local/worker and shared-service readers.
Show room/message/author/reply context, wrap to text geometry, render one message
at a time and page long bodies. Only complete messages advance unread state; the
shared owner waits for complete output drain and does not commit on a failed close.
Each screen re-fetches under current approval/file permissions and retains only
cursors, not a per-member history cache. ANSI section changes use trusted theme
colors and clear the screen; plain text contains no new escape controls.

Keep valid file-reference bytes contiguous for native recent-file discovery. A
narrow terminal may soft-wrap that token; charge its full physical row cost. Other
member text is escaped before wrapping. Do not widen OSC/DCS/APC handling.

## Alternatives and tradeoffs

A full schema rewrite and provider connection now would mix missing historical
dates/subjects, identity, ACLs and outbound publication with presentation. Defer
that to explicit #103/#101/#104 gates. SQLite remains the current store. A generic
MIME/news library may help later, but needs a concrete dependency/license/NetBSD
review; the first narrow codec uses only std and rejects unsupported formats.

Keeping the old twenty-post output batch was simpler but could scroll off-screen
and mark long content read without a complete screen journey. One message at a
time adds deliberate navigation and bounded repeated queries. No additional owner
thread/process or per-member retained message/history buffer is added. This is not
a new measurement or support claim for 250 users.

## Evidence, confidence and reversal

The sources are [RFC 5536 article syntax](https://www.rfc-editor.org/rfc/rfc5536),
[RFC 5537 processing](https://www.rfc-editor.org/rfc/rfc5537) and
[RFC 3977 transport](https://www.rfc-editor.org/rfc/rfc3977), retrieved 2026-10-21;
see research 0047. The product direction is confirmed. Profile interoperability
with independent news software remains unproven. Native codec/renderer tests and
real local/shared PTY controls are the falsifiable gate for this increment; record
actual counts and source hashes in the batch evidence, not older research records.

Reversal is cheap: no schema, old post data, export, listener or wire endpoint is
changed. Old measured sources are preserved in the exact pre90 baseline snapshot
before shared UI/workspace edits. Broader metadata migration and external copies
would increase reversal cost and require separate validation/consent.

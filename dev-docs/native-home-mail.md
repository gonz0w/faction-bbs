# Native home-mail adapter contract

The default-off `faction-client/network-mail` feature connects the member-local
[v2 checkpoint backend](native-mail-checkpoints.md) to an explicit home endpoint.
It includes `native-mail` and the optional `faction-xmpp` path dependency. Ordinary
client/server operation and the v1/v2 local commands remain independent. The
[member guide](../docs/online-mail.md) describes the local workflow;
[ADR 0080](decisions/0080-native-home-mail.md) records the adoption decision.
Authority is [#209](https://github.com/gonz0w/faction-bbs/issues/209), within #90.

## Entry, configuration and authority

The entry is `faction-client --mail-online STORE PRIVATE_CONFIG [--unicode]`.
Both paths are absolute. The caller starts from a trusted local terminal; no
remote ANSI frame, BBS account or plugin can unlock this path. An explicit
[private local picker entry](../docs/picker-home-mail.md) reuses the same console
between visits; only its locally validated catalog may select the store/config.
The [parked Files entry](local-mail-read-state.md) reuses that same pair during a
completed native Files visit, with fresh local confirmation and no remote rendering
or keyboard forwarding during private mail.
Core dumps are disabled before secret input. The configuration and CA are preloaded
before connection, using descriptor-relative `NOFOLLOW` on every path component.
Each final file must be same-UID, regular, mode `0600`, single-link and nonempty;
its immediate parent must be same-UID mode `0700`. Final opens use `NONBLOCK`,
then inspect metadata, so FIFOs/devices cannot become credential inputs.

Version-1 TOML has exactly `version`, `account`, `endpoint`, `domain`, `ca` and
`pin`. The account is a restricted lowercase ASCII bare JID matching the unlocked
checkpoint's home. The numeric socket address supplies the route; its port must
be nonzero and its IP must not be unspecified or multicast. The domain matches
the account domain and supplies TLS verification/SNI and the stream destination.
The CA path is absolute. The pin is the full SHA-256 DER leaf digest, encoded as
64 lowercase hexadecimal characters. No password field or unknown extension is
accepted. Limits are 8192 configuration bytes and 65536 CA-file bytes.

The native terminal asks separately for the checkpoint secret and a hidden home
password of 1–128 printable ASCII characters. Home credentials remain
memory-only and are dropped after each cycle.
The SASL library can make ordinary memory copies; complete buffer erasure, swap
protection and protection against a compromised device/terminal are not claimed.
The home learns its account credential and public routing/key metadata, never
member private keys or decrypted mail. TLS server trust, home-account authority,
full Ed25519 peer verification and local BBS approval remain separate.

## Transport lifecycle

Reuse the lower-level tokio-xmpp 6.0.0 stream/authentication APIs with native-tls
0.2.18 and the qualified incremental rxml 0.14 guard. Connect once to the configured
numeric address. Verify the explicit CA chain, domain and exact leaf pin before
XML or credentials. Built-in trust roots are disabled. Require direct TLS with a
minimum TLS 1.2 setting, then the qualified SASL PLAIN profile and exact resource
`faction<device-id>`. No STARTTLS, DNS discovery, alternate authentication mechanism
or general Client reconnect behavior is selected. Protocol logs compile out;
`SSLKEYLOGFILE` or `TLSKEYLOGFILE` prevents connection.

One current-thread Tokio runtime drives a manual cycle. All socket operations
share the connection's original cooperative ten-second deadline. Dropping the
future/session closes the connection; local Ctrl-C/D cancels it. This bounds
cooperating async work, not OS scheduling or arbitrary filesystem delays. The
trusted private regular configuration/CA checks do not establish a hard timeout
for every possible local filesystem or TLS-backend operation.

| Boundary | Limit |
| --- | --- |
| Decrypted XML across a connection and its one authenticated stream restart | 65536 unique bytes |
| XML read interval/stanza, including conservatively charged root whitespace | 16384 bytes |
| XML depth, including stream root | 32 |
| Parser events | Reject event 256 before forwarding it to the DOM |
| Serialized outgoing stanza | 16384 bytes, checked before any write |
| Stanzas considered while awaiting one IQ response | 32 |
| Receive phase | At most 32 stanzas; two-second idle wait within the original deadline |

Standard namespaced `xml:lang` is accepted only on IQ/message roots, with an
empty reset or at most 64 ASCII letters/digits/hyphens. It is ignored: it cannot
choose terminal rendering, device trust or sender time. Other unknown or nested
language attributes remain rejected.

The parser guard precedes DOM construction and rejects DTD/entity declarations.
The DOM codec validates namespaces, attributes, text, cardinality and its own
bounded tree. It cannot detect comments or processing instructions discarded by
the qualified parser; no additional lexical-rejection claim is made.

## Public discovery and the supported OMEMO profile

Use [XEP-0384 0.9.1](https://xmpp.org/extensions/xep-0384.html)'s
`urn:xmpp:omemo:2` namespace. Device lists use PEP node
`urn:xmpp:omemo:2:devices`, item `current`; bundles use
`urn:xmpp:omemo:2:bundles`, item ID equal to the canonical decimal device ID.
Bundle children are `spk` with an ID, `spks`, full Ed25519 `ik`, and `prekeys/pk`
entries with IDs. Publication requests open access and the bundle node's `max`
item capacity through standard publish options. A server refusal is a visible
failure; the adapter does not silently reconfigure a home.

Each operation checks the local device list first. A list containing another
local-account device stops publication. Publication must use public material from
the latest durable checkpoint: an authenticated message accepted while awaiting
an IQ must not leave a consumed prekey in a later announcement. The read-only
native export returns 32 existing public prekeys from the native store, excluding
the passive session's used key. It does not generate keys or change private
snapshots. An active initiator's recorded prekey ID refers to the peer and must
not suppress an unrelated local prekey with the same number.

The codec accepts at most 32 device IDs and 25–32 distinct public prekeys, then
selects a peer prekey uniformly. It publishes 32 from the existing 100-key native
store. This satisfies the standard's minimum of 25 within a bounded profile;
it does not implement all recommendations or arbitrary-sized peer bundles.
All offered keys are validated for shape before selection. Native session start
checks cryptographic validity under the already-enrolled full Ed25519 identity.
Changed identities, new devices, legacy Curve pins and server lists never grant
trust. The caller may only start an unstarted session by explicit local action.
The first sender prepares; the recipient starts passively when accepting that
sender's authenticated message. Competing active sessions are not repaired
implicitly.

Peer discovery IQ replies must match the request ID, peer account and local
account. Only the restricted resource suffix is stripped; address case is not
rewritten. Own-home IQ responses additionally permit the configured home service
or omitted self-addressing fields before normalization. That accommodation never
applies to peer ownership checks.

Messages contain one `encrypted/header` with sender `sid`, one recipient
`keys jid` and one `key rid` with canonical `kex`, plus encrypted `payload`.
Every outgoing message includes one empty standard storage hint; it requests
retention without guaranteeing it. Incoming live messages permit that optional
hint. The serialized outer XML changes compatibly; persisted Wire, operation IDs
and ciphertext are unchanged. The key is
1–256 bytes; payload is 16–8208 bytes. Base64 is canonical. Wrong namespaces,
unknown fields, duplicate children, wrong account/device routing and oversized
values are rejected. There is no clear identity/time extension or plaintext body
fallback. Labels, multiple recipient devices, empty session-management messages,
carbons and group chats are outside this live profile. The separate
[archive adapter](native-mail-archive.md) unwraps only a matching own-home MAM2 page.

## Checkpoints, time and retry

Compose requires nonempty subject/body with bounds of 128/1024 UTF-8 bytes.
The console offers
one-line input and an empty-start multiline writer (at most 32 body lines),
inert review and an explicit `queue` confirmation. In an exact current verified
online conversation, W reuses that writer and the existing atomic local queue;
it rechecks recipient authority and preserves the group, filter, query and eligible
selection on return. It does not require the foreground exchange archive cursor,
connect, acknowledge a message or retire pending ciphertext. The existing
checkpoint transaction stores native state, sent history, a nonzero operation ID
and immutable ciphertext together. Only a successfully persisted outbox entry
can reach the transport. Retries use that entry's same ID and ciphertext.

The network codec copies sender identity from the enrolled local binding and
sets `Wire.sent_at` to zero solely as an input marker for `receive_network`.
The checkpoint decrypts on an isolated candidate, validates SCE addresses/content,
and derives the sending time from authenticated SCE. Server or arrival timestamps
are not substitutes. It commits protocol state, accepted history and the replay
receipt atomically. The existing Wire encoding and encrypted checkpoint schema
remain unchanged.

Duplicate matching considers at most the 64 retained received-history entries,
using each record's authenticated timestamp to reconstruct the canonical frame
digest. A delayed duplicate therefore does not depend on its new arrival time.
Tampering and other rejections leave canonical state unchanged. Each protocol
mutation must use current trusted local time; socket waiting does not extend the
fixed session lifetime or freeze an earlier eligibility check.

A socket write, stream response or completed cycle is not a peer/read/exactly-once
receipt. Pending entries remain until an explicit local retirement decision.
Disconnect, cancellation and ambiguous writes preserve retries. Local checkpoint
I/O ambiguity poisons the handle; reopen the canonical checkpoint before further
work. There is no automatic temp/backup fallback, rollback or session reset.

## Limits and evidence boundary

The existing one-peer, eight-outbox, 64-history and 30-day contracts remain.
Recovered/history-only, revoked, expired and full states require explicit action.
A new local device does not clear full history or remove previously published
home entries. Ordinary BBS operation remains usable when home mail is unavailable;
private content/counts never enter broad presence or sysop statistics.

The [integrated software evidence](evidence/home-mail/README.md) records actual
host/native acceptance, whole-client runtime cost and preserved failed candidates.
It is distinct from the earlier standalone transport measurements.

The [selected product inventory](dependencies/network-mail.json) records three
platform graphs, features, licenses, archive checksums and extracted-source hashes.
Run `python3 tools/check_network_mail_dependencies.py` with the selected cached
sources to verify it offline; `--write` is deliberate reviewed regeneration.
The [earlier transport evidence](evidence/xmpp-transport/README.md) covers the
research transport and synthetic payload, not acceptance of this integrated
mailbox. Its 2 GiB fresh-build qualification and 1 GiB runtime observations must
not be represented as a fresh 1 GiB compiler or product-capacity result.

[#90](https://github.com/gonz0w/faction-bbs/issues/90) remains open for the remaining
mailbox, trust and recovery requirements. [#211](https://github.com/gonz0w/faction-bbs/issues/211) scopes durable manual catch-up.
The opt-in [archive adapter](native-mail-archive.md) adds one bounded MAM/RSM
page and an encrypted atomic cursor. Archive-enabled U/K/N omit initial presence
and reject live encrypted frames before checkpoint mutation; N returns after
persisted writes. A is the explicit archive receive path. Schema 2 keeps live
synchronization. Neither mode promises background notification/reconnect,
attachments, roaming or multi-device operation. The owner-approved separate
foreground attempt in [ADR 0081](decisions/0081-durable-mail-archive.md#owner-approved-amendment--2026-11-07)
does not change those actions. A NetBSD software VM exercised the original
foreground and same-SSH paths before the separate #358 provider change.
Source checks and the prior qualified primitives do not establish a
security audit, full protocol conformance, physical hardware support or capacity.

## Bounded foreground contract

The accepted separate attempt requires a warm prepared session, an archive-enabled
vault with a saved nonempty cursor, and the current independently verified peer.
Cold, empty-cursor, history-only, revoked, expired or full states reject before
connection or mutation. The reviewed peer context must be rechecked before release
or acceptance. A fresh hidden home password authorizes each single connection:
receive briefly, or send exactly one selected retained ordinary-text pending
operation after fresh peer discovery, then receive briefly. Attachment releases
and bulk outbox sends are excluded. Cancellation closes the session; a local write
is not a delivery receipt and no arrival does not establish that the peer is offline.

The conversation review exposes E, pages the current home, peer, device
and identity, and offers R or B; S is conditional on the highlighted eligible
pending text and reviews its retained record, operation and subject. Q opens a
bounded picker of eligible canonical pending ordinary texts for the exact current
peer independently of the conversation filter and query. Explicit selection
uses the same review and confirmation path for one operation; an empty or stale
selection sends nothing. Returning preserves the conversation view. The existing
full-store refusal applies to E even after W queues the eighth pending operation.
A literal `exchange` confirmation precedes the fresh hidden password; blank input
returns without connecting. The original R and S classic and Unicode review, cancellation
and dispatch controls have been tested. Classic and Unicode two-client host tests
with synthetic TLS homes exercised production selected send, live receive and later MAM
recovery/deduplication while preserving cursor, read state and pending operations.
Classic and Unicode host and NetBSD 11 ARM64 software VM journeys exercised the
conversation writer and queued-text picker, including cancellation, exact selected
wire, authenticated receipt, retained view and state, privacy and teardown.
NetBSD checks also covered same-SSH BBS/SFTP outage and return at 1 GiB runtime;
independent native evidence review accepted source `c3c6371`. The later [#362](https://github.com/gonz0w/faction-bbs/issues/362)
provider test-only coordinator repair is separate host/CI qualification work and
was not included in the guest inputs.
The original product source `81922457`, with the `f01977a` test-only CA fixture
repair, also completed earlier NetBSD software VM and same-SSH controls.

Full outer peer routing derives from the verified account and device. The OMEMO
bare account, operation ID and ciphertext remain immutable. Only this new path may
admit one qualified own stanza-id: the encrypted element, empty store hint and SID
element occur in the exact qualified child order; SID has exactly `by` and `id`,
`by` equals the own bare account, and `id` is 1–256 ASCII graphic bytes. The SID
element is empty. Foreign, duplicate, reordered, oversized or otherwise malformed
metadata is rejected and grants no cursor, time, trust or receipt meaning. Existing
decoders and modes remain strict.

Authenticated input uses the existing atomic `receive_network` vault boundary,
preserving cursor, read state and pending work. Later deliberate archive retrieval
can recover older retained ciphertext and deduplicate it, subject to the existing
32 skipped-key bound and rejection at 33. The original cooperative ten-second
connection/session deadline, two-second receive idle wait, 32-stanza limit, XML
16 KiB interval/64 KiB stream, depth 32 and rejection at parser event 256 remain.
No presence, automatic archive query, polling, reconnect or background lease is
added. This contract is policy for #355/#356, not evidence of qualified behavior.

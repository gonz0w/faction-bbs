# Native manual archive contract

The default-off `network-mail` feature adds one explicit own-home archive page to
the [native home-mail adapter](native-home-mail.md). It uses the existing verified
TLS/account connection and native OMEMO/SCE checkpoint. No server obtains a private
device key, local unlock secret or decrypted subject/body. Home authentication
and archive metadata do not enroll a peer or grant BBS authority.

The member interface is [Catch up retained encrypted mail](../docs/archive-mail.md).
[ADR 0081](decisions/0081-durable-mail-archive.md) records the choice; implementation
and acceptance authority is [#211](https://github.com/gonz0w/faction-bbs/issues/211).

## Explicit encrypted format transition

The outer `FACTVLT2` encryption, password/recovery wrappers, keys and v2 associated
data stay unchanged. The inner `FACTCP02` encoding remains byte-compatible for
stores that have not opted in. `enable_archive(now)` explicitly writes `FACTCP03`;
unlock decodes either version and never migrates implicitly. Older implementations
cannot decode the new inner version. There is no downgrade API.

The new inner suffix contains the archive home, local device ID, random nonzero
16-byte epoch, and optional opaque `after` ID. All are inside the encrypted
checkpoint. Decode requires that home and device match the containing state.
An archive ID is 1–256 ASCII graphic bytes without whitespace. It is not a number,
timestamp, trusted chronology or recipient identity.

`ArchiveCheckpoint { home, local_device, epoch, after }` is a compare-and-swap
token. `archive_checkpoint(now)` returns it only for an enabled, healthy active
store with an unexpired, unrevoked verified peer, fewer than 64 history records
and fewer than eight pending operations. Those gates also apply to duplicates
and own sent echoes. Read-only inspection exposes enabled/cursor-present booleans
without printing opaque positions in the member console.

Enabling preserves native key/ratchet bytes, existing history, peer trust, durable
operation IDs, ciphertext and recovery data. Repeating it on a healthy enabled
store returns without resetting the epoch or position. Official backups and
recovery retain the position in history-only state, where it cannot resume native
keys. Local password/recovery rotation preserves that state. Explicit fresh-device
creation, after pending retirement, keeps retained history and schema 03 but starts
a new device/epoch with no `after`; peer verification must be repeated.

## Atomic authenticated entry and position

`receive_archive(expected, id, wire, now)` requires the exact current token,
including scope, epoch and prior position. It rejects `id == after` and a nonzero
transport-supplied `wire.sent_at`. Incoming peer ciphertext follows the existing
native authentication path; authenticated SCE supplies the message time. One
candidate checkpoint contains native state, accepted history and the new position,
and the existing atomic file publication commits them together.

A previously accepted ciphertext can occur under a later archive ID. The bounded
received-history lookup reconstructs each candidate frame using its authenticated
stored time. A match advances only the position, without another decryption,
native-key transition or history row.

An own sent echo must match the current own-to-enrolled-peer address/device/identity
binding and the immutable frame digest of a retained `Sent` record, reconstructed
using that record's authenticated send time. Matching remains possible after an
explicit outbox retirement. Unknown own ciphertext fails; the archive cannot
cause a new encryption, trust decision or delivery acknowledgement.

A file-write failure keeps the existing poisoned-handle rule: drop the handle and
reopen the canonical checkpoint. The archive path adds no temporary-file fallback,
lock removal, cursor-only side file or automatic rollback. Process death leaves
whole checkpoints rather than a new cursor paired with old native state.

## Selected archive protocol

The codec uses the MAM2 profile in
[XEP-0313 1.1.3](https://xmpp.org/extensions/attic/xep-0313-1.1.3.html),
[RSM, XEP-0059 1.0](https://xmpp.org/extensions/xep-0059.html),
[forwarding, XEP-0297 1.0](https://xmpp.org/extensions/xep-0297.html), and
[delay metadata, XEP-0203 2.0](https://xmpp.org/extensions/xep-0203.html).
These references explain the wire elements; the selected implementation is narrower
than general XMPP/MAM conformance.

Each cycle generates separate random IQ and query IDs. It addresses the query to
the configured own bare account, submits the MAM2 form with `with` equal to the
independently verified peer, and requests RSM `max` 4 plus the saved `after` when
present. It does not discover arbitrary archive services, query another account,
accept group archives or negotiate broader profiles.

Returned wrappers must belong to that query and own archive, address the local
bare/full account, and contain exactly the supported forwarded delay and encrypted
chat message. An omitted outer sender means the own archive; a present sender must
be the exact own bare account. Strict inner OMEMO bindings distinguish incoming
peer messages from own sent echoes. Unknown fields, unexpected children, duplicate
IDs, a repeated current `after`, and mismatched address/device/query metadata are
rejected. Bounded standard `xml:lang` is inert; it does not change display mode,
identity, trust or time.

The UTC delay syntax is checked but never supplies authenticated send time or
message order. RSM IDs are opaque; the client cannot detect arbitrary server
omission or reordering. The finishing IQ must match this query, page endpoints
and supported RSM shape. `stable='false'` is rejected; omitted `stable` or true is
accepted. The server's `complete` flag is reported as its claim, not proof that
all conversation history exists locally. Optional count/index values confer no
trust. A matching own-archive `item-not-found` for an existing cursor becomes an
explicit unavailable-position error; it does not reset or skip.

The network buffers at most four entries and validates the complete wrapper/fin
sequence before the first local commit. Thus malformed wrappers, wrong fin,
protocol errors and timeout before fin leave the checkpoint unchanged. After fin,
authentication and atomic publication occur per entry. If entry two fails, entry
one can remain the last durable accepted position. The next explicit request uses
that position; there is no whole-page rollback or automatic page loop.

Existing transport limits remain: 65,536 input bytes per stream lifetime, 16,384
per bounded interval, depth 32 and a 256-event parser boundary. Archive DOM checks
also bound accumulated element/attribute/text bytes, depth, nodes and attributes. A cycle consumes at most
32 stanzas, with a two-second next-stanza wait inside the existing cooperative
ten-second deadline. Presence consumes the bounded loop without granting trust.

## Receive mode and storage requests

The online **G** action confirms `upgrade` locally. **A** preflights the active
checkpoint before connecting, asks for the home password through hidden input,
and runs one archive query. It publishes no keys, announces no presence and retries
no pending sends. Cancellation and terminal restoration retain the existing console
contract. Opaque archive IDs and epochs are not member-facing diagnostics.

For schema 03, **U**, **K** and **N** also omit presence. **N** exits after writing
pending ciphertext; it does not enter live receive. Unsolicited live mail during
these actions is rejected rather than changing native state outside the archive
position. Schema 02 retains its existing live synchronization behavior.

Every outgoing encrypted message, including a retry from schema 02, now includes
exactly one empty `<store xmlns='urn:xmpp:hints'/>`, following
[XEP-0334](https://xmpp.org/extensions/xep-0334.html). This is a retention request
for bodyless encrypted messages. It does not alter persisted Wire bytes,
operation IDs, ciphertext or trust, and does not guarantee that any home stores
or forwards the message. No plaintext fallback is introduced.

## Verification and limits

Storage tests cover explicit migration, scope/epoch compare-and-swap, known sent
echoes, duplicates, limits, recovery quarantine and process death before/after
publication for fresh receive, duplicate and own-echo commits. Protocol/network
tests cover strict wrapper/fin matching, partial authenticated application,
timeouts, stale position, metadata substitution and the four-entry boundary.
Console tests exercise deliberate upgrade, refusal and cancellation separately.

The fixture tools distinguish actual two-home offline retrieval, live child death
after an observed checkpoint publication, and separate scans of both homes for
private content markers. A live child kill does not identify an exact fsync point;
that boundary is exercised separately by deterministic storage tests. A local
client run cannot claim it scanned inaccessible remote server files.

The [evidence validator](../tools/check_archive_mail_evidence.py) requires both
host and native routes, source-matched execution stages, separate server proofs,
compatibility/default-off controls and complete teardown before accepting a pinned
record. Its synthetic tests are not native measurements. Optional cost evidence
must identify the entire observed archive UI process. A 2 GiB complete builder and
1 GiB runtime are distinct conditions; neither establishes a fresh complete
1 GiB build, physical-board suitability or supported capacity.

The [pre-archive baseline](evidence/home-mail-archive-baseline-manifest.json)
preserves executed sources and unchanged prior records, including the original
home-mail result. Current code does not inherit their measurements. The selected
Rust dependency graph, native crypto/vendor code, existing resource limits and
server authority remain unchanged by the archive implementation.

No complete-history guarantee, read receipt, general archive reorder detection,
background notification, roaming, attachment support, full protocol conformance
or security audit is claimed. Parent [#90](https://github.com/gonz0w/faction-bbs/issues/90)
remains broader than this manual page contract.

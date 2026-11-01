# Durable attachment messages in native mail

The optional `native-mail` library supports an explicitly enabled encrypted checkpoint containing one attachment descriptor per mail history record. `native-mail` alone provides local checkpoint, queue, receive and consumption APIs. Home-network delivery of the encrypted attachment **message** uses the existing `network-mail` feature and adapter. The separate [private storage and transfer APIs](private-attachment-storage.md) persist and transfer bounded ciphertext through deliberate cycles; local verification and saving remain explicit. See [#257](https://github.com/gonz0w/faction-bbs/issues/257), [#258](https://github.com/gonz0w/faction-bbs/issues/258), and the still-open parent [#88](https://github.com/gonz0w/faction-bbs/issues/88).

## Opt in and send

`CheckpointVault::enable_attachments` explicitly migrates a checkpoint to `FACTCP05` inside the unchanged `FACTVLT2` envelope; `attachments_enabled` inspects the choice. Unlock and ordinary mail calls do not silently migrate `FACTCP02`, `FACTCP03` or `FACTCP04`, which retain their earlier attachment rejection. The migration retains existing history, pending ciphertext, verified peer and native state, and independently present archive and read state. It does not enroll a peer, restart a session, acknowledge mail or mark it read. The [checkpoint contract](native-mail-checkpoints.md) describes the envelope and publication rules.

A trusted local caller may prepare ciphertext and a valid [descriptor](attachment-metadata.md) separately, or use the automatic private transfer path to encrypt and durably spool the selected bytes before queueing. `queue_attachment(operation, subject, body, &descriptor, sent_at, now)` validates the descriptor and usable blob limit before advancing native state. The encrypted checkpoint contains the descriptor, including its private key, IV, hash, name and locator, with the history record. `Record::attachment` borrows it for trusted local use. Bulk ciphertext is not stored in the checkpoint.

The attachment intent binds the verified recipient, original sending time, subject, body and every descriptor field using a distinct deterministic domain. An identical pending operation returns the exact persisted `Wire` after reopen; changing the intent rejects, and an acknowledged operation cannot reseal. The existing `network-mail` home adapter uses that persisted wire and the existing explicit acknowledgement decision. The [checkpoint contract](native-mail-checkpoints.md) describes retry and failure semantics.

## Receive and consume

Live, network and archive receipt share the authenticated transaction. It preserves the existing verified-peer and time checks, duplicate behavior and, for archive pages, cursor handling. The descriptor is accepted independently of whether the bulk ciphertext is available. The caller can inspect the descriptor only from retained encrypted history.

`mail_attachments::verify_received` requires an attachment-enabled healthy vault, a retained received-record index, its expected frame identity and explicitly supplied ciphertext. It obtains the actual stored descriptor and verifies the bytes with the existing blob primitive. `save_received` performs the same verification before the existing safe save to an explicit absolute destination selected by the caller. Cancellation means making no save call. Its fixed diagnostics preserve the distinction between `BeforePublication` and `PublicationUncertain`: after uncertain publication inspect the destination before retrying. See [blob and save](attachment-blobs-save.md) for limits and filesystem assumptions.

Accepted historical attachments remain locally consumable after session expiry, revocation, history-only backup/recovery or a fresh device change. Those states do not resume the old live protocol. The private locator identifies ciphertext but does not authorize a connection. The separate transfer API fetches only from an authenticated retained record through an explicitly selected invited-home route; existing HTTPS locators remain inert. The default-off `network-mail` console provides an explicit [private attachment desk](../docs/private-attachments.md) using the managed transfer path. These underlying APIs do not themselves provide a CLI action or public HTTP endpoint.

The existing 512 KiB checkpoint, 64 history records, eight pending messages, 8192-byte SCE, 8208-byte payload and 1 MiB plaintext blob limits remain. The bounded transfer and store have their own limits and external authorization boundary. The separate [NetBSD software qualification](evidence/netbsd-attachments/README.md) covers the native libraries, transfer and production console in one guest. It does not establish independent modern-client interoperability or a full attachment product; the original requirements of #88 remain open. Ordinary classic access and text mail remain available.

## Explicit own-home carrier

The default-off own-home provider selection uses a private version 2 profile with
`route = "own-home"`. Route selection occurs before a socket: both local principals
use the existing binary Session, local sender/foreign recipient uses publication,
and local recipient/foreign sender uses retrieval. A wrong local full principal,
neither principal local, or an unsupported action is refused. The full unlocked
vault identity and verified peer snapshot remain required. The configured home is
the only endpoint; errors and Busy never select IQ or another endpoint.

The existing worker's eight SDK requests per ten-second cycle remain in force.
Each carrier operation stays within the worker's original deadline and cannot renew
or extend it. Its eight transport attempts are distinct from the worker's SDK
request limit. Submit, close and paced collection share that attempt and deadline
bound; each `collect_paced` waits within the budget and the client makes at most six fresh collects. Only a fresh authenticated `Response::Progress { binding, offset,
committed }` with the exact binding, complete offset and `committed = true`
releases mail. A receipt, Busy, EOF or timeout cannot release it; retry the exact
persisted operation with fresh authority. The recipient retains normal transfer
confirmation, sees the sender home under separately preapproved policy, and has
no client invitation authority or legacy IQ invitation prompt on this route.

Host and fresh NetBSD maintained-client exercises completed maximum 1 MiB classic
and Unicode journeys. The NetBSD run used 1 GiB RAM, two CPUs and the original
60-second budget, with 65 upload and 65 fetch calls for 1,048,592 ciphertext
bytes. Its build temporarily required 2 GiB for the compiler; 1 GiB was restored
before qualification. Homes had separate runtime, configuration, keys, stores
and approved policy; one opaque test XMPP delivery/archive queue served both.
Fresh native provider and client suites passed 357 checks (including 38 doc tests)
and 266 tests respectively; eight inherited client helper tests were ignored. These fixtures do not establish
deployed S2S, provider adoption or independent client interoperability. Public
evidence is recorded in the [own-home software evidence](evidence/own-home-attachments/README.md).

In a host interruption exercise, an exact persisted upload retried after source
removal and same-policy restart committed, released mail and reached an offline
recipient. A later fetch stopped at 32 KiB; that original episode failed and is
unexplained. A separate fresh fetch resumed partial state, first confirmed at
61,440 bytes, and completed 131,088 ciphertext bytes in six calls; verification
produced 131,072 plaintext bytes and a new private `0600` save without overwrite.
This is not uninterrupted completion, automatic retry or a fix claim. Ordinary
text mail passed an outage exercise. A separate native software exercise passed
eight BBS/SFTP controls while both homes were unavailable: independent registration
and approval, preserved encrypted state, and a 131,328-byte exact public SFTP
transfer with privacy and terminal restoration checks.
Existing historical evidence is unchanged.

## Failure attribution

For the own-home route, a stopped foreground upload or fetch emits one bounded
`Attachment diagnostic v1` line after the existing worker drop/join and before
the original stop message. Success output and stop markers remain unchanged;
there is no automatic retry. The line contains fixed enums, numbers and the
static value `unavailable`, never member names, account IDs, paths, URLs,
filenames, keys, tokens, payloads or debug-formatted errors. See the
[member recovery guidance](../docs/private-attachments.md) before retrying.

The stop origin names the selecting branch (`foreground_deadline`,
`sdk_deadline`, `cancel_branch`, `sdk_error` or `progress_bound`), not a
historical root cause. The transport stage is separate from the last phase.
The latter does not prove I/O is still pending; connect calls count entries,
not completed connections. SDK cycles, logical requests, carrier attempts and
connect calls are distinct. `cycle_*` values describe the current available
snapshot; `observed_*` totals sum only available cycle samples. Interpret them
with `unavailable_cycles`, `aggregate_valid` and `totals_clamped`: unavailable
is not zero and partial sums are not complete wall time. Durations are bounded
u64 microseconds. Connect includes TCP/TLS; exchange still includes unknown
server callback, Store and scheduling time. Planned and actual wait are distinct,
and overlapping timings must not be summed as equivalent elapsed time.

An optional caller-owned `OnceLock` retains a by-value final snapshot through the
existing worker join after timeout or cancellation; initialization failure or a
join panic is explicitly unavailable. This adds no thread, queue, live polling,
Runtime, protocol or authority cache. The retained diagnostic state contract is
at most 512 bytes, separate from the 1024-byte output bound. A focused host size
check observed 496 bytes on macOS ARM64; a fresh NetBSD size check was not run.
The observer supplies classic or Unicode context separately for both legacy and
own-home callers; that mode is not in the Rust line. Its strict parser accepts
the defined ordered schema only. These observations do not establish a
performance fix or the cause of [#326](https://github.com/gonz0w/faction-bbs/issues/326);
the diagnostic work is tracked by [#329](https://github.com/gonz0w/faction-bbs/issues/329).

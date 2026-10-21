# Local file areas with interchangeable transfer adapters

Status: original design complete; implemented under #83 (see plan 0033 and ADR 0032). Date/retrieval: 2026-10-21.
Owner: project founder. #22; implementation #83. Decision: ADR 0030.
Stopping condition: define area semantics, SFTP boundary, quotas, recovery and
negative tests without claiming an installed transfer service.

## Scope, gates and alternatives

Owner selected **a separate restricted SFTP service first**, classic in-session
transfers later. Keep the Rust/NetBSD BBS and ordinary terminal-only SSH service
unchanged. No JS/TS, public HTTP requirement, automatic file federation, member
shell/exec or credential/process-limit relaxation. Files must remain useful
independently of the new native-client/encrypted-mail research.

| Alternative | Terminal UX / reuse | Authorization / resource tradeoff | Decision |
| --- | --- | --- | --- |
| Separate SFTP service using base OpenSSH | Existing CLI/GUI clients; encrypted transport, offset-based transfer operations | Chroot/request filters do not implement BBS grants/revocation/quotas. Need scoped admission plus mediated file operations | Owner-selected first transport; concrete handler/admission design needs experiment |
| Separate native SFTP handler under OpenSSH | Can present a virtual catalog and check each operation; reuse SSH crypto | More protocol implementation/testing; select maintained no-JS dependencies and prove malformed-input bounds | Compare against base-service reuse, not yet selected |
| ZMODEM-style in-session adapter | Strong classic BBS feel | Current safe terminal codecs cannot pass arbitrary binary data; requires a distinct bounded transfer state and client support | Later adapter, not a reason to weaken terminal filtering |
| Public/optional HTTP portal | Convenient links | Adds an access surface and possible privacy/dependency burden | No first-iteration dependency; no-JS and local approval remain mandatory |

No numeric winner: owner chooses transport UX; runtime/admission/NetBSD cost and
complete failure behavior are unmeasured. Source documentation passes availability
screen, not Faction security/integration gates.

## Proposed data and behavior contract

Local area: immutable ID/slug, printable title/description, visibility policy,
upload/review policy and byte/count limits. Local object: random generated ID,
area, uploader reference, safe display filename, short description, byte length,
SHA256 and review/publication status. Names never become host paths. Approved
members see only permitted areas; private grants and current member epoch are
checked for listing, details, grant creation and ongoing transfer work.

Grant: server-generated operation ID bound to a transfer-client SSH public-key
fingerprint, local member/epoch, direction, area/object or reserved upload, byte
ceiling, expiry and active-consumption state. The private key stays on the client.
Bind any future home identity to a local approved membership before minting it.
Separate SSH signature verification, transfer grant authorization and OS ownership.
A raw public key, invitation, SFTP connection or remote XMPP claim is insufficient.

The file service gets its own UID/storage and bounded IPC authority. It never
opens the BBS SQLite database or receives passwords, room content, mail keys or
member TTYs. The current BBS admission helper UID is not reusable. Determine
short-lived scoped admission and periodic/chunk authorization before implementation;
base chroot/read-only mode alone is insufficient for per-area or live revocation.

Uploads reserve quota before data, use a private no-follow create-new staging
object, stream bounded chunks and verify length/hash. On finalize, fsync the
complete immutable object and publish catalog state transactionally; define
reconciliation for a crash between filesystem and catalog commits. Pending,
interrupted and rejected objects never become downloadable. Existing files are
never overwritten. Retained partials remain charged, expire on a bounded policy
and need a valid grant/offset to resume. No archive extraction or execution.

Initial **experiment inputs**, not defaults/support promises: two active transfer
streams, 64 KiB data chunks, 16 MiB per upload, 256 MiB board object quota, two
pending uploads/member, 64-byte ASCII display names, 512-byte descriptions,
20-row pages and five-minute grants. Choose final configurable policies using
real transfer tests. Quotas are disk/accounting bounds, not RAM allocations.
Record actual peak/idle host costs separately. No 250-user experiment required.

## Backup, privacy and operation

Persist both catalog and immutable objects with a consistent manifest/epoch;
restore into a fresh private destination and verify every referenced object.
Current faction database exports contain no file-area blobs/catalog. Define a
separate complete file-data bundle; do not label a database-only copy complete.
Retention/review/removal belongs to the sysop; a downloaded copy cannot be recalled.
Record bounded transfer outcomes, never file bodies/tokens by default. Do not
claim malware scanning or quarantine review makes arbitrary content safe to run.

## Falsifiable implementation checks

1. Real SFTP client upload -> pending review -> publish -> other approved member
   download -> exact content/hash equality; terminal catalog and return work.
2. Pending/revoked membership, wrong board/area/key/direction/object, expired or
   replayed grant and removed private-area access deny with no content disclosure.
3. Traversal, absolute paths, dotfiles, control/ANSI filenames, symlinks/hardlinks,
   rename/overwrite, device/FIFO targets and unbounded protocol frames fail closed.
4. Concurrent quota reservations cannot overspend; partial/oversize uploads remain
   invisible and charged until bounded cleanup. Partial/finalization restart is
   reconciled without publishing truncated data or deleting live transfer locks.
5. Revocation during transfer stops further authorized chunks within a measured
   bound; a transfer helper failure leaves BBS rooms/chat running. Existing BBS
   shell/exec/SFTP/forwarding denials and process/credential ceilings remain.
6. A separate complete file-data backup/restore recovers metadata plus exact bytes;
   a missing/corrupt object fails verification. No live user data in public evidence.

## Primary evidence and confidence

Retrieved 2026-10-21: NetBSD [sftp-server(8)](https://man.netbsd.org/sftp-server.8)
documents request allow/deny lists, read-only mode and start directory;
[sshd_config(5)](https://man.netbsd.org/sshd_config.5) documents chroot, forced
commands and subsystem settings. Online pages describe NetBSD-current and are
not proof that every option exists in the deployed NetBSD 11 binary; pin and
query actual VM versions during #83. Owner choice is confirmed; all proposed
runtime/resource/authorization behavior above remains unmeasured. No production
handler, new listening service, dependency or native support claim is adopted.

Reversal cost: a transport adapter can be replaced without changing immutable
object IDs, local catalog/approval policy or backup format. Adding another
adapter must not broaden an existing grant. #83 owns implementation and measured
results; #22 is the bounded design deliverable, not the file-sharing release.

# Native terminal transfers and file links

Status: implemented and software-validated; PR review pending. Owner authorized merging #86 and continuing
#84 with integrated native-client transfers, room/chat links and planned encrypted
mail attachments. #84 remains open for actual two-home E2EE/identity research.

## Scope and boundary

Reuse base OpenSSH for BBS/transfer crypto. A separate local Rust client owns a
PTY, strict profile/known-host policy, terminal-control filtering and its own Files
menu. Approved BBS login/art/menus remain. Only Files negotiates a bounded versioned
control interface; public key/nonce/operation requests travel over the same BBS
session. The local client selects paths and confirms transfers. SFTP runs alongside
the still-open BBS connection under the existing independent file service.

No remote-selected host, shell, local path or automatic transfer from terminal
output. No credentials/device-mail keys pass through file control. The owner gains
no thread/process or credential limit. No mandatory new daemon/listener or JS/TS.

Schema 5 adds a stable **public file namespace**, separate from the private rotating
session identity. Local room/chat tokens reference namespace plus immutable object;
sharing validates published/current access, reading and opening recheck permissions.
A link grants no access and does not copy/federate data. Restore preserves namespace
and tokens while rotating session identity and discarding transfer authority.

## Work

1. Merge #86 after exact-head checks/reviews; freeze its 74 measured inputs and
   complete original result before any core/service edits (sources/pre84).
2. Implement file references, migration/export compatibility, send/read scope tests
   and Files link commands; keep ordinary clients usable.
3. Add bounded native Files control and a local Rust terminal client with OpenSSH,
   a client-only PTY bridge, integrated upload/download/progress/cancel, no-clobber
   local files and terminal restoration. Keep remote output/local action separate.
4. Test real host and NetBSD SSH/SFTP, file links/permissions/revocation/restore,
   malformed frames/terminal controls, local path ownership/aliases, wrong host key,
   failure/cancel and return to BBS. Software VMs only; no 250-user/hardware claim.
5. Update README, docs/dev-docs/ADR/backlog; open a focused reviewed/check-green PR.

## Encrypted mail attachments

Keep a separately scoped child of #84/#33 for member-side encrypted blobs. Encrypt
before upload; keys/private filename metadata belong inside E2EE envelopes. Current
sysop-readable file areas cannot provide server-blind attachments. Compare actual
XMPP sharing/encryption profiles and implementations; no mail crypto/protocol/SSO
adoption or server-readable fallback in this client increment.

## Outcomes

- Squash-merged #86 at `c85dbd1d60e9c8bb8646ed937f2ee4973e58f5f1` after
  exact-head green CI and no outstanding review conversations. Preserved 74 exact
  executed inputs and the original complete file-area result before edits.
- Created #87 for this client/reference deliverable and #88 for device-encrypted
  attachments; #84 remains open for actual two-home XMPP/E2EE/device/auth research.
- Implemented a member-device Rust client with a client-only base-libutil PTY bridge,
  strict OpenSSH profile/known hosts, bounded terminal filter and Files control.
  Upload/download/progress/cancel and recent room/chat link selection keep the
  terminal session connected. Native Files has a fixed local palette and ASCII
  path forms; no mail keystore/XMPP/roaming login or general plugin runtime.
- Added schema-5 public namespace/references, per-viewer metadata redaction and
  current open/download checks; full restore retains public links and rotates
  private authority. Ordinary SSH/manual SFTP remain available.
- Real macOS and NetBSD-native clients passed ten grouped integration checks,
  including changed host key, upload/review/download, no-overwrite/cancel/retry,
  scoped room/chat/recent links, revocation and complete stopped/drained schema-5
  catalog/blob backup/restore. Payload 131,328 bytes with matching SHA256.
- Current workspace passed 100 Rust tests on macOS and native NetBSD 11 ARM64;
  host clippy and six maintained terminal/lifecycle/credential smoke tools passed.
  Python suite passes 101 tests; existing dependency inventories remain unchanged.
  Six frozen snapshots/eighteen historical results remain byte-for-byte protected.
- The two-core/1-GiB software VM, loopback forwards, keys, synthetic files and raw
  transcripts were removed after service shutdown. No physical test, supported
  capacity, peak memory, cryptographic mail or independent security-audit claim.

See [sanitized results](../evidence/native-client-netbsd/results.json),
[member setup](../../docs/native-client.md), [control seam](../native-file-control.md)
and [ADR 0033](../decisions/0033-native-terminal-files.md). Keep broader #84 open;
new PR closes only #87. No final merge is implied by this implementation request.

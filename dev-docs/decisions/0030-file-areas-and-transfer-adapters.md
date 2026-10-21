# 0030: Local file areas with separate transfer adapters

Status: accepted owner direction; handler implementation follows ADR 0032.
Date: 2026-10-21. Authority: owner's explicit SFTP-first clarification.
Issues #22, #83, #5. Research: [0043](../research/0043-file-areas-and-sftp.md).

Keep area/catalog/approval policy independent of byte transfer. Use a separate
restricted SFTP service first, then allow classic in-session transfer adapters
without replacing storage or authorization. Keep the BBS SSH listener terminal-only.
No public HTTP dependency, file federation, member-selected host path or backend
process/thread/credential relaxation. A native client may later integrate the
transfer UX, but local file work does not wait for federated identity/mail.

An extension gets a scoped operation, never general database/filesystem access.
Grants bind member/epoch, area/object, direction, bytes, lifetime and client key;
recheck current authorization during use. Separate storage UID, private quota-
charged quarantine, review and immutable publication are required. File backup
includes catalog plus objects; current database export is not a file-area backup.

Source docs confirm OpenSSH has useful SFTP/chroot/request filters. They do not
pass integration gates for live BBS revocation, safe names/paths, concurrent quotas,
crash consistency or NetBSD footprint. Compare base-service reuse with a bounded
native SFTP handler. Unknown gates block production adoption of either handler,
not implementation experiments. Binary passthrough inside existing safe terminal
codecs and public HTTP-first delivery are rejected for this first transport.

#83 owns actual upload/review/download, authorization/path/quota/interruption,
backup/recovery and resource observations. Experiment bounds in 0043 are inputs,
not released limits. Transport reversal cost is low with opaque objects and
scoped grants; published object and backup formats require deliberate migration.

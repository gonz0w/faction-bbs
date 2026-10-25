# Current architecture contracts

Faction is a Rust terminal BBS with primary NetBSD software validation, independent
local approval and classic/plain access. It is a hobby preview, not a supported
hardened release. [GitHub Issues](https://github.com/gonz0w/faction-bbs/issues) own
all unfinished work; this document describes component boundaries, not a roadmap.

## Community, presentation and access

`faction-core` owns the local SQLite/WAL data model, current member/sysop roles,
approval/epochs, rooms/unread/replies, channel/file grants and community actions.
The current schema is 18; migrations and stopped complete export/verify/restore
must preserve explicit version contracts. Connection-bound sessions cannot import
caller-supplied identity, role or owner authority. Owner recovery stays outside
member RPC. See [privacy/security](../docs/security-privacy.md),
[export/restore](../docs/export-restore.md) and the core's schema/tests.

`faction-terminal` provides inert bounded presentation, validated sysop themes,
ANSI/ASCII/optional modern capability paths, menu action IDs and text/input codecs.
It holds no account/store/network authority. Local CLI, inherited worker and shared
service adapters route actual actions back through current core checks. Displayed
keys are presentation; hiding an item is not authorization. See
[themes](../docs/theme-packs.md), [capabilities](../docs/terminal-capabilities.md)
and [shared UI](../docs/shared-ui.md).

Base OpenSSH handles member transport. The restricted native frontend has no board
DB access and passes an authenticated owned TTY to the trusted owner. The normal
shared owner uses fixed credential helpers and bounded readiness/output work;
worker rollback remains available. See [restricted SSH](../docs/restricted-ssh.md),
[shared service](../docs/shared-service.md), [process envelope](../docs/process-confinement.md)
and [lifecycle](../docs/service-lifecycle.md). Experimental isolation/admission
profiles have narrower measured claims, not blanket production confinement.

## Optional features and native client

Rooms/community desks/files remain locally authorized. Discussion articles have
a bounded Netnews codec; formatting does not publish or federate a room. Local
slash-command chat is not IRC wire interoperability. Broad Who activity excludes
private resource names; Who, group `/who` and Lounge share visibility/availability.

SFTP transfers use a separate restricted service and fresh scoped file authority.
Room/chat file references confer no new permission. Lua doors and original-game
connectors use separate hosts/resources; they cannot inherit DB, recovery or member
private-key authority. See [files](../docs/file-areas.md), [Lua](../docs/lua-doors.md)
and [classic doors](../docs/classic-game-doors.md).

The optional native terminal client integrates deliberate transfers, bounded
capability-gated local graphics, independent-board profiles and local mail vaults.
Device keys/decryption/private mail stay on the device. An encrypted local vault or
two-home reference proof is not a production native XMPP/OMEMO network adapter.
Local plaintext passport souvenirs/notes are separate from private mail. See
[native client](../docs/native-client.md), [private mail](../docs/private-mail.md),
[local vault](../docs/local-mail-vault.md) and [extension contracts](extension-contracts.md).

## Invariants and evidence

Ordinary operation needs no public HTTP, global directory, federation, overlay,
Lua/emulator or external provider. Independent invited home boards, explicitly
shared chat and opt-in room/mail adapters require their issue-specific consent and
trust contracts; local password/approval/circle stores never replicate automatically.
No JavaScript/TypeScript is allowed in source, transitive/browser/tool/CI graphs.

Normal budgets remain eight sessions, 5000-ms admission, two credential permits/
kernel slots, three owner helpers, bounded encoded output and five-second drain.
Old software 250-member measurements are not current capacity support or physical
SBC results. Preserve previously executed source bytes and rejected trials before
editing measured inputs; [evidence](evidence/README.md) defines provenance. Product
contracts and accepted ADRs remain in docs; new decisions and defects require
finite GitHub issues, never another local plan/status file.


The default-off [native checkpoint backend](native-mail-checkpoints.md) owns
member-device v2 encryption, protocol state, accepted history and durable outbound
ciphertext. It cannot connect or decrypt through an SSH board stream. Ordinary
client/board operation and the v1 local vault do not enable this adapter.

The explicit optional [checkpoint console](../docs/checkpoint-console.md) renders
authenticated device-local v2 metadata/history/outbox and manages deliberate
upgrade/recovery. It is a standalone entry, outside remote stream and picker
routing. Read-only status/operation inspection does not relax protocol delivery
gates for expired, revoked or history-only stores.

The separate [native XMPP experiment](native-xmpp-transport.md) qualifies a bounded
member-side transport only. It is outside the product Cargo workspace and owns
no vault/server authority. Its selected runtime/resource observations do not
qualify the future integrated mailbox or change normal service budgets.

The default-off [manual native home-mail adapter](native-home-mail.md) combines
that qualified transport with atomic member checkpoints through a separate local
entry. It publishes existing public prekeys, verifies one already enrolled peer,
releases only persisted ciphertext and commits authenticated incoming SCE/history.
Only the first sender explicitly starts a session; the recipient accepts the first
authenticated prekey message. No network write retires the outbox. This adapter
adds neither BBS authority nor reliable archive synchronization, background mail,
roaming or attachments. The research transport's earlier observations remain
separate from this integrated implementation.

# 0031: Home identities and client-encrypted private mail

Status: accepted owner requirements and architecture direction; concrete XMPP,
E2EE and cross-BBS authorization profiles remain proposed/unvalidated.
Date: 2026-10-21. Authority: owner's explicit clarification answers.
Issues #33/#17/#84/#5. Research: [0044](../research/0044-home-identities-and-encrypted-mail.md).
Supersedes: earlier chat-only federation constraints in ADR 0007/product guidance
only for explicit addressed private mail and narrowly scoped home-identity proofs.
Historical records and existing local software behavior remain unchanged.

## Decision

Use independently hosted home authorities and fully qualified member identities,
for example `alice@home.example`. Start with invitation-only federation and the
founder's board as the first peer; no mandatory central account hub. Home identity
is portable between visits, while local membership/roles/grants/approval remain
a visited sysop's responsibility. No shared password/account approval database.

A native local terminal client keeps ANSI/BBS/IRC interaction and member-held
device keys. Home mailbox access remains direct from that client even while it
visits another board. Existing plain SSH continues for rooms/chat/files/games.
Private mail must be encrypted/decrypted on member devices and must not be
rendered as plaintext inside a BBS server's SSH process. Server-readable-first
mail, server-side password-unlock decryption and automatic plaintext fallback
fail the owner's custody gate and are rejected.

Login may unlock a member's local keystore, with independently generated device
keys. Server password reset is not mail-key recovery. New-device enrollment,
verified key trust and recoverable encrypted history need explicit design before
promising a fresh login can read old mail. Separate the trusted local UI from
untrusted remote ANSI and do not send home secrets to visited boards.

## Gates and alternatives

XMPP/OMEMO reuse is the leading experiment, not an adopted complete stack.
Ordinary SSH-server decryption fails server blindness. A permanent centralized
hub fails independent-home direction. Inventing a new cryptosystem is rejected.
A native terminal client with optional independent services plausibly meets gates,
but interoperability, dependency/license graph, scoped auth, hostile-home device
substitution and recovery remain unknown. #84 requires actual private software
proof. XMPP federation alone does not establish visiting-board SSO.

Optional identity verification, E2EE client, mailbox storage and network transport
are distinct module contracts. An external proof never grants local membership;
a peer list does not authorize arbitrary publication or mailbox replication.
Server-visible routing/time/size metadata and home availability remain limits.
New-board visits are distinct from new-device key enrollment and home migration.
Losing a home domain is not solved by possessing a local private key.

No implementation or runtime migration follows automatically from these accepted
requirements. Preserve current local auth/credential/resource boundaries and
standalone board operation. #83 SFTP file work proceeds first. E2EE profile and
backup-format adoption require evidence because device/history migration is costly;
network adapter replacement is cheaper when scoped interfaces stay stable.

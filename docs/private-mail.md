# Home identities and encrypted private mail

Status: network mail/identity remain owner-selected direction, **not implemented**.
The [native terminal client](native-client.md) now provides SSH/Files integration;
its optional [local encrypted vault](local-mail-vault.md) now stores private drafts
and explicit device pins with member-controlled backup/recovery. It does not yet
hold OMEMO device keys, route XMPP, receive/send mail or perform roaming authentication.
A separate [two-home reference proof](../dev-docs/research/0045-encrypted-mail-and-roaming-proof.md)
passes 12 actual OMEMO-2/SCE controls and 11 native scoped SSH verifier controls.
It is research tooling, not the shipped client or an adopted complete protocol.
Native protocol/inbox integration and history recovery remain [#90](https://github.com/gonz0w/faction-bbs/issues/90),
and trusted membership binding [#91](https://github.com/gonz0w/faction-bbs/issues/91).
The [native signing investigation](../dev-docs/research/0052-netbsd-signing-exit.md)
records a controlled source correction; the installed base signer remains rejected.
Native client/identity/mail
proof [#84](https://github.com/gonz0w/faction-bbs/issues/84); local mail #33 and
federation #17. This supersedes the old blanket chat-only federation restriction
only for explicit private-message delivery and scoped home-identity proofs.

A member registers at a home authority and receives an address such as
`alice@home.example`. Another board can have its own `alice@other.example`;
the complete address distinguishes them. Your first board is the first trusted
peer, not a mandatory permanent central account server. Independent sysops invite
federation peers and retain their own community admission decisions.

## One identity, separately approved communities

The future mail increment of the native Faction client will own the member's device keys. It keeps
the ANSI/BBS feel while connecting to a visited board and the home mailbox through
separate channels. Ordinary SSH remains useful for rooms, chat, files and games.
It cannot provide server-blind mail by having the server decrypt and render text.

```text
Member's native terminal client
  local keys + trusted local mail screen
    |-- SSH ------------> visited BBS (its own approval and permissions)
    `-- XMPP over TLS --> home mailbox (ciphertext and routing metadata)
                              `--> invited remote home server --> recipient device
```

The client authenticates its home identity, then presents a narrowly scoped proof
to a visited board. **How that proof is issued/verified is still research.** XMPP
federation is message routing and does not itself log a user into arbitrary BBSes.
No visited board receives the home password, private key or global bearer token.
No replicated password database or automatic local sysop/member privileges.
A visited board may bind the proven home identity to a local membership record;
that record does not become an independent copy of the home identity.

An already enrolled member device reads the home inbox directly while the member
visits another board. Changing boards does not copy the mailbox into that board
or require giving its sysop mail keys. A genuinely new device needs explicit key
enrollment/recovery; merely entering an account password does not recover all
old encrypted messages.

## Server-blind means member-side cryptography

The owner requires **end-to-end encryption for this private-mail feature**. Both
local and cross-board private messages follow that rule. Encrypt subject/body and other private message content inside the encrypted
payload; do not leak a mail subject through an outer plaintext stanza. No plaintext
fallback when a peer/client lacks encryption. Ordinary room posts and local chat retain
their existing behavior; this is not retroactive encryption of stored content.

Login may unlock a locally encrypted keystore. Generate random device/message
keys locally; do not use the server's password hash as a mail key or decrypt mail
inside an SSH process on a BBS. Password changes can rewrap a local keystore;
server-side password reset must not decrypt history or silently authorize a new
device. Recovery needs a member-controlled recovery secret or trusted device.
The [local vault](local-mail-vault.md) selects a bounded encrypted persistence format
and member-held backup/recovery secret. OMEMO device/state/history recovery and
new-device enrollment remain unimplemented; a local vault backup is not yet an
OMEMO history export.

XMPP with OMEMO is the leading reuse candidate, not an implemented/validated
protocol profile. Device-key verification/enrollment is essential: a compromised
home server must not quietly add a device that receives future messages. Pin and
interoperate a concrete OMEMO namespace/version before adoption. See
[research 0044](../dev-docs/research/0044-home-identities-and-encrypted-mail.md).

Servers still see necessary addressing, timing, sizes and delivery information.
Encryption does not guarantee anonymity or prevent a recipient retaining a copy.
A new device cannot necessarily decrypt old ratcheted history without its own
usable encrypted history backup; mailbox ciphertext alone is insufficient.

## Modularity and availability

Identity verification, local membership policy, mailbox storage, XMPP transport,
client-side cryptography and terminal presentation are separate interfaces. A
board remains usable locally with all network adapters disabled. Peer trust is
explicit and revocable; no public registration/directory or public HTTP portal is
required. XMPP server/runtime, authentication proof, metadata retention, home
migration and recovery are validation decisions, not settled dependencies.

If a home server disappears, a portable private key does not automatically keep
its domain/address or mailbox available. Export/recovery, domain continuity and
contact verification after a move need an explicit design. Federation can route
messages between invited homes without requiring every sysop to keep every inbox.

## Encrypted attachments

Tracked separately as [#88](https://github.com/gonz0w/faction-bbs/issues/88), under
#84/#33. Encrypt bytes **on the member device before upload** with fresh attachment
keys; filenames, media types and decryption/integrity material belong inside the
E2EE envelope. Treat remote locators as untrusted, enforce destination/size/time
bounds and require local confirmation before saving. Never send a private attachment
through the current sysop-readable file area and call it server-blind.

Evaluate reuse of [XEP-0447](https://xmpp.org/extensions/xep-0447.html) (Experimental
v0.3.1, metadata/source sharing) and [XEP-0454](https://xmpp.org/extensions/xep-0454.html)
(Historical/Experimental v0.1.0, encrypted media), retrieved 2026-10-21. They are
research candidates, not adopted formats; the latter's historical status matters.
Pin versions, assess actual OMEMO/client interoperability and library maintenance,
and prove that homes, visited boards and blob stores cannot read content/private
filename metadata or silently enroll a recipient device. Include tamper/wrong-key,
revoked device, offline/expiry, retry/cancel, quotas and export/recovery experiments.
Necessary routing, sizes and timing can still be visible. No plaintext fallback.

## What the proof does and does not establish

Two separately owned homes on one NetBSD software VM exchange device-encrypted
subject/body/replies over authenticated S2S TLS. Reference clients re-open queued
mail using the same local device state, fetch bounded ciphertext archives and
refuse fresh tampering, plaintext downgrades and a server-added unverified device.
Known private markers were absent from inspected server stores/logs. This proves
specific internal behavior, not independent-client interop or a security audit.

The reference Python adapter is not a product dependency; the actual client remains
Rust. Mail keys live only in reference client memory. Encrypted local storage,
restart/recovery/history backup, multi-device enrollment and a trusted local inbox
are still needed before member use. Do not turn password login into key recovery.
No ordinary SSH process on a BBS renders this private mail.

Standard OpenSSH certificates can provide target-scoped admission in the native
control. They do not yet bind a home principal to a Faction membership or supply an
online home issuer. Each CA has explicit local principal grants; certificate
possession is not mail-key ownership or imported approval. The installed native
signer remains rejected after the [signing-exit investigation](../dev-docs/research/0052-netbsd-signing-exit.md).
An uninstalled one-line source correction passed the controlled comparison; it
is not an adopted home issuer. The earlier externally minted certificates still
test only the native verifier. Reviewed issuer selection remains part of #91.
Private OAuth/OIDC is still a separately researched alternative. Existing local
BBS login remains available and unchanged.

# Home identities and encrypted private mail

Status: experimental **manual member-device mail** is available through the
[optional native home-mail entry](online-mail.md). It authenticates directly to
one configured home, publishes durable public device material, verifies one peer
and exchanges encrypted subject/body/replies. The local encrypted checkpoint
keeps native protocol state, authenticated history and exact pending ciphertext.
Ordinary SSH never decrypts or renders this mail.

Reliable offline/archive catch-up is [#211](https://github.com/gonz0w/faction-bbs/issues/211).
Integrated background/picker notifications remain #99, trusted visited-board
membership binding #91 and attachments #88. Parent #90 stays open. The earlier
[two-home reference proof](../dev-docs/research/0045-encrypted-mail-and-roaming-proof.md)
and [native interoperability qualification](../dev-docs/research/0054-omemo-interop-netbsd.md)
remain separate immutable evidence; they do not establish full XMPP/OMEMO conformance.
The [native signing investigation](../dev-docs/research/0052-netbsd-signing-exit.md)
remains separate from mail authentication and installed-signer adoption.
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

The optional native mail entry owns the member's device keys. It keeps
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

The client authenticates its configured home account over verified TLS. Presenting
a narrowly scoped identity proof to a visited board remains separate #91 work. XMPP
federation is message routing and does not itself log a user into arbitrary BBSes.
No visited board receives the home password, private key or global bearer token.
No replicated password database or automatic local sysop/member privileges.
A visited board may bind the proven home identity to a local membership record;
that record does not become an independent copy of the home identity.

The manual mail entry currently runs separately from a board visit. The integrated
journey in #99 will keep the home inbox available across visits without copying
mail keys into a board. A new device still needs explicit key enrollment/recovery;
entering a home account password cannot recover old encrypted history.

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
and member-held backup/recovery secret. The separate v2 checkpoint keeps native
state/history/outbox; official backup or recovery opens history only and cannot
resume old ratchets. Fresh-device enrollment remains deliberate.

The [adopted native profile](../dev-docs/native-home-mail.md) uses
`urn:xmpp:omemo:2` with the reviewed SCE subset and full typed Ed25519 verification.
A server device list never enrolls a key. The narrow public-key export reuses
already committed native prekeys; it adds no cipher or ratchet. The
[native qualification](../dev-docs/research/0054-omemo-interop-netbsd.md),
[checkpoint contract](../dev-docs/native-mail-checkpoints.md) and
[adoption decision](../dev-docs/decisions/0080-native-home-mail.md) describe exact
sources and limits. This is a bounded one-peer implementation, not a complete
current-spec conformance or independent security-audit claim.

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

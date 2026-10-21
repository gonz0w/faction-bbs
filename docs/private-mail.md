# Home identities and encrypted private mail

Status: owner-selected direction, **not implemented**. Native client/identity/mail
proof [#84](https://github.com/gonz0w/faction-bbs/issues/84); local mail #33 and
federation #17. This supersedes the old blanket chat-only federation restriction
only for explicit private-message delivery and scoped home-identity proofs.

A member registers at a home authority and receives an address such as
`alice@home.example`. Another board can have its own `alice@other.example`;
the complete address distinguishes them. Your first board is the first trusted
peer, not a mandatory permanent central account server. Independent sysops invite
federation peers and retain their own community admission decisions.

## One identity, separately approved communities

A future native Faction terminal client owns the member's device keys. It keeps
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
Exact format/cryptographic libraries and key-backup policy remain unselected.

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

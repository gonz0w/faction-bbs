# Home identities, roaming BBS access and encrypted mail

Status: requirements selected; protocol/implementation proof pending. Retrieved
2026-10-21. Owner: founder. #33/#17; concrete proof #84. ADR 0031.
Stopping condition here: separate identity, membership, routing and key custody,
choose a bounded reuse experiment and make incompatible SSH assumptions explicit.
No runtime interoperability, crypto audit or supported network is claimed.

## Confirmed owner choices

- Independent home authorities with invited federation peers, the founder's board
  as the first peer. Usernames are rooted in their registration authority.
- A **native local terminal client** for encrypted mail and roaming identity;
  preserve ordinary SSH for the useful BBS. ANSI/IRC/BBS interaction stays central.
- No BBS/XMPP server reads this mail, including a home sysop. Encrypt on the
  sender's member device and decrypt/render on recipient devices. No plaintext
  fallback. The earlier server-readable-first suggestion was rejected.
- Each visited sysop still approves local community access. No distributed
  password database, global sysop role, forced public portal or central authority.
- Keep files local and optional services pluggable; SFTP file areas build first.

These requirements supersede blanket chat-only federation for explicit addressed
mail and limited home-identity proofs. They do not authorize copying rooms, files,
local-only chat, approvals or mailboxes to every participating board.

## Separate four authorities

| Question | Authority and proposed behavior |
| --- | --- |
| Who owns `alice@home.example`? | The home identity service authenticates the account; its domain namespaces the address. A visited board must validate a scoped home proof, not accept a typed address. |
| May Alice use this board/area? | The visited BBS checks its own approved membership, role, epoch and grants. Home authentication does not bypass it. |
| Where does Alice's inbox live? | Home mailbox stores bounded ciphertext; client accesses home directly while visiting another board. Servers route explicitly addressed messages. |
| Which devices may decrypt? | Verified member-controlled device keys and enrollment/recovery policy. A server-controlled password reset or device-list update cannot silently authorize new decryption devices. |

The client separates its trusted local mail/login UI from remote ANSI output.
Remote prompts/control sequences must not solicit home secrets, unlock keys or
inject into local mail composition. A local client can make login unlock a local
keystore; a server-side SSH process cannot render plaintext without seeing it.
Random device keys are separate from authentication/password-verifier material.
A remembered password and server ciphertext alone do not automatically restore
old multi-device ratchet state or history. Design encrypted local-history backup,
trusted-device transfer and a user-held recovery secret before promising recovery.

## Alternatives and gates

| Option | Requirements screen | Tradeoff / decision |
| --- | --- | --- |
| Existing SSH BBS decrypts mail after login | Fails server-blind content requirement | Reject for the requested encrypted-mail feature, even if disk data is encrypted |
| One shared identity/mail hub | Conflicts with independent homes / no permanent central dependency | Founder can operate the initial home, without making it every board's authority |
| Native client + independent XMPP homes + verified E2EE devices | Plausibly meets terminal, federation and custody requirements; actual libraries/versions/NetBSD cost unmeasured | Selected architecture to investigate; XMPP/OMEMO leading reuse candidate |
| Local BBS mail plus later federation bridge | A plaintext bridge/decrypting relay fails current requirement | May route opaque encrypted envelopes only; no silently weaker mail mode |
| Build a new federated protocol/cryptosystem | Large unaudited interoperability/security burden | Reject as first move; reuse existing protocols and established implementations |

No numeric ranking: custody/independence are gates, not weighted conveniences.
The weakest decision-critical unknowns are interoperable key trust/recovery and
scoped visiting-board authentication. XMPP names are not proof of universal SSO.

## Primary sources and what they establish

- [RFC 6120](https://www.rfc-editor.org/rfc/rfc6120.html): client/home-server and
  interdomain streams, TLS/authentication and federation policy. It supplies
  routing/authentication building blocks, not local BBS admission or user SSO
  into arbitrary services. [RFC 7622](https://www.rfc-editor.org/rfc/rfc7622.html)
  defines addresses. Domain routing/identity is separate from an IP/mesh locator.
- [RFC 6121](https://www.rfc-editor.org/rfc/rfc6121.html): message/presence semantics.
  A private inbox/compose/reply experience can be presented over messaging, but
  Faction still needs durable queue/archive/retention policy.
- [XEP-0384](https://xmpp.org/extensions/xep-0384.html), **Experimental 0.9.1** at
  retrieval: multi-device E2EE. Its security text requires establishing device
  trust before using a session. Pin actual namespace/version across both clients;
  the extension number does not prove incompatible releases interoperate.
- [Profanity](https://profanity-im.github.io/), an existing console XMPP client,
  documents [OMEMO/fingerprint workflows](https://profanity-im.github.io/guide/0131/omemo.html).
  This is useful independent terminal-client evidence, not validation of its
  libraries, current protocol profile, licenses or NetBSD behavior for Faction.
- [XEP-0313](https://xmpp.org/extensions/xep-0313.html), Stable 1.1.3: server message
  archives. For the requested design, archive only opaque encrypted messages plus
  necessary routing/time metadata. Stored ciphertext is not proof a new device
  can decrypt old messages.
- [XEP-0184](https://xmpp.org/extensions/xep-0184.html), Stable 1.4.0: client delivery
  receipts do not prove human reading or guarantee delivery. Keep queued,
  transported, recipient-client-received and locally-opened states distinct.
- [XEP-0493](https://xmpp.org/extensions/xep-0493.html), Experimental: OAuth-based
  scoped access to an XMPP account. [XEP-0388](https://xmpp.org/extensions/xep-0388.html)
  is a Stable extensible SASL profile. Neither automatically defines a complete
  Faction BBS visiting login. [RFC 8628](https://www.rfc-editor.org/rfc/rfc8628.html)
  is a standard OAuth device flow; compare applicability, consent UX, phishing/
  session binding and no-JS/private-operation constraints before choosing.
- [XEP-0283](https://xmpp.org/extensions/xep-0283.html), Experimental: notifying
  contacts of a new address. It is not lossless mailbox/key portability after
  losing a home domain. [XEP-0227](https://xmpp.org/extensions/xep-0227.html), Stable,
  concerns server export/import, not member-side secret recovery.
- [XEP-0114](https://xmpp.org/extensions/xep-0114.html) documents a trusted external
  component protocol. Its component trust is broader than an ordinary member's;
  do not treat a component secret as a safe generic BBS login mechanism or expose
  server component authority to Lua/third-party plugins.

Confidence: high for cited specification semantics and selected owner requirements;
unknown for a concrete Faction client/server dependency profile, interoperability,
resource cost, device recovery and visiting-board auth. No server or crypto library
is selected by this report. No new public accounts or federation peers contacted.

## Recommended bounded proof (#84)

Use two private software homes and two member-side clients with synthetic accounts.
Compare an existing lightweight XMPP server (Prosody is a candidate, not adopted)
and known terminal E2EE clients before building a client integration. Review exact
no-JS dependency/license graphs and crypto profiles first. Keep network services
separate from the BBS owner/frontend, optional and disabled by default.

Prove encrypted send, offline queue/reconnect and home inbox retrieval while the
client visits the other BBS. Inspect server stores/logs/backups and captured
application envelopes for a known synthetic plaintext marker. Its absence is a
narrow observation, not a proof against a compromised client/server. Test tampering,
unknown/removed devices, downgrade attempts, duplicate delivery, bounded queues
and explicit non-delivery. Verify device fingerprints out of band; a malicious
home must not enroll a new device through password reset alone. Record metadata
that remains visible and all trust assumptions.

For visiting-board auth, compare a standard device/issuer proof using separate
authentication keys with narrow scopes; require issuer/audience/nonce/connection
binding, short expiry and replay denial. Never reuse raw E2EE keys for a new
signature scheme or pass home passwords to a visited BBS. Reject trusted-peer
claims for unowned namespaces and retain the recipient sysop's local deny/revoke.
If the auth proof isn't resolved, demonstrate mail independently and leave that
specific feature unclaimed; do not simulate SSO with password forwarding.

Home outages affect home-auth/mail access; they must not stop local-board use.
Keys alone do not keep an abandoned domain working. Determine same-domain host
migration, domain-loss moves, encrypted export, recipient verification and recovery
semantics separately. Do not add public blockchain/key-transparency services as
implicit dependencies. Future verified migration is compatible with invited peers.

Reversal: adapters can change while home-principal/local-membership separation
stays. Choosing an E2EE profile/backup format has costly device/history migration;
that adoption needs actual interoperability and recovery evidence. Existing local
accounts/session/crypto limits remain unchanged while this design is explored.

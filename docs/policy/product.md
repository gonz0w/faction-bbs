# Product and compatibility contracts

These local clauses were extracted from [PROJECT.md](../../PROJECT.md) without changing their
requirements. Source identity is the consumer revision containing this extraction.
Command paths and literal configuration paths remain relative to the repository root.
Selected applicability is recorded only in root `policy.toml`; linked contracts retain
their existing scope. The owner changes required policy through the existing process.

## Product and implementation contracts

Read [product direction](../project.md), [privacy/security](../security-privacy.md),
[current architecture](../../dev-docs/architecture.md), and relevant feature docs/ADRs.
Rust, primary NetBSD, BSD-3-Clause and **no JavaScript/TypeScript anywhere**, including
transitive/browser/tooling/CI dependencies, remain. Keep the Citadel room model,
WWIV login/approval, sysop-defined ANSI/lightbar/season identity and classic/plain
fallback. Capability observations/overrides, never terminal brand alone, gate
modern Unicode/graphics. Untrusted terminal strings never become commands or art.

Preserve current local approval/role/epoch/grant checks on every action. Keep
transport admission, BBS identity, destination approval, OS operator authority and
member-device cryptographic identity distinct. Who shows broad current activity;
group `/who` and Lounge keep distinct interfaces with universal visibility and
availability. No private room/channel/draft/game names in broad presence.

All new private mail, keys and decryption stay on the member's device. No
server-readable fallback, server-side SSH decrypt/render, shared passwords or
automatic visited-board membership. Independent invited home boards are selected;
XMPP/OMEMO remains a candidate beyond existing reference proofs/local vault work.
IRC live chat, Netnews discussion articles and private mail have separate formats.
Optional federation requires the relevant explicit room/author/peer/device policy;
no automatic replication or publication of local rooms/files/accounts/circles.

Keep ordinary board operation independent of optional HTTP, federation, emulators,
Lua, overlays or external providers. Doors use separate host identities/resources;
no generic emulator/native launch under the BBS owner/frontend. Additional game
expansion, hardening #106–#111 and advanced combined 250-member experiments are
owner-deferred. Existing defenses remain; deferral never authorizes weakening them.

## Accepted first optional roaming model

The owner [selected this model](https://github.com/gonz0w/faction-bbs/issues/91#issuecomment-6422333314) on
2026-10-26 for the first optional prototype: one restricted OS transport account
per independently approved home/local-member binding. Devices sharing that
binding have one BBS identity; the UID path does not attest individual leaf keys.
Certificate expiry governs new SSH admission. Current local binding/issuer/member
revocation and normal session rules govern established access.

Keep independent local consent/approval, root-managed transport policy/manifest,
immutable UID history and device-private mail keys. This choice does not increase
resource/experiment caps, adopt a package or online issuer, authorize host changes
or a public listener, or establish production support. Remaining enrollment,
issuance and operational work stays issue-scoped under #91.

# 0047: Modern terminal, operator and protocol research inputs

Date: 2026-10-21. Owner stories #95–#101. Status: primary-source reconnaissance,
not a selected stack, implemented feature, protocol interoperability result or
benchmark. [Product stories](../../docs/modern-sysop-and-communications.md) own the
user intent; GitHub holds acceptance/status. Rust/NetBSD/no-JS and member-held
mail keys remain requirements. This report installs/runs no new dependency,
changes no firewall/host setting and leaves the live playground intact.

## Terminal facts and implementation questions

[Ghostty's feature documentation](https://ghostty.org/docs/features) explicitly
lists Kitty graphics, keyboard and synchronized rendering. Its GPU belongs to the
client emulator; a BBS host does not need one to emit supported terminal output.
Do not infer feature support from a marketing name or assume its VM console is
the same terminal as the host's SSH window. Exact Faction support needs tests.

The [Kitty graphics specification](https://sw.kovidgoyal.net/kitty/graphics-protocol/)
has support queries and several transfer mechanisms. Those alternatives are a
reason to constrain the renderer and negotiation, not permission to expose file/
shared-memory reads or network/clipboard paths. #96 must bound replies, image
sizes, bytes/caches and refresh and prove unknown-terminal/multiplexer fallback.
Current control filtering is unchanged; no graphics escape is now allowed.

[btop's own configuration documentation](https://github.com/aristocratos/btop)
describes braille, block and TTY graph modes, font limitations and refresh controls.
These are text-cell techniques, distinct from bitmap protocols. Use that visual
reference for #95's modest read-only dashboard before treating images as essential.
No btop dependency, collectors, full source graph or port is adopted/validated here.

Questions for a measured fixture: which verified features survive direct console,
SSH and multiplexers; which fonts/widths affect alignment; how to route probe
replies separately from commands; what delta-render/idle/asset budgets fit current
bounds; and how a user override recovers from incorrect detection. Reference
images/code/art remain subject to provenance/license policy and no-JS gates.

## NetBSD abuse controls and NAT placement

The [NPF configuration manual](https://man.netbsd.org/npf.conf.5) defines source/
destination filtering and separate source/destination address translation. The
[npfctl manual](https://man.netbsd.org/npfctl.8) documents address tables, validation/
control operations and existing-state effects. Those are useful primitives for a
narrow sysop workflow, not a reason to give the BBS owner raw /dev/npf access or
replace administrator policy. The exact NetBSD release/configuration must be
pinned when testing; web manuals alone establish no deployment result.

Inference from those primitives: an address-based rule can distinguish only the
address observed at its enforcement point. Destination NAT can leave the original
source visible; source NAT/proxies can collapse multiple users behind a gateway.
A whole shared-source ban may deny innocent members. Do not claim that host, guest
and upstream router all have the same view, or trust an application-supplied IP.
[QEMU's documented host forwarding](https://www.qemu.org/docs/master/system/invocation.html)
is an explicit VM topology input, not proof that the real outside source survives.
#97 needs direct/bridged/translated/host-forward/IPv6/private-network controls with
source observations and lockout recovery before adoption. No such matrix ran here.

#98 separately scopes administrator-authorized NetBSD configuration plans. Keep
normal local diagnostics, host authority and narrow apply operations separate;
preview/backups/rollback and preservation of unrelated settings are requirements.
No global tuning or base SSH penalty changes are authorized by these stories.

## IRC and XMPP serve different contracts

[IRCv3's specifications](https://ircv3.net/irc/) extend the IRC client protocol.
[InspIRCd's spanningtree documentation](https://docs.inspircd.org/4/modules/spanningtree/)
is a concrete server-link design within a compatible network. Neither establishes
universal links between independently chosen IRC networks. #100 compares a
maintained server/adapter/bridge and actual external-client interoperability;
keep approved local membership, per-channel subscriptions and identity mapping.
Public/shared channels are not an automatic anonymous Internet service.

The requested XMPP native experience (#99) depends on #84/#90's actual encrypted
adapter and #91's independent identity binding. A notification is not decryption,
an XMPP account is not local BBS approval, and an IRC bridge must not receive
private mailbox keys/plaintext. #88's device-encrypted blobs/metadata stay distinct
from existing sysop-readable community files. No protocol runtime is selected here.

## Persistent-room candidates for #101

| Candidate | Why investigate | Required gaps before adoption |
| --- | --- | --- |
| NNTP/private news feeds | Persistent articles grouped by topic, stable article identity and established peer-feed model | Private peer/reader ACLs, local membership mapping, TLS/auth, retention/moderation and actual footprint; not inherently E2EE. |
| XMPP PubSub, with any needed forum model | Could reuse invited XMPP homes and persistent event topics | Forum reply/order/backfill/unread semantics and remote-subscription policy; pubsub is not a complete replicated forum or OMEMO guarantee. |
| Matrix rooms | Explicit federation and persistent room/state model | Room/identity/access model, HTTP federation exposure, dependencies and complete NetBSD resource cost; no browser/JS client implied or adopted. |
| ActivityPub forum model | Existing actor/inbox-based federation and forum implementations to compare | Private distribution, moderation/auth/identity semantics, HTTP endpoints and no-JS native/server graph; not automatic private-room support. |

Primary protocol sources, retrieved 2026-10-21:

- [RFC 3977](https://www.rfc-editor.org/rfc/rfc3977.txt), NNTP. Implementation controls
  need pinned maintained software; [INN's incoming peer configuration](https://www.eyrie.org/~eagle/software/inn/docs/incoming.conf.html)
  and [reader access configuration](https://www.eyrie.org/~eagle/software/inn/docs/readers.conf.html)
  show distinct feed and reader policy surfaces. These web pages describe development
  HEAD, not a named release used here. No INN build, license/source-graph or cost
  claim is made. NNTP is an initial comparison candidate, not a selected engine.
- [XEP-0060 v1.30.0](https://xmpp.org/extensions/xep-0060.html), Stable generic
  publish-subscribe. [XEP-0045 v1.35.5](https://xmpp.org/extensions/xep-0045.html),
  Stable multi-user chat, is a comparison for channel semantics, not proof of a
  persistent Citadel room adapter or E2EE. Do not confuse XMPP chat rooms with
  Faction's discussion-room membership/content model.
- [Matrix server-server specification](https://spec.matrix.org/latest/server-server-api/),
  living protocol reference. A real experiment must pin the exact specification,
  room version and software; no server/library source graph or footprint checked.
- [ActivityPub Recommendation](https://www.w3.org/TR/activitypub/), 2018-01-23.
  Research a concrete implementation/profile rather than assume that a generic
  protocol preserves private discussion policy. No forum service adopted/tested.

FTN/echomail can be a heritage/store-forward comparison if useful, but the attempted
FTSC document fetch was unavailable; no unverified protocol/library claims are made.
It remains an optional follow-up candidate rather than fabricated source evidence.

## Decision boundary, confidence and next validation

High confidence in the owner requirements and linked documented mechanisms;
unknown for complete Faction interoperability/security/footprint or production
configuration. Prefer comparing reusable protocols to inventing a new federation
wire format. This is a comparison order, not evidence that any candidate wins.

The first room experiment must use two independently owned, invited software
nodes with an explicitly selected shared room, and a separate private room that
never leaves its board. Include author/peer authorization, denied subscriptions,
history policy, identity/reply mapping, duplicates/loops, partition/reconnect,
moderation/removal/retention and unchanged local approval. Measure entire helper/
service/storage costs with adapters disabled and enabled. No public room sync,
central hub, copied account store, mandatory web portal or private-mail downgrade.

Reversal is cheap for these unimplemented stories. It becomes costly once peers
hold replicated history, which cannot be reliably retracted by disconnecting a
board. Record publication consent and retained-copy limits before that boundary.
Keep a useful local board when every optional network adapter is unavailable.

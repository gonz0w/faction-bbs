# Modern terminal communities and the local sysop console

Status: confirmed product stories and research direction, 2026-10-21. These are
**requested capabilities**. The optional [terminal settings](terminal-capabilities.md)
and [bounded Kitty preview](kitty-graphics.md) now have software implementations;
the broader dashboard, network and rich-art stories remain planned.
The owner wants a modern underground communications platform, shared with invited
independent sysops, whose interface keeps its ANSI/BBS identity while taking
advantage of terminals such as Ghostty. GitHub holds implementation status.

| Story | Work |
| --- | --- |
| Local VM/host-console sysop dashboard | [#95](https://github.com/gonz0w/faction-bbs/issues/95) |
| Modern terminal capability detection and graphics | [#96](https://github.com/gonz0w/faction-bbs/issues/96) |
| NAT-aware abuse/firewall workflow | [#97](https://github.com/gonz0w/faction-bbs/issues/97) |
| NetBSD setup and maintenance assistant | [#98](https://github.com/gonz0w/faction-bbs/issues/98) |
| Integrated native XMPP notifications/private conversations | [#99](https://github.com/gonz0w/faction-bbs/issues/99) |
| Real IRC/IRCv3 and invited server-link comparison | [#100](https://github.com/gonz0w/faction-bbs/issues/100) |
| NNTP-first opt-in room federation | [#101](https://github.com/gonz0w/faction-bbs/issues/101) |
| Netnews-format discussion articles | [#103](https://github.com/gonz0w/faction-bbs/issues/103) |
| Optional news-provider subscriptions | [#104](https://github.com/gonz0w/faction-bbs/issues/104) |

These stories refine #18's operations, #13/#25's terminal/themes, #21's screen
journey, #26's chat and #17's federation. #90/#91 still own native mail/verified
home identity and #88 owns encrypted attachments. This does not close those gates,
select a new runtime, enable a protocol or change a running host's policy.

## The operator's side of the board

After authenticating to the VM/host operating system at its console, an authorized
operator can open a local sysop dashboard. It should feel like the old BBS status
screens with modern panels, graphs and keyboard controls. The first increment is
read-only: sessions/limits, pending approvals, aggregate activity, service/peer/
queue health, transfer/door status, CPU/memory/disk/network and backup health.
Unavailable or stale data must be labeled rather than fabricated.

Give the screen the sysop's theme and board identity, useful help and clean
transitions. btop is a visual reference for information density and text-cell
graphs, not a required runtime or a claim that all of its collectors are adopted.
Keep refresh and history bounded, pause sampling when unused where practical,
and measure its complete cost separately from the BBS session budget. No private
message bodies, passwords, device keys or encrypted-mail plaintext in telemetry.

An OS operator and an in-BBS sysop are separate authorities. A member logging in
through `factionterm` does not acquire a host shell or administrative privileges.
Use an explicit local dashboard command, with an optional dedicated operator
login flow. Preserve normal OS authentication, an emergency console/shell and
safe exit; do not replace getty or automatically log in root. This operator VM
console does not change the member-access decision against physical serial support.

Future service/member/host actions need narrow authorized operations. A host
configuration helper must not turn the shared BBS owner or theme/Lua processes
into root, or expose a general shell-command field.

## Modern capabilities, recognizable BBS identity

Detect individual capabilities and offer explicit overrides. A terminal name or
forwarded TERM value is a hint, not a guarantee; a VM console can support less
than a Ghostty window connected over SSH. Keep at least these rendering tiers:

1. Readable plain text and basic ANSI, usable through the actual VM console.
2. Enhanced text-cell UI: colors, Unicode block/braille graphs, better formatting,
   efficient synchronized/delta redraw and negotiated input features.
3. Separately negotiated graphics: bounded image artwork/previews where a reviewed
   protocol and the complete connection path support them.

Research true color/256 colors, Unicode widths/fonts, graphics, synchronized
updates, keyboard and optional mouse support across direct TTY, SSH, multiplexers
and the native client. Ghostty documents Kitty graphics; its feature list is not
a Faction compatibility test. Do not infer Sixel support from the word “modern.”
btop-style graphs can be made from text cells without bitmap graphics or a GPU
on the NetBSD host. [Research notes](../dev-docs/research/0047-modern-terminal-and-federation-options.md)
link the primary specifications and distinguish those techniques.

Capability probes must be bounded and parsed in a dedicated state. Probe replies
must not become menu hotkeys, chat input, pasted commands or identity authority.
Keep existing terminal-control filtering; any graphics path needs a narrow trusted
producer and validated data/pixel/byte/cache limits, not raw OSC/DCS/APC pass-through.
No automatic terminal file/network fetch, clipboard access, external URL opening
or secret prompt can follow from untrusted artwork or remote content.

Continue the Oblivion/2-inspired menus, arrow selection/Enter and hotkeys, sysop
palettes/layouts and seasonal identity. Entering a distinct screen should clearly
replace the old view; live chat can deliberately scroll. Preserve active input.
Room readers need author/room/message context, clear separators, wrapping, quote/
reply formatting and pagination while keeping Citadel's chronological room model.
The [menu requirements](menus-and-themes.md) and #21/#25 own this polish. Richer
terminals should enhance the experience without imposing a fixed “dashboard skin”
on every board. Historical scene artwork still needs distribution rights.

## Host configuration and abuse management

The sysop assistant should explain which NetBSD settings are required or optional:
service users/groups, permissions, packages, rc.d order, listeners/interfaces,
SSH separation, optional file/game/protocol services, timekeeping and backup/disk
health. Start with diagnostics and a plan. Show exact changes, required authority,
validation, maintenance/restart effects, backups and rollback before an authorized
apply. Preserve unrelated administrator settings and the current session/helper/
credential limits. Never silently tune kernel/NTP/allocator/network settings.

For abusive peers, separate BBS account bans, SSH admission penalties and packet
filtering. NetBSD NPF is a native candidate for structured address/CIDR controls.
The operator needs to see the observed peer, rule scope/location, reason/expiry,
allowlist, removal and console recovery path. Do not flush the host firewall or
import a member-supplied “original IP.” A BBS sysop role alone does not authorize
host firewall changes. The [research notes](../dev-docs/research/0047-modern-terminal-and-federation-options.md)
cover why NAT placement matters; #97 requires actual software topology tests.

A destination-port mapping may preserve the original source; source NAT or a
proxy may instead present a shared gateway. If the guest sees only a gateway,
guest IP rules cannot distinguish individual callers behind it. Enforce at an
appropriate host/gateway or use authenticated member controls, with explicit
shared-source effects. Treat shared NAT addresses as network locations, not
member identities. The current localhost VM playground is not evidence that
external source-address blocking works. This story enables no live firewall or
host changes and weakens no OpenSSH penalty.

## A single terminal experience, distinct communication services

| Surface | Direction | Boundary |
| --- | --- | --- |
| Public/shared live channels | Evaluate real IRC/IRCv3 integration and invited compatible server links | “Public” describes conversation scope; it does not automatically open anonymous Internet access. Local-only channels stay local. |
| Private live conversations, mail and private attachments | Tight native XMPP integration with verified member-side encryption | Home inbox access stays on the member device. Visited sysops receive neither keys nor private content. Actual OMEMO profile/runtime remains gated. |
| Persistent discussions | Citadel-style chronological rooms; Netnews article direction with optional NNTP adapters | Local by default. Shared rooms need explicit sysop/author/peer publication policy and no automatic account/approval replication. |
| Community file areas | Existing local reviewed SFTP files | Readable by the sysop; they do not substitute for encrypted private attachments. |

The native client should indicate unread/private activity and let members open,
read, reply and chat without losing the BBS session. That notification/inbox UI
must be trusted local presentation, with truthful offline/stale states. A BBS
server must not decrypt mail to draw a badge or private body. Private subjects,
content keys, vault secrets and home credentials stay off visited boards; sharing
notification metadata requires its own explicit policy. Ordinary SSH stays useful
for rooms/chat/files/games, with a clear route to the native private client.

The existing local `/join` and `/me` commands are not IRC wire interoperability.
Compare maintained IRC services/adapters before adoption. Linked servers in one
compatible IRC network are not universal XMPP-style federation between arbitrary
IRC networks. Plan nick/full-home identity mapping, permissions/moderation, private
channels, reconnect/history and loop prevention. No IRC bridge handles private
XMPP plaintext/keys or imports home roles. Every adapter can be disabled while
the local board remains useful.

## Netnews room articles and optional NNTP connections

The owner now wants to investigate federated discussion boards using an existing
protocol. This authorizes **research into explicitly shared rooms** beyond the
earlier local-only-room direction; it does not authorize exporting an existing
room, enabling public discovery or mirroring all board data. Local rooms, accounts,
roles, approval stores and community files retain their current locality.

The owner subsequently selected Netnews-style formatting for discussion-room
posts only. IRC live chat and XMPP private messages retain their own formats.
[Article/news stories](discussion-articles-and-news.md) separate #103 local article
model/codec/migration, #101 NNTP-first policy/interop and #104 optional provider
subscriptions, read-only first. Formatting never implies publication or replacing
SQLite. Current posts have no original subject/date; migration must be honest.

XMPP PubSub, Matrix and ActivityPub remain sourced alternatives if NNTP fails a
named requirement. No production wire implementation is selected or enabled.
Protocol names alone establish neither lightweight NetBSD operation, E2EE, private
peering nor equivalent membership/deletion policy. See the
[research](../dev-docs/research/0047-modern-terminal-and-federation-options.md).

Any future adapter needs room/author/peer publication rules, history/backfill and
new-subscriber policy, stable identities/references, local unread cursors,
dedup/loop handling, moderation/retention, peer removal/export and honest retained-
copy limits. Invited federation must not become implicit public syndication or
#42's separate permanent/public publishing. No room protocol is adopted here.

## Build order and current state

Keep near-term improvements useful: screen/message polish, a bounded read-only
local operator view and the ongoing native encrypted-mail work. Research terminal
capabilities and wire protocol reuse before enabling extensions. Host/firewall
writes follow a reviewed privilege/apply/recovery design, not a dashboard shortcut.
The stories have concrete acceptance criteria and software validation gates; none
requires physical SBC access. At the owner's request, the local playground is now
stopped and privately preserved for later testing; its image checks are separate
from these unimplemented communication stories.

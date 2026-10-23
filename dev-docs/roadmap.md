# Roadmap and stage

## Discussion discovery

After merged #164, #142 adds useful access to older conversations: bounded literal
filters, exact read-only sources, own bookmarks and deliberate reference exports.
Accelerated FTS, other bookmark types and member import remain follow-on work.
Classic terminals, current resource defaults and device-only private-mail search
remain; the saved playground stays off and hardening stays later.

## Personal catch-up

After merged #163, #138 adds a local returning-member desk: notice, capped room/
reply previews, own favorites, eligible files and gatherings. Explicit saved visits
avoid automatic login tracking; source jumps reuse existing authorized readers.
Private-mail notifications stay member-side under #99; no new capacity/hardening
claim. The saved owner playground stays off. See [the guide](../docs/catchup.md).

## Local questions and voting

After merged #162, #139 adds optional local single-choice polls, explicit ballots,
result-visibility policy/text-cell charts, immutable wording after first vote,
creator close/cancel and sysop moderation. Keep classic terminals and current
resource defaults. Multiple selection/grouped surveys remain follow-on work;
the owner playground stays off and hardening stays later.

## Local gatherings and social nights

Issue #136 adds the next local social batch: optional one-off UTC events, explicit
counts-only responses, organizer changes/cancellation, sysop moderation and approved-
login reminders with deliberate room/chat jumps. Classic terminals and current
resource defaults remain. Recurrence/external calendars and federation remain
separate. The saved owner playground stays off; hardening stays later.

## Current playable preview

After merged #151, #152 makes the current social board easy to try through a
[private persistent SSH playground](../docs/test-playground.md). Sample rooms,
wall/pages/plans/chat and the existing Lua arcade reuse the current runtime.
This batches owner testing and lifecycle work; next social stories remain open.
Hardening stays later.

## Local people and projects

After checked #150, #135 adds optional local member pages and dated `.plan` updates:
explicit author preview/visibility/removal, bounded history, sysop presentation and
moderation, classic fallback and schema-10 recovery. No public Finger/home files,
identity proofs or profile federation. Next social stories remain separate, and
hardening stays later.

## Full-screen writing and room recovery

After checked #148, #141 adds the [full-screen room editor](../docs/writing-desk.md),
classic fallback and explicit recoverable board checkpoints. Native device mail
remains separate. Schema-8 stopped upgrades/backups, inert themed titles, current
approval and original resource defaults stay explicit. Social wall/member pages/
gatherings and the other #133 stories can build on the room/visit foundation;
advanced editor/autosave/member-export increments remain separate. Hardening later.

## Useful room navigation

After checked #132, the [named room browser](../docs/room-browser.md) adds themed
ANSI room selection, current unread counts and selected-room reading/posting.
Plain and manual N/H/P remain, with unchanged chronological/unread semantics.
This #21/#12/#25 batch adds no schema, federation or hardening work.

## Native community visits

After checked #131, the optional [native board launcher](../docs/board-launcher.md)
adds member-owned profiles and deliberate visits/return without restarting the
client. Directory cards never import profile/host-key authority; every destination
requires its own account/approval. Member submissions and automatic profile handoff
remain follow-on #27 work; hardening remains later.

## Useful community directory

After checked #130, #27/#21 adds the optional local [Other BBSes directory](../docs/other-boards.md):
reviewed pack, themed arrow/hotkey paging and SSH connection cards. No automatic
connection, credentials, host-key trust, remote approval or federation. Submission
UI/native handoff remain follow-on work. Hardening stays later.

## Local operator batch

After merged #130, related #95/#18/#98 work adds the first optional
[local sysop console](../docs/sysop-console.md): private aggregate snapshots,
live CPU/RAM/network counters, bounded recent graphs, themed panels and classic fallback. No member SSH
authority or host settings change. #95 now adds explicit [backup checks](../docs/backup-checks.md) and a Recovery
panel; peer adapter telemetry and broader visual acceptance remain. #98 applies/rollback, #97 firewall and hardening stay separate.

Updated 2026-10-23. GitHub issue and milestone state is authoritative; see the [issue index](issue-index.md).

## BBS menu and theme polish

After merged #121's software compatibility work, #25 now improves the ordinary
lightbar main menu and sysop authoring workflow. Compact/wide ASCII panels,
base titles/descriptions and manual seasonal palettes preserve classic fallback,
action checks and output limits. [Preview/customization](../docs/theme-packs.md)
needs no live board. Final original/authorized identity art and native visual review
remain separate work. Optional annual UTC/system-local calendars,
priorities, manual/disabled recovery and original seasonal login templates now
extend this foundation. Each approved login pins its shared catalog selection;
examples default disabled. See the [calendar guide](../docs/theme-packs.md#seasonal-calendar-and-login-screens).
Opt-in custom grids now define explicit positions, spatial arrows and ordered
Tab with classic fallback. Bounded SAUCE/iCE import enables original/licensed
80-column login cards. Opt-in long-form identities now page in bounded screens
with Enter/skip and classic recovery. Animation, larger formats and final scene
identity stay open.
The opt-in [Nightwire artwork](../docs/artwork-nightwire.md) now provides original
bespoke lettering and radio-city scenes across short/paged and seasonal identities,
with matched compact cards. Authoring source/previews are reproducible; owner
visual acceptance and real terminal/font certification remain open.
Hardening stays later.

## Modern terminal foundation

Merged #116 completes schema-7 Unicode composing. #96 now supplies optional
[native room/subject text and terminal settings](../docs/terminal-capabilities.md),
cell-aware wrapping/clipping, color/graph samples and isolated sync probing.
Default classic mode, native-client control filtering and existing resource/
approval rules remain. Bitmap rendering, extended input and full emulator/font
interop remain separate work; hardening #106–#111 stays later.

## Unicode subject composing

#115 is merged (a5c0465), completing the bounded local #103 text profile and
original INN validation. #114 now implements [schema-7 UTF-8 composing](../docs/unicode-subjects.md)
in local/worker and shared terminals, with a stopped upgrade, versioned snapshot
validation and bounded classic display. Optional native scalar rendering follows under #96;
actual news peers/providers stay #101/#104. Keep hardening #106–#111 later.
Software-only native NetBSD validation suffices; no new 250-user capacity claim.

## New owner stories from the running playground

[Modern sysop/communications stories](../docs/modern-sysop-and-communications.md)
#95–#101 capture the OS-console dashboard, modern terminal detection/graphics,
NAT-aware abuse rules, NetBSD setup assistant, native XMPP unread/private UX,
actual IRC reuse/server linking and NNTP-first optional room federation. The
owner selected Netnews article formatting for discussion posts only (#103); IRC
and XMPP keep live/private formats. #104 captures optional news-provider access,
read-only first; [article/news requirements](../docs/discussion-articles-and-news.md)
separate local codec/migration from publication and peering.
These are planned, not implemented. Improve #21/#25 screen/message presentation
and bounded operator visibility while keeping the existing local board useful and
continuing #90/#91/#88. Privileged host applies and protocol adoption follow their
own design/validation gates. The local playground was already cleanly powered
off; a private standalone image checkpoint and restart helpers are preserved.

Researching explicit shared rooms is now allowed beyond older local-only wording;
this does not enable automatic export, account replication or public syndication.
Read [research 0047](research/0047-modern-terminal-and-federation-options.md) before
assuming Ghostty graphics, guest IP bans, IRC federation or a forum protocol works.

The room-article batch supplies a dependency-free text-article codec, paged room
presentation (#21/#25), persistent metadata, authored subjects and local article
projection. Merged #115 completes the bounded #103 profile and original INN offline
controls; schema-7 Unicode composing follows in #114. #101/#104 still gate live
peer/provider interoperability. No automatic room export or external service is
introduced. See [room articles](../docs/room-articles.md).

## Product-first sequencing

Latest owner steering: **local SFTP file areas (#83) first**. The optional shared
Files/SFTP implementation and complete backup path are now available for the
ordinary small-board preview; see [file-area setup](../docs/file-areas.md).
The [native transfer client](../docs/native-client.md) and scoped room/chat links
(#87) now follow in the working preview. Home identity/encrypted mail research
(#84) and device-encrypted attachments (#88) remain separately open. Use independent
invited home authorities, member-held keys and separately approved communities.
No BBS/home server reads this private mail; ordinary SSH still serves local BBS
features. See ADRs 0030/0031 and [extension contracts](extension-contracts.md).
Pause game expansion under #81 while preserving DOS/Amiga/other-host adapters.
The [two-home reference proof](research/0045-encrypted-mail-and-roaming-proof.md)
passes encrypted mail/offline/archive/device-refusal controls and native scoped
SSH certificate verification. No complete native crypto/SSO runtime is adopted.
#90 now has a [local encrypted vault](../docs/local-mail-vault.md), private drafts,
explicit pins and member backup/recovery. Native inbox/send/ratchet/history and
independent interop remain; #91 binds proven home identity to local membership;
#92 investigates native signing failure. Attachments remain #88.
These explicit mail/identity capabilities supersede older chat-only wording below.

The owner explicitly redirected work toward a running product for ordinary users
on 2026-10-21. [#71](https://github.com/gonz0w/faction-bbs/issues/71) supplies the
[small-board install/member journey](../docs/quickstart-netbsd.md): eight-session
shared backend, ordinary restricted SSH, sysop-owned ANSI theme, approved rooms
and local chat, with private backup/recovery. Keep improving that experience.
[#73](https://github.com/gonz0w/faction-bbs/issues/73) adds the first optional
[Lua arcade](../docs/lua-doors.md): sysop Lua lobby and two original games outside
the backend under a separate game UID. The follow-up adds persistent local top
tens/personal bests, run counts and an arcade guide. Native legacy compatibility
and broader game APIs remain follow-ups. The owner accepted both original game
servers and isolated DOS hosting; the [classic connector](../docs/classic-game-doors.md)
now has owned-fixture NetBSD evidence and a separate [original LORD software proof](../docs/original-lord.md) (#78). TWGS
activation/validation (#79) is owner-deferred. The maintained host/sysop workflow
(#81) and per-title inventory (#34) follow. Keep improving the
usable board before resuming combined 250-user edge research.

Further 250-user combined abuse/recovery and uncommon host/isolation scenarios
are deferred under #70/#55/#20. Preserve findings and security constraints; do not
make experimental profile adoption, long-soak capacity or physical SBC testing a
prerequisite for completing the ordinary board. Keep existing admission limits,
local approval and credential protections. Files/mail/directory and later network
features stay explicitly sequenced rather than becoming first-preview promises.

## Current stage: first local implementation

The repository now has a product baseline, BSD-3-Clause licensing, contribution/security guidance, a Codex-friendly workflow, documentation checks, and an issue backlog. Rust and NetBSD as primary target are selected. The first [local core/console](../docs/local-development.md) now exercises real accounts, approval/revocation and durable chronological rooms. A [local ANSI/theme/menu baseline](../docs/theme-packs.md) now supports both navigation routes, bounded identity import and manual seasons. [Local command chat](../docs/local-chat.md) now exchanges messages between simultaneous consoles with private-channel grants and bounded history. [Private snapshot/restore](../docs/export-restore.md) now covers current durable state with no-overwrite publication and fresh sessions. [Restricted SSH](../docs/restricted-ssh.md) and [native NetBSD application validation](research/0024-restricted-ssh-netbsd.md) now pass a bounded real-member slice. Managed [NetBSD lifecycle and eight-session load/failure checks](research/0026-service-lifecycle-load.md) now pass. Complete resource/capacity, durable failure and maintained appliance work remain next; no complete production stack or network release exists.

The [decision framework](research/decision-framework.md) was accepted through merged PR #36, closing issue #1. [Issue #3](https://github.com/gonz0w/faction-bbs/issues/3), merged through PR #37, supplies [proposed journeys and synthetic link/workload controls](research/0003-constrained-user-journeys.md) for modern private networks with occasional degradation. No runtime experiment or production budget is validated by that analysis. [#8 interaction research](research/0008-interaction-budget.md) now proposes small text pages, capabilities and recovery requirements with an application-paced ASCII/ANSI comparison. [#28 resource research](research/0028-small-host-budgets.md) proposes tiny application budgets and a full measurement contract for 100–250 connected members on a 1–2 core / roughly 1 GiB host, plus a bounded C allocation-ownership experiment. Neither establishes secure server performance. The [#4 native comparison](research/0004-runtime-comparison.md) and [#9 SQLite recovery/export evidence](research/0009-local-storage-comparison.md) now support [a proposed Rust/readiness + SQLite validation slice](decisions/0005-rust-sqlite-validation-slice.md), with Go retained in the comparative evidence. The owner subsequently accepted Rust/primary NetBSD in [ADR 0006](decisions/0006-rust-and-primary-netbsd.md). SSH/private-network operation is [accepted product direction](decisions/0004-ssh-and-optional-private-network.md); the production backend and minimal appliance remain open. [Native NetBSD 11 ARM64 evidence](research/0023-netbsd-dependencies.md) now verifies pinned Mio/rusqlite with bundled and explicitly linked pkgsrc SQLite, plus nine synthetic restricted OpenSSH gateway oracles; physical SBCs, real identity and complete session costs remain unvalidated. Next #11/#20/#35 should bound the integrated secure read/post/chat slice, including #23 SSH enforcement and #18 complete software/VM accounting. Physical SBC checks are deferred per the later owner instruction below. #4/#9 remain open for integrated evidence; broad surveys must not block local core building. Bounded optional-extension research continues alongside it. Browser-console feasibility under the strict no-JS policy remains an explicit decision gate.

## Software-first build path and extension research

The owner has no physical SBC test hardware. Developer-host and NetBSD VM tests
drive software building now; physical SBC testing follows a usable complete
product and gates hardware-specific claims only. See [ADR 0007](decisions/0007-software-first-and-extension-direction.md).

Build the [first useful local software slice](first-local-slice.md) under #35:
real local identity/approval, safe terminal login/menu, durable Citadel rooms and
local chat, then SSH/VM operation and bounded load/recovery tests. Do not wait for
every optional network/storage/game comparison before building this slice.

Lua menus/isolated doors are first after that core. The [extension report](research/0010-extensible-ansi-platform.md) records a 16-oracle Rust/Lua boundary probe and
optional modern storage/network/payment directions. The new future issues capture
permanent rooms (#42), payment models (#43), ANSI shops (#44) and game economies/
wagers (#45); existing #5/#10/#15/#16/#17 own generic adapters, Lua and networks.
Compare member external wallets and sysop-run payment services before choosing.
No live provider, publication, financial operation or production sandbox is adopted.

## Proposed build gates

1. **Requirements and budgets:** enough specificity to compare options fairly; core journeys, privacy/access boundaries, terminal capabilities, bandwidth, hardware/resource targets, and minimal deployment needs.
2. **Evidence-backed architecture:** primary-source comparisons and recorded decisions for runtime, persistence, terminal access, domain boundaries, and extension isolation. No JS/TS, no mandatory public web service, no central dependency.
3. **Representative experiments:** one named uncertainty and pass/fail observations per experiment, including approval enforcement, safe ANSI/session behavior, bounded resource use, persistence/reconnect, and optional game handoff.
4. **First useful local BBS:** select a small vertical slice only after the earlier gates. Define its exact features, tests, support targets, and deferred work in a milestone. Do not interpret the menu inventory or all open issues as its scope.

## Later, separately scoped work

Original-game compatibility, broader file/mail features, customization, imports, operational tooling, optional/private HTTP access, subscribed shared-chat federation, Reticulum, and mesh/store-and-forward experiments have different dependencies. Research leads may begin early; product support requires later evidence and explicit acceptance.

Shared chat, addressed device-encrypted mail and scoped home-identity proofs may cross boards. Rooms, files, games, directories, local-only channels and local accounts/approval stores remain local; visited boards do not replicate home mailboxes. A separate optional public-publishing module is under research; it is never an implicit export of private core data. Alternative-network research must not make every deployment run every adapter. No release date or production support commitment is set.

Memory/scaling now proceeds under [plan 0014](plans/0014-memory-scaling.md): bounded
credential work, native whole-VM/process/SSH metrics, guarded distinct-account
experiments and measured architecture follow-up. Default service limits and the
full #28 workload/soak/production gates remain separate from short profiling runs.

The next small-host increment compares shared board/readiness ownership (#55)
and calibrated persistent credential peaks (#56); see [ADR 0015](decisions/0015-shared-board-readiness-experiment.md)
and the [comparison](research/0029-shared-board-and-credential-peaks.md). Keep the
full member UI and base OpenSSH approval/isolation behavior while deciding how
to reduce remaining per-transport cost. An isolated prototype is not adoption;
250-user capacity and the broader #28 workload/soak gates remain unvalidated.

[#58](https://github.com/gonz0w/faction-bbs/issues/58) isolates remaining
per-transport costs after the shared-owner comparison; it preserves base OpenSSH,
current approval/credential/terminal denials and all mandatory process accounting.

The [repeatable headroom follow-up](research/0031-repeatable-headroom.md) now
passes three short fresh-boot 250-member menu/chat/credential/control workloads
under the unchanged guard. A supported pageable time-daemon policy leaves
137.23 MiB lowest sampled stress headroom while all base services remain. This
bounded #58 result keeps Rust/base OpenSSH/Argon2 and the normal eight-worker
default. Next build: #55 full shared UI/fairness and scoped candidate adoption;
#20 timekeeping/update/rollback and #28 active mixes, soak, complete budgets and
authenticated-transport admission. Lua/isolated doors follow the useful core.

The [full shared UI/theme candidate](../docs/shared-ui.md) now closes the prototype's
room/reply/retry/sysop/chat and theme/season/plain/resize/editor gaps. Twenty-seven
native UI/fault checks and three short guarded 250-member VM workloads pass with
full SSH accounting. Next #55/#20 work is scoped frontend/owner confinement,
maintained service selection and host-policy/timekeeping/rollback; #28 owns active
mixes, soak, complete budgets and authenticated-transport admission. Normal defaults
remain eight. Lua menus and isolated doors still follow the useful core.

The maintained [shared service](../docs/shared-service.md) now promotes the full UI
into the actual NetBSD rc.d/SSH path, with strict configuration, the small native
terminal entry point, conservative eight-session defaults and worker rollback.
[Plan 0019](plans/0019-shared-service.md) tracks native service-path validation.
The shared implementation advances #55 while stronger confinement, broader #28
workloads and #20 maintained image/timekeeping/update policy remain open. No
service script installs the experimental host profile automatically.

The prior service-integration evidence is [research 0033](research/0033-shared-service-integration.md)
and ADR 0019: three fresh-boot 250-member runs pass with 135.23 MiB lowest sampled
stress headroom, while normal admission remains eight. Native UI 27 and actual
service lifecycle/configuration/rollback 11 checks pass. #18/#20/#28 retain the
maintained image, broader active/soak/admission and timekeeping/confinement gates.

The [native process envelope](../docs/process-confinement.md) advances #55/#19:
peer UID verification, inherited-authority hygiene and kernel ceilings now have
installed-service evidence. Filesystem/network/exec and same-UID isolation remain
explicit next security gates, alongside #28 active/soak/transport admission and
#20 maintained host/timekeeping policy. See [plan 0020](plans/0020-process-confinement.md).

The [experimental OS profile](../docs/experimental-isolation.md) advances #55/#19
with a minimal noexec view, permanent pre-main drop and explicit NPF group/user
policy, preserving original ceilings and default eight. Three clean-boot 250-member
VM runs pass with 132.71 MiB minimum stress headroom; two rejected allocation
controls remain recorded. Independent setuid-code review, broader protocol/
interface/process/same-UID and #28 active/soak/admission budgets plus maintained
host policy remain gates. Lua/doors still need a separate constrained launcher.

The [SSH key-admission follow-up](research/0036-ssh-admission.md) advances #55/#28:
base OpenSSH, dedicated helper UID and persistent process-birth reservations now
cap key admission without a resident supervisor. Native state/restart/abuse and
full UI checks plus three fresh 250-member runs pass the unchanged guard
(129.34 MiB lowest stress). An inetd candidate failed twice and is preserved.
Normal defaults remain eight. Global pre-fork rate limits, repeated key-query
amplification, shared-NAT penalties/fairness, active/soak/complete budgets,
independent authority review and maintained host/timekeeping policy remain open.

The [sustained/abuse follow-up](research/0037-sustained-abuse.md) now passes two
fresh-boot 15-minute active 250-member intervals and original post-load controls.
Keep #55 open: #66 bounds repeated unsigned key-lookup work; #23/#18 retain native
shared-source denial/admission policy, #20 CPU/timekeeping interpretation and
maintained host policy, #28 broader active/long-soak/complete budgets. All historical
measured bytes remain unchanged and normal defaults remain eight.

The [bounded key-lookup follow-up](research/0038-bounded-key-lookups.md) now
limits one monitor to four charged lookups, with explicit quiescent v2 migration,
native adversarial syscall counts and two fresh 250-member matrices passing the
unchanged guard (130.36 MiB lowest stress, zero swap). This resolves #66's
finite helper-work uncertainty; #55 remains open for scoped adoption/independent
review, #23/#18 global admission/source fairness, #20 host/CPU/timekeeping and
#28 broader overlap/soak/complete budgets. Defaults remain eight.

The [global SSH admission experiment](research/0039-global-ssh-admission.md) adds
a listener-only pre-fork gate with fixed durable budget and volatile source
shares, preserving native OpenSSH penalties and all existing BBS/crypto/resource
boundaries. Private authenticated peer addresses are a separate shared-NAT
research direction selected by the owner. Keep #55 adoption/independent loader
review, #23/#18 broader source/admission policy, #20 maintained host/timekeeping
and CPU interpretation, and implementation of #28's overlap/long-soak/budget
contract under #55 open. #28 itself is the closed budget-definition deliverable.

## Later-phase hardening

[#106–#111](../docs/later-phase-hardening.md) capture release/member signing,
minimal VM, post-compromise isolation, storage/backups and independent validation.
These extend #19/#18/#55/#29/#30/#31 and #97/#98; they do not block ordinary product
work or weaken existing safeguards. Software fixtures now, physical claims later.

The next #103 increment implements schema-6 local metadata and headers; see
[upgrade requirements](../docs/post-metadata.md). Actual From/Path/group injection,
authored subjects, broader MIME and peer/provider interoperability remain open.

The next #103 increment implements authored subjects, reply defaults, local article
projection and strict duplicate/conflict comparison. Broader header/MIME/date/
reference policy and independent news software remain open; #101/#104 own actual
peering/provider delivery. Keep #106–#111 hardening queued for its later phase.

#103's bounded local article foundation now includes profile-v2 MIME/UTF-8 wire
support, common dates, root/recent References and original INN offline controls on
macOS/NetBSD. Live news/peer/provider gates remain #101/#104. Internationalized
composer/storage compatibility is #114. Hardening #106–#111 remains later.

Follow-ups: [bounded inline bitmap renderer #117](https://github.com/gonz0w/faction-bbs/issues/117)
and [real emulator/font/multiplexer validation #118](https://github.com/gonz0w/faction-bbs/issues/118).

#117 now adds a narrow opt-in trusted inline RGB preview and a separate native
member gate under [ADR 0042](decisions/0042-kitty-graphics.md). Classic remains
default; rich licensed asset import, real GUI/font/tmux validation (#118), sysop
dashboard and later hardening remain separate.

#118's [real software matrix](../docs/terminal-compatibility.md) now qualifies
twelve pinned profiles and actual native/member-SSH paths. Old screen is
classic-only; tmux keeps image fallback. Physical keys, pixel/artwork quality and
GPU FPS remain unclaimed; richer licensed art continues under #25.

Nightwire #25 follow-through now measures canonical page/menu placement and CP437
widths in real terminal fixtures. See the [compatibility guide](../docs/artwork-terminal-compatibility.md).
Visual font/palette quality and owner artwork acceptance stay separate; no new
capacity/native deployment or hardening claim.

The local community wall (#134, plan 0060) adds the next bounded hangout feature:
ASCII notes, local sysop policy, moderation and explicitly retained weekly editions.
Keep protocol/identity adapters and later hardening separate from this local UI.

The integrated native client now exposes its local offline vault from the private
board picker between visits, explicitly configured by the member. Encrypted
network inbox/send/roaming/attachments remain #90/#91/#88; no server unlock route.
The owner-tested private playground is stopped and retained for later visits.


The #144 curated terminal gallery batch adds owner-reviewed original/licensed
exhibits using existing canonical cells and classic text editions. See
[the guide](../docs/scene-gallery.md) and [plan 0068](plans/0068-scene-gallery.md).
Further gallery submissions/live curation remain optional follow-on work; retain
ordinary product usability ahead of deferred hardening.

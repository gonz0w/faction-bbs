# Faction BBS agent guide

## Read first

**Current priority (owner, 2026-10-22): stop/preserve the tested playground, merge
remaining checked PRs in dependency order and batch useful client/product work.
Integrate the local vault with the current client; keep hardening #106–#111 later.** Native-device
encrypted mail and scoped home identity (#84/#90/#91) remain follow-on work. See docs/file-areas.md,
docs/private-mail.md and ADRs 0030/0031. Pause further game expansion; preserve
optional DOS/Amiga/other-host adapters under #81 without claiming new title support.
The owner selected independent invited home servers and a native local terminal
client. All new private mail must be device-encrypted: no server-readable fallback
or server-side SSH decrypt/render. This explicitly supersedes the older chat-only
federation rule for addressed mail and scoped home-identity proofs. It does not
automatically replicate rooms/files/local-only chat/account approval. Existing local auth stays
until a verified adapter exists; XMPP/OMEMO is a candidate, not adopted SSO.

## Modern operator and communication stories

Latest owner feedback after the SSH playground, 2026-10-21: read
[modern sysop/communications stories](docs/modern-sysop-and-communications.md)
and [research 0047](dev-docs/research/0047-modern-terminal-and-federation-options.md).
Track #95 local OS/VM-console sysop dashboard; #96 capability-based modern
terminal graphics; #97 NAT-aware NPF abuse controls; #98 NetBSD setup/maintenance;
#99 integrated native XMPP unread/private UI; #100 real IRC/IRCv3 reuse/server
links; #101 NNTP-first opt-in room federation, #103 Netnews article model/codec
and #104 optional news-provider subscriptions (read-only first).
These are requested stories/research, not implemented or adopted protocols.

Preserve the modern underground ANSI/BBS identity, sysop themes, improved screen
transitions and formatted chronological room reading (#21/#25). Use actual
capabilities and bounded negotiation/overrides, never brand/TERM alone. A VM
console may lack Ghostty graphics; text-cell graphs/fallback still work. Do not
weaken current OSC/DCS/APC filtering for an unreviewed graphics path or let query
replies become commands/authority. No member content or raw remote art becomes
trusted graphics. Keep no-JS and measured output/memory limits.

The local operator console follows OS login and is separate from member SSH and
the in-BBS sysop role. Start with bounded read-only metadata/health, not secret
content or a root BBS owner. Host/firewall actions need narrow host-authorized
operations and explicit change plans/recovery; no ambient privileged shell,
automatic NPF activation, base source-penalty changes or silent OS tuning. NAT
controls must reflect observed peer/enforcement topology and shared-source effects;
never infer an original IP from user-writable environment/headers/files.

XMPP unread/private chat/mail/files belong in trusted member-side UI with local
keys; no sysop/server decrypt fallback, even for a badge. Keep #90/#91/#88 gates.
Real IRC integration is now a specific reuse research direction; existing local
slash commands are not wire support and IRC server links are not universal
cross-network federation. Public/shared channels do not imply anonymous Internet
exposure, and private/local channels must not leak through bridges.

#101 expands permission to research explicitly shared rooms beyond older blanket
local-only-room wording. The owner selected Netnews-style formatting for
discussion-room posts only;
see [article/news stories](docs/discussion-articles-and-news.md). Keep IRC and XMPP
formats for live/private messages. This selects a representation direction, not a
production wire runtime, room export or public syndication. Local remains default
and room/author/peer consent/history/ACLs must be proven. Keep local
approval/password/role stores local, invited independent
homes, optional networks/private HTTP and separation from #42 public/permanent
publishing. Prioritize the bounded #103 article codec/migration and NNTP
validation against
named interoperability/privacy/NetBSD/no-JS/cost gates; keep XMPP PubSub, Matrix and
ActivityPub as alternatives if those gates fail. Format is not publication. Do not
rewrite historical evidence. The owner requested the local playground stopped
and privately preserved; retain its images, credentials and restart helpers outside
Git. Physical hardware remains unnecessary for software work.

Ordinary running product first. Follow
[the NetBSD quickstart](docs/quickstart-netbsd.md), issue #71 and ADR 0026. Further
combined 250-user edge cases are deferred. Research counts below are historical
observations; old records 035/036/040 now resolve preserved rc-script versions and
must not be presented as measurements of the new normal startup helper.

Read `README.md`, `docs/project.md`, `docs/security-privacy.md`, `dev-docs/roadmap.md`, and the relevant GitHub issue before proposing or changing work. Follow `dev-docs/workflow.md`. This is an early implementation/research project: Rust/primary NetBSD direction is selected (ADR 0006); a local account/room core and development console exist (ADR 0008), with opt-in bounded ANSI menus/declarative themes (ADR 0009), BBS-local bounded chat (ADR 0010), owner-only private export/verify/restore (ADR 0011) and a restricted SSH development gateway with native NetBSD app evidence (ADR 0012). There is no supported network BBS release or accepted complete production stack.
The maintained `crates/faction-service` now promotes the full shared UI into the
opt-in NetBSD rc.d path, using strict root/owner TOML configuration, native
`faction-tty`, base OpenSSH and eight-session/five-second defaults. Worker rollback
remains available. See docs/shared-service.md, docs/service-lifecycle.md and the
previous integration plan 0019. Historical standalone experiments are preserved;
all nine historical native records retain actual executed bytes in dev-docs/evidence,
with two pinned buildable source snapshots. Never refresh old hashes to current code.

Keep the two credential permits spanning queued/active/result lifetime, two kernel
slots, unchanged Argon2, connection-bound opaque authority, current core role/
epoch/grant checks, shared validated themes, lazy encoded output caps, rotating
work and five-second pending-output deadlines. Default remains eight; explicit
250/faster admission settings require experimental_capacity=true. Service scripts
must not silently tune allocator/NTP/sysctls. Base services remain in resource
accounting; shared-owner failure affects all connected members. Stronger process
confinement, maintained host/timekeeping policy and full #28 workloads remain open.

Current native confinement (ADR 0020/research 0034/results 035) passes 29 shared
UI/peer/failure checks, 31 frontend checks, five inherited-authority startup checks,
11 shared lifecycle/rollback, 20 SSH, 13 worker lifecycle and 63 native Rust tests.
Three fresh-boot hardened-service 250-member runs pass; final sampled headroom is
183.25–184.33 MiB, lowest stress 137.61 MiB. Every measured owner/frontend has
verified soft/hard core/memlock/process zero, frontend files 8/additional LWPs 0,
service files `2 * max_sessions + 48`/additional LWPs 3 ceilings. Service starts its fixed
three helpers first. Preserve both kernel peer UID checks, same owned stdio TTY,
descriptor/environment hygiene and strict status+EOF completion. Never relax
these limits to launch doors; extensions need a separate launcher/UID boundary.
See docs/process-confinement.md and plan 0020. Normal-mode filesystem/network/exec isolation and broader same-UID/admission
gates remain open; the separate experimental profile below narrows specific paths. These short software
VM observations are not supported capacity or physical SBC claims. Current result
pins current sources directly; preserve those bytes properly before they become
historical. Both buildable snapshots and all nine old record hashes remain fixed.

The separate experimental isolation profile (ADR 0021/research 0035/results 036)
adds a pre-main C initializer, one common minimal noexec nullfs view, root-only
policy/config sources, permanent privilege drop and explicit dedicated-GID/two-UID
NPF policy. Only its native frontend is setuid; never setuid ordinary frontends or
Rust binaries. Base loader/runtime libraries and the initializer are trusted.
Preserve the numeric fixed sysctl lookup: a named lookup caused avoidable startup
allocation and two 250-member controls failed (126.36/123.47 MiB). Final candidate
passes three fresh boots with 132.71 MiB minimum stress and 178.11–178.84 MiB final
headroom, zero swap and original ceilings. Native 31 frontend/29 UI/16 lifecycle
checks plus live boundary/lock-alias controls pass. See docs/experimental-isolation.md
and plan 0021. Do not turn narrow TCP/UDP/noexec tests into all-family, process,
same-UID or arbitrary-code sandbox claims. Never auto-apply NPF or host tuning;
Lua/doors need a separate boundary. Independent privileged-code review remains.
Both current records 035/036 pin executed sources directly; preserve actual
bytes before changing them. All nine historical records/two full snapshots and
the rejected-candidate overrides remain unchanged.

The selected experimental SSH admission path (ADR 0022/research 0036/results 037)
uses base AuthorizedKeysCommand under dedicated noninteractive factionauth, with
AuthorizedKeysFile none, strict root rc preflight and fixed private process-birth
reservations. No resident admission daemon/proxy/SSH parser. Real-root identity
survives OpenSSH's temporary effective-UID change during key lookup. Preserve
signature verification, one channel, forwarding/forced-command restrictions,
local BBS approval and all existing crypto/credential/resource limits. Default
key cap/burst eight/refill 5000 ms; explicit 250 experiment uses 250-ms key refill
and unchanged 100-ms application refill. This is not a total TCP/pre-fork limit:
base MaxStartups permits up to eight additional preauth connections and base
source penalties remain. Key counters called launches count reservations, not forks.
Never unlink/reset a live registry. Missing state fails closed; root boot init
must prove no member listener/terminal identity/orphan SSH session survives.
Do not auto-lower security.curtain: the prototype requires default visibility.
Helper UID/state are trusted and must never be reused for Lua/doors.

Native 14 key/state/rate/restart/config checks, six channel/restart, 29 shared UI,
31 sealed and 31 ordinary frontend, 20 ordinary SSH, 13 worker lifecycle, six native
PTY flows and 63 Rust tests pass. Silent preauth is bounded at eight and expires.
Three fresh selected 250-member boots pass: minimum stress 129.34 MiB; final
174.66–175.23 MiB; zero swap. The rejected inetd listener completed functionality
but failed two stress guards (125.50/126.04 MiB); preserve its actual sources,
all three trials and earlier live-rlimit instrumentation failure. All 035/036/037
current source pins, rejected overlays, nine historical records and two buildable
snapshots remain authoritative. Preserve bytes before edits, never refresh old
hashes. Independent authority review, preauth/lookup CPU floods, NAT fairness,
complete budgets/soak and maintained host policy remain gates; #55 stays open.

## Room article foundation and paged reading

Owner batch #103/#21/#25: read docs/room-reading.md and ADR 0036. The bounded
faction-articles codec is a bounded text-only profile, independent of board storage,
authority and transport. Reject unsupported headers/MIME/control fields; no invented
legacy subjects/dates, automatic NNTP feed, schema migration or IRC/XMPP conversion.
The ASCII-header/date/body profile is explicit in docs/discussion-articles-and-news.md.

Local/worker and shared-service sessions use the same inert wrapped presenter.
Retain only room/post/body-offset cursors. Re-fetch each screen under current local
approval and file permissions. Never mark a partial message read; the shared owner
marks a completed message only after output drains successfully and not after a
failed close. Keep file-reference tokens contiguous, charging soft-wrap rows, so
native recent-file discovery remains usable. Section clears/colors come only from
trusted theme presentation. No new per-client history cache, owner thread, listener
or relaxed credential/default/session limits. No fresh 250-member claim.

Seventh source snapshot sources/pre90 and mail-client-baseline-manifest.json freeze
the exact 89 original native-client execution inputs and unchanged result before
these edits. This is the same preserved baseline prepared in separate PR #94; it
does not import the native vault implementation. Keep all historical hashes fixed.
The room-reader-netbsd result's 87 executed inputs and original result are now
frozen in eighth snapshot sources/pre103-metadata. Keep all eight snapshots and
twenty old result identities unchanged; never refresh hashes to current code.

## Persistent post metadata (#103)

Read docs/post-metadata.md and ADR 0037. Schema 6 atomically adds a keyed metadata
table and legacy backfill. New-post metadata commits with the post; retries retain
ID/time. Generated subjects are labels, not authored titles. Legacy dates remain
unknown. Public proto-article IDs use a reserved .invalid domain and grant no
membership, encryption, signature, mail delivery or publication authority. Current
approval is rechecked for metadata; headers stay inert/bounded. No new dependency,
owner helpers or credential/default-session changes.

Coordinate a stopped/drained upgrade with verified complete DB/blob/config/key
backup before first owner open. Keep the existing owner-open migration model;
no online/mixed-version promise or in-place downgrade. Exact schema-2..6 snapshot
validation rejects malformed/missing metadata and rotates only private restore
authority. Rollback restores the older complete backup using matching old binaries
in fresh paths; never relabel user_version or unlink live lock files.

## Later hardening backlog (owner, 2026-10-21)

Read docs/later-phase-hardening.md and #106–#111. Keep release/image OpenPGP signing
separate from optional member-held content signatures; no real key publication or
server-side member keys. Compare NetBSD mechanisms, separate VMs and optional
FreeBSD jail hosts; chroot alone is not complete containment. Explicitly threat-model
frontend/worker/door, shared owner, guest root and hypervisor compromises. Local
plaintext readable by an authorized owner remains exposed if that owner is compromised;
offline disk encryption is not live-process confidentiality. Preserve E2EE private
mail, current approval/SSH/credential/helper/session boundaries and emergency
operator recovery. Stories are later phase; no hardening/NPF/host activation or
absolute security claims. Continue normal product development now.

## Ordinary small-board startup

Issue #71 / ADR 0026 prioritize docs/quickstart-netbsd.md. Build/install the
unprivileged non-setuid `faction-daemon` with the normal rc script. It detaches
before exec of the fixed shared/worker backend with fixed environment; there is
no resident supervisor or per-member helper. Preserve real backend pidfile/socket
readiness and live lock inodes. The dedicated SSH script follows LOGIN, retries
only its configured interface for up to 30 seconds, and checks the actual daemon
pidfile rather than the parent's successful return. Never silently bind wildcard,
change host networking, add experimental profiles or start services in setup.
Old 035/036/040 records now resolve preserved rc bytes and have pinned record
identities. They do not claim current boot changes were measured at 250 users.

## Optional Lua arcade

Issue #73 / ADR 0027 adds docs/lua-doors.md: an optional shared-backend Doors
lobby and two original games. Keep the host-only runtime Doors capability;
local console/worker rollback retain disabled Doors. Lua never executes in the
BBS process. factiongames has a separate primary group and cannot read board
state. Both IPC peers are UID checked. Only complete bounded screen/state/choice
frames are accepted, and current local approval is rechecked on actions/results.
The host owns ANSI rendering and session state; no BBS identity/password/session,
DB/TTY/log/listener descriptor or owner-operation API goes to Lua.

Preserve the single evaluator child, Lua allocator/instruction/source/output
caps, native data/stack/process/descriptor limits and parent wall timeout; failed
or partial results are discarded and children reaped. Do not relax the existing
backend process/thread or two credential permits/kernel slots to run games.
The optional service/installer are non-setuid, opt-in, and do not change host
network/NPF/tuning. This is sysop-installed text Lua, not member uploads, a native
legacy launcher or hostile native-code/all-family network sandbox. Lua VM/C/OS
remain trusted. Persistent personal bests/counts now use schema 3 and an opaque one-shot core run;
Lua never selects the member or a SQL operation. Only complete terminal result
screens after a gameplay input can submit one bounded score; approval/epoch is
rechecked in the commit transaction. Never expose game_finish as a terminal RPC.
Keep schema-2 backups readable, exact schema validation and fresh restored board
identity. Scores are local hobby records, not currency or verified game outcomes.
Native/DOS compatibility remains separate work.

## Optional classic server doors

Issue #77 / ADR 0029 adds docs/classic-game-doors.md. Owner accepted original
servers and isolated original DOS binaries; keep both under #34/#78/#79. Do not
rewrite a classic before reuse testing. The connector has owned-fixture
NetBSD evidence and a narrow original LORD 4.06 software proof (research 0042,
docs/original-lord.md). Keep the separate private game-only VM/UID, one active
DOS game globally because record locking is unavailable, independent game tokens
and unchanged original binaries. The external DOSBox receive-state patch and
Python fixture are validation tools, not a production launcher/runtime adoption.
TWGS activation is owner-deferred; no TradeWars gameplay claim. Preserve the old
classic parser under dev-docs/evidence/classic-doors-netbsd/pr80 before future
edits. No local classic runs
under factiond/factionterm/Lua and no process/thread/credential limits are relaxed.

Only trusted-config literal endpoints (six maximum), no DNS/member-selected
address, and explicit destination/legacy-link confirmation before connect. Lua
can request only the host list/confirmation, never change a destination or send
BBS authority. The original game runs on a separate host; bounded safe Rust
Telnet/CP437/basic-ANSI parsing runs in the shared owner and is not a process
sandbox. Preserve all buffer/byte/deadline/terminal-control gates, eight active
remote sockets, current approval and Ctrl-] local return. Remote saves/scores
stay remote. No JS/TS dependency, mandatory helper or auto-enabled public service.
Keep the Lua installer root keys before any TOML array tables. No experimental
network policy/tuning is automatically changed for remote games.

## Native local mail vault

Issue #90 / ADR 0035 adds docs/local-mail-vault.md. Explicit local command only,
before any SSH profile/child; no terminal-control unlock or remote compose/pin
requests. Keep owner-only descriptor-relative no-link storage, exclusive live lock
inode, authenticated bounded format, atomic writes/create-only backup, hidden
input, five-minute inactivity/cancel/error relock and core-dump denial. Random
local data/recovery keys and Argon2id64MiB/t3/p1 run on the member device; server
passwords/credential limits are unchanged. Old backups keep old secrets and pin
state; no rollback/revocation/memory-erasure guarantee. Local address claims/pins
are not authenticated home identity, remote device enrollment or implemented
OMEMO state. Inbox/send/reply/interop/history remain #90, trusted live membership
binding #91, native signer failure #92 and encrypted attachments #88. Do not
adopt research Python or an unreviewed ratchet to label this complete. Preserve
#89's 89 measured inputs via mail-client-baseline-manifest.json; old results
never become measurements of this revision. See plan 0036 and research 0046.

## Product constraints

- Preserve the WWIV-inspired approved-login -> BBS-specific ANSI art -> sysop-defined welcome/menu session. No community access before authentication and local sysop approval.
- Use Citadel-style message rooms, IRC-style command-driven chat, BBS menus, local file areas/mail, and optional isolated game doors. Existing games are reuse candidates, not implemented support claims.
- Keep each sysop's identity, accounts/approval, content, files, games, directory, and local-only chat local. Explicit shared chat, addressed device-encrypted private mail and scoped home-identity proofs are future directions; #101 separately researches opted-in shared rooms. No automatic local-data/account replication or room export follows from that research.
- Ordinary SSH client access with optional invitation-only private-network mode is accepted product direction; network admission/SSH identity never implies local BBS approval or host shell access. A separate terminal UID/frontend, trusted broker and member worker rollback now pass real NetBSD application SSH oracles. SSH keys grant admission only; local BBS identity/password/approval remain separate. Use the opt-in NetBSD rc.d scripts and bounded broker lifecycle (ADR 0013); never unlink a live pidfile lock. Keep the eight-session default and owner tools out of the frontend; higher worker/launch limits are explicit owner-only profiling experiments (ADR 0014), never an implicit production capacity claim. Credential work has two cross-process slots per board; never unlink live credential lock files; complete production security/resource/deployment gates remain.
- Direct network-terminal access is central. A browser portal/console is a goal that must be optional and may be private or disabled. Do not assume public HTTP exposure.
- Privacy, security, exportability, low bandwidth, and lightweight operation are requirements. Use minimal opt-in telemetry; never log message bodies, credentials, or private content by default.
- No JavaScript/TypeScript in source, tooling, dependencies, or browser-terminal assets. Do not introduce Node/npm, JS terminal libraries, or JS-based CI actions. Browser-console feasibility must be researched within this constraint.
- Prioritize tiny working sets and bounded per-session work. The owner research target is 1–2 cores / about 1 GiB host RAM and 100–250 connected members; do not treat host capacity as application allowance or claim supported capacity. Compare low-memory native options, selective assembly and specialized systems with equivalent security/behavior. External helpers/blockchain are research candidates, never implicit permission to publish private data or add a mandatory service.
- Newer and older technology is welcome on merit. Prioritize a practical hobby SBC/Pi or VM appliance; MINIX is a candidate, not a mandatory port. Compare whole runtime footprint and update/recovery costs. The owner selected Rust and NetBSD as primary target (ADR 0006). NetBSD 11 ARM64 has narrow native Mio/rusqlite/OpenSSH compatibility evidence. Readiness + SQLite WAL remains the next integrated validation slice (ADR 0005), not an accepted complete production stack or all-port support claim.
- Evaluate fast, lightweight language/runtime options comparatively. Do not choose a stack, storage engine, protocol, federation topology, or scripting runtime from familiarity.
- Keep community behavior independent of addressing, discovery, routing, connection mechanics, and optional transport adapters. Mesh/non-IP support remains future research.
- Honor the ANSI/ASCII, underground scene, and sysop customization direction without trading away access control, accessibility, or a usable plain-text baseline. Oblivion/2-inspired arrow-key highlighting + Enter and direct displayed hotkeys are owner requirements (`[D]oors` -> D/d); both routes use the same authorized action. Themes are sysop-defined, including optional seasonal palettes/login identity screens. Use `docs/menus-and-themes.md` and the attributed #25 art reference catalog: ACiD/iCE ANSI and Razor/scene ASCII typography inform art direction; generic block-font placeholders are not approved identity art. Never bundle archived work without rights/provenance.

## Current sequencing

- Owner steering on 2026-10-21: prioritize a running product for ordinary users;
  defer further edge-case and combined 250-user research. Start from the ordinary
  eight-session shared SSH/ANSI/rooms/chat path in docs/quickstart-netbsd.md and
  issue #71. Improve setup and member/sysop workflows, then extend the optional Lua arcade and work toward isolated legacy
  doors. Keep existing approval, credential and access protections. Experimental
  isolation/admission/host-tuning adoption is not a first-preview prerequisite;
  preserve incomplete #70 findings without claiming its full matrix passed.

- Physical SBC test hardware is unavailable. Build/research software on developer hosts and NetBSD VMs now; the owner will load a complete product onto hardware later. Physical tests gate hardware-specific claims, never near-term software work. Keep resource measurement and recovery/security tests.
- Build the first useful local core first (`dev-docs/first-local-slice.md`, #35). Lua menus and isolated game doors are the first extension priority after it (#10); a boundary probe is evidence, not a production sandbox.
- Optional permanent/public rooms, ANSI stores/payment adapters, blockchain games/wagers and modern networks are active future research. Compare both external-member-wallet and separate sysop-service models before choosing; no provider/custody/currency adopted. Follow `docs/extensions.md` and ADR 0007.
- The owner selected explicitly addressed encrypted private mail and scoped home identities in addition to optional shared chat (ADRs 0030/0031). Native member-side keys/decryption are required. A public-publishing module remains a separate explicit outbound category under research, never authority to export private local content. Keep external data, publication, payment requests and spending privileges distinct; no live/funded external actions are implied by prototype authorization.

## Work and documentation

- GitHub Issues are the canonical backlog and task status. `docs/` holds current product/member/sysop information; `dev-docs/` holds engineering, research, decisions, plans, and roadmap context.
- Historical records under `dev-docs/archive/` are read-only context, never current instructions or an active backlog. The old planning workflow and slash commands are retired. Do not recreate a `.planning/` directory.
- Use a short issue-linked branch and focused PR. For multi-step work, keep a self-contained execution plan under `dev-docs/plans/` using the template; routine edits do not need a plan.
- For material decisions, record the question, constraints, alternatives, primary sources with retrieval dates, tradeoffs, confidence, reversal cost, and falsifiable validation. Separate product decisions from untested technical hypotheses.
- Update `README.md` status, affected docs, and `CHANGELOG.md` in the same change when scope, behavior, setup, support, or stage changes. Review README freshness on every PR. Do not claim security, game, network, or platform support without evidence.
- Do not expand an issue into a full platform rewrite. Capture additional work in an issue and cross-reference existing work before duplicating it. Future candidates are not automatic first-release commitments.

## Checks and safety

- Run `python3 tools/check_repository.py` for repository/documentation changes. Run the smallest meaningful additional checks for the affected prototype/code once introduced; report unavailable checks plainly.
- Never commit credentials, actual deployment endpoints, user data, private artwork, generated secrets, or unlicensed third-party assets. Public repository visibility must not imply public deployment visibility.
- Keep legacy games/extensions optional, permission-bounded, and failure-isolated. Treat input, file uploads, terminal control sequences, and remote channel traffic as untrusted.
- During review, prioritize approval bypasses, accidental public exposure, local-data federation, JS/TS dependencies, unsafe terminal/file handling, and unsupported status claims.

## Sustained workload evidence

Research 0037/results 038/plan 0023 add two fresh-boot 15-minute active 250-member
intervals with 423,798 confirmed peer deliveries, 23 UI/post/input-burst cycles
and 11 fresh real logins each. Minimum active 152.97 MiB; minimum stress/control
133.13 MiB; zero swap; all original limits/defaults/crypto and prior matrices
remain. Actual first fixture has 1002 accounts/52 rooms/100004 non-load posts from
UI oracles; retain first-trial posts across reboot, then use only the new bounded
synthetic reset variant. Never use these reset tools on a real board.
The historical result-038 helper accepts 64 unsigned queries on one reservation;
the bounded lookup update below amends that behavior for #66. Native penalties temporarily deny legitimate same-
source callers; existing sessions survive. CPU tick/runtime counters disagree;
retain raw calibration and withhold CPU-budget/throughput acceptance (#20/#28).
Current 035/036/037/**038** source pins, nine historical records, two buildable
snapshots and rejected overrides are authoritative. Preserve actual executed
bytes before edits; never refresh old hashes. No long-soak/supported-capacity claim.

## Bounded SSH key-lookup update

ADR 0023/research 0038/result 039 limit a real-root monitor birth to four durable
helper allowances. Persist before key IO; exhausted calls emit no keys and skip
writes/fsync/key reads/cohort liveness queries. Keep fixed state/identity validation,
parent/visibility checks and base MaxAuthTries 3. Do not claim a global TCP/fork
rate bound. Registry v2 is 8104 bytes; old v1 requires explicit stopped/drained,
inode-preserving --migrate-v1 through root preflight. Never silently reset or mix
helper versions. Keep helper UID separate from extensions and all original crypto,
local approval, credential permits/locks, process ceilings and default eight.
Eight native known-key adversaries measure 56 helpers/32 reads+commits; signed
failure/unknown-key each 24, no authentication. Migration, exhausted-restart,
options/ordinary clients and two fresh 250-member matrices pass (130.36 MiB
minimum stress, zero swap). Active intervals explicitly use 250/249/250 members
for useful chat plus fresh source-recovery checks; no reset/penalty weakening.
The third snapshot sources/71491cc now freezes all 137 original 037/038 inputs
and both records; original two snapshots/nine records/overrides remain unchanged.
035/036 still pin current unmodified inputs. Result 039 originally pinned 145 current sources; the global update below now
preserves them in the fourth snapshot. Never refresh old hashes.
Keep #55 adoption/independent review and #23/#18 global admission/source fairness,
#20 CPU/host/timekeeping and #28 complete overlap/long-soak budgets open.

## Global listener gate experiment

ADR 0024/research 0039/result 040 add an opt-in listener-only C preload candidate,
not production adoption. Keep original sockets, native source penalties/netblocks/
exemptions, default eight, key helper v2/four lookups and all BBS/credential/crypto/
resource guards. Root preflight pins exact base SSH/module bytes. Global state is
120 bytes under private `/var/db/faction-gate`, fsynced before return to the base
fork path; source table is 32 volatile entries (burst two, double global refill).
Normal global burst eight/refill 5000 ms; faster settings require explicit opt-in.
Never use `/var/run` for this persistent budget: its initial candidate failed
reboot recovery and is preserved. Never delete/repair/reset live authority. HUP
stops the listener; explicit restart retains authenticated key reservations.
The library/environment must disappear at session exec; independent loader/
privileged-code review and upgrade validation remain adoption gates. No proxy,
resident rate daemon or member-side library is introduced. One NAT is still one
source; owner selected unchanged penalties plus optional authenticated private
identities, not exemptions. Native wg is experimental; no provider/250-peer
support is adopted. Fourth snapshot `sources/5b62d0a` now freezes 145 original039
inputs and its entire record; four snapshots/twelve historical records stay fixed.
035/036 still pin unchanged current bytes. Preserve new040 executed bytes before
future edits; never refresh an old hash.

GitHub #28 is closed for its budget-definition deliverable. References above to
its remaining workloads mean implementation of that contract under #55/#18/#20/
#23, not reopening #28 or claiming its original definition work is unfinished.


## Optional file areas (#83)

Read docs/file-areas.md and plan 0033. Schema 4 stores local catalog, policy,
reservations and short-lived key-bound one-use transfers. factionfiles streams
only opaque objects under its own UID; factionfileauth can only look up a current
ready key. Neither opens the board DB. Preserve UID-checked bounded policy IPC,
current approval/epoch/area/object checks on each request/chunk, private charged
partials, immutable publication and explicit two-lock GC. Only sysops may inspect
completed pending bytes. No member-selected host paths, exec, archive extraction,
resume, new owner thread/process, credential change or automatic network exposure.

The first installer leaves separate SSH disabled and binds loopback by default.
File listeners/children must drain before backup/upgrades. Database exports include
catalog but no file bytes and discard transfer grants; use the complete stopped/
drained catalog/blob tool for file backup and fresh-identity restore. Never unlink
live slot0/slot1 or owner PID locks. Physical hardware/250-user capacity and hostile
native-code/all-family confinement remain unclaimed.

The fifth source snapshot sources/pre83 freezes 220 old execution inputs before
schema/theme/owner changes. The independently pinned file-areas baseline manifest
also pins original LORD/classic records. All prior snapshots/result identities
remain unchanged; new evidence validates current executed inputs directly.

## Native terminal transfers and local references (#87)

Read docs/native-client.md, ADR 0033 and plan 0034. Client-only openpty/libutil and
OpenSSH execute on the member device, never the owner. Keep strict local profile/
verified host keys, no shell/endpoint/path/action from remote text, bounded terminal
filter/control frames and explicit local transfer confirmation. Files control binds
the approved connection, nonce and ordered sequence; it does not import member IDs,
roles or mail keys. Preserve scoped SFTP/current permission checks and owner limits.

Schema 5's public random file namespace is distinct from private rotating session
identity. References grant no authority; validate sending, render per viewer and
recheck opening/downloading. Restore retains links and rotates session authority.
Malformed reference prose remains readable; at most four recognized links/message.
Native cancel abandons only the member's own current grant; upload bytes remain
charged/private pending explicit sysop remove/GC. Mail/identity/attachments remain
#84/#88 research; current file areas are sysop-readable, never E2EE storage.

Sixth snapshot sources/pre84 freezes the 74 actual merged-#86 file-area inputs and
pins its whole original result. Never refresh old hashes or counts to current code.
Current native-client evidence is separate; no new 250-member or physical claim.

## Encrypted-mail and roaming research (#84)

Read research 0045, ADR 0034 and plan 0035. The separate Python reference oracle
uses unchanged MIT OMEMO/Twomemo crypto and a bounded SCE adapter; it is **not** a
new product runtime/library adoption. Twelve actual two-home mail controls and
11 native certificate-verifier controls pass on software fixtures. The optional
Slixmpp Rust/PyO3 graph has a prohibited JS asset and remains rejected; its upstream
Python fallback is for reference testing only. No maintained runtime source/schema/
SSH/default/owner/credential ceiling changes in this increment.

Native mail UI, encrypted keystore, fingerprint/enrollment/recovery and independent
interop are #90. Trusted issuer/connection-to-local-membership binding and online
issuance are #91; neither typed claims nor user-writable SSH_USER_AUTH/environment
is sufficient authority. Keep local approval/revocation and ordinary login. Native
ssh-keygen signing exits SIGSEGV after emitting a certificate in four controls (#92).
Never accept its artifact as success; externally minted control certificates prove
only native verification. No server mail keys/plaintext, role import, custom
cryptosystem, automatic public endpoint or independent audit/capacity claim.
Device-encrypted attachments remain #88, separate from sysop-readable file areas.
All previous evidence/source identities remain unchanged; preserve current record
bytes before future source edits rather than refreshing historical measurements.

## Authored subjects and local article projection (#103)

Read docs/room-articles.md, ADR 0038 and plan 0040. Keep schema 6, borrowed PostDraft,
subject-aware retry comparison, optional authored ASCII subjects and previewed Re:
defaults. Do not overwrite generated/legacy provenance or invent dates. Local
projection rechecks current approval and file permissions; it returns exact bodies
or fails, never per-viewer rewrites under one ID. Per-article reserved author,
local numeric group and not-for-mail aliases are not public/home/peering identities.
Traverse at most 128 same-room descending ancestors; emit root plus fifteen recent
IDs. Reject cycles/deeper work rather than traverse unbounded history. No remote insertion, authority from headers,
automatic export or network adapter. Strict normalized duplicate/conflict matching
includes Path; future relay semantics need explicit validation. All existing owner,
credential/session/helper/default limits stay. No new capacity or full-wire claim.

Ninth source snapshot sources/pre103-authored and post-metadata-baseline-manifest.json
freeze #112's exact 90 execution inputs and original record before these changes.
Preserve all nine snapshots and twenty-one historical result identities unchanged.
Requested OpenPGP/VM/storage hardening remains later phase; do not activate host,
firewall or signing operations while continuing normal product work.

## Broader local article profile and original INN (#103)

Read docs/article-profile.md, ADR 0039, research 0048 and plan 0041. Profile v2
supports bounded decoded UTF-8 subjects, numeric-zone dates and single text/plain
MIME. Use reviewed base64, canonical QP soft breaks and normal final-newline
semantics; no invented LF or body rewriting. Keep v1 historical bytes/results
unchanged. Current schema-6 composer/storage validation stays ASCII; #114 owns
versioned Unicode composing. No silent schema widening or imported member authority.

INN 2.7.4 is an optional original reference build, not a product runtime or server.
Pin/review its source and runtime artifacts. Its -D still resolves a hostname and
opens NNTP: force synthetic INN_HOSTNAME and an owned temporary loopback fixture
that emits only 400 and verifies zero client commands. No real boards/providers,
credentials, posting, public listener or changed host/NPF policy. Perl is only an
upstream reference-build tool, not a new BBS dependency. INN accepts a deliberately
invalid Date; Faction must reject it independently. Live transport, relay semantics,
consent/history and resource budgets remain #101/#104; hardening stays later.

Tenth snapshot sources/pre103-broader and authored-articles-baseline-manifest.json
preserve the exact 95 #113 inputs and original result. Keep all ten snapshots and
twenty-two historical result identities fixed; never refresh old measurements.

Eleventh snapshot sources/pre114 and broader-articles-baseline-manifest.json freeze
#115's exact 100 executed inputs and original result before Unicode composing.
All eleven snapshots and twenty-three historical result identities stay fixed.
The optional INN build explicitly uses /usr/bin/false for sendmail; no mail service
is installed or contacted. Linux CI needs the explicit path even in offline mode.

## Unicode composing (#114)

Read docs/unicode-subjects.md, ADR 0040 and plan 0042. Current writers use schema 7;
never widen schema-6 ASCII validation or manually relabel a store. The 6-to-7
transaction checks the exact metadata catalog and original rows, then preserves
all IDs/subjects/timestamps/origin while enforcing UTF-8 byte length. Old exports
remain read-only and require their original contracts; rollback uses a complete
matching older backup/binary in fresh paths. No mixed versions or online upgrade.

Both local/worker and shared subject input use faction-terminal::subject, at most
160 bytes with incremental UTF-8 validation, scalar/pending-tail backspace and
terminal restoration/failure closure. Reply clipping is at a UTF-8 boundary and
keeps a single inherited Re: prefix. Non-ASCII/control previews remain inert
ASCII escapes with an independent cell bound; this is a compatibility display,
not native Unicode glyph/graphics detection (#96). Do not trust TERM/brand as
permission for richer rendering. Keep original bodies, retry keys, approval/epoch,
owner helpers, credential/default limits and all eleven frozen snapshots/results.
Hardening #106–#111 remains deferred. No INN/provider, firewall or host changes.

## Native text and capability preferences (#96)

Read docs/terminal-capabilities.md, research 0049, ADR 0041 and plan 0043. Classic
is default. T opens session-only independent preferences; never trust TERM/brand
for rich features. The common native scalar renderer uses pinned unicode-width
without default features; ambiguous=1, invisible/formatting/cluster-dependent
forms escaped. Body offsets are bytes in one unchanged rendering mode. Preserve
cell clipping, UTF-8 boundaries, contiguous file tokens and output-drain unread.

DECRQM sync probing is explicit and isolated after approval: nine query bytes,
32-byte response ceiling, 300 ms after output drains, one exact reply. Extra/
malformed/incomplete/late control input never becomes menu/posting commands.
Unknown/no reply stays conservative. Sync brackets complete bounded frames and
releases during cleanup. Keep fixed helpers, credential/default/session bounds
and 32-KiB/five-second output rules. Do not relax admission for test fixtures;
respect normal eight-launch burst and five-second refill.

Native client defaults to filtering queries/sync/APC/DCS/clipboard; Unicode/color use
explicit preference there. Mouse, extended keyboard and general bitmap passthrough stay disabled.
#117 separately adds the explicit fixed-asset inline preview below; no arbitrary
art or file/shared-memory/network paths.
Native chat/grapheme editing, Unicode theme text and rich asset formats remain
separate. No host/firewall/hardening change or fresh capacity claim.

Twelfth snapshot sources/pre96 and unicode-subjects-baseline-manifest.json freeze
all 101 original #116 build/run inputs and unchanged result before this work.
Preserve twelve snapshots and twenty-four historical result identities unchanged.

## Optional Kitty preview (#117)

Read docs/kitty-graphics.md, research 0050, ADR 0042 and plan 0044. Classic stays
default. T/G has dedicated positive 300-ms/128-byte probes; no report becomes a
command/authority. The native gate needs local `--kitty` plus a local positive
probe before SSH and regenerates only exact original-asset chunks/query/delete.
Never allow arbitrary APC, paths, PNG, compression, shared memory, network fetch,
clipboard, Sixel or remote/member/game art. Sync remains filtered natively.

One static read-only 38,400-byte RGB asset, 51,410 wire bytes, thirteen <=4-KiB
commands and one frame/second. No per-member image cache or new helper/dependency.
Shared chunks advance only after drain under rotating work and current approval;
32-KiB/five-second queue plus total upload deadlines remain. Native gate checks
order/rate/unfinished timeout. Cleanup terminates partial APC and deletes only the
Faction image ID; broken transports may need reset. Keep original theme import
rules, admission eight/5000 ms, fixed helpers/crypto/schema and hardening later.

Thirteenth snapshot sources/pre117 and modern-terminals-baseline-manifest.json
freeze all 106 exact #119 inputs and the unchanged original record. Preserve
thirteen snapshots and twenty-five historical identities. No actual GUI/font,
tmux, hardware or fresh capacity claim. #118 owns real emulator certification.

The kitty-graphics-netbsd record now pins 112 final executed inputs directly.
Preserve those actual bytes/result before future runtime changes, never refresh
its hashes. 151 Rust tests/host, seven graphics plus three native-gate PTY controls
per host and nineteen shared native controls pass; the original private saved
playground remains off and only the disposable clone was removed.

## Real terminal compatibility (#118)

Read plan 0045. Research uses privately isolated, pinned native emulator/reference
fixtures, not a new product graph. No screenshots/desktop capture permission is
available; distinguish real report/cell/image-retention observations from visual
pixel or font-glyph certification. Never reuse/control existing user terminals,
load personal configs or replay unexpected human input into BBS commands.

Fourteenth snapshot sources/pre118 and kitty-graphics-baseline-manifest.json
freeze all 112 actual #120 inputs plus its unchanged result before compatibility
work. Keep fourteen snapshots and twenty-six historical identities fixed.
Normal service eight/5000-ms admission, crypto/helper/output/probe limits and
classic defaults stay. Software VM/member SSH/native client tests use only a
new synthetic board and new loopback forwarding; saved playground stays off.

The terminal-compatibility result now pins 115 application inputs and seven host
harness sources, with two separate reproduction sources. Preserve those bytes
and results before future changes. Twelve actual profiles/48 BBS routes pass
with three old-screen profiles explicitly classic-only; native default filters
and no-DCS-passthrough stay. Do not turn this into pixel/glyph/GPU certification.
No product runtime/dependency/filter/schema/default/limit changes were needed.

## BBS menu polish (#25)

Read docs/theme-packs.md and plan 0046. Both UIs pass actual TTY columns into the
same pure menu: 38-cell panel at 40–79 columns, 76 cells at 80–240; at most ten
entries fit 18 rows. Keep fixed recovery instructions, visible `>`/unavailable
markers, full lightbar background and bounded two-row focus redraw. New version-1
base-only menu_title (24 ASCII bytes) and entry description (30 bytes) are inert;
seasonal packs never remap them. Older packs default, older binaries reject new
fields atomically. Local authoring previews show services enabled only for layout;
never treat their output as actual availability or native visual certification.
No new authority, dependencies, schema, owner helper, per-member history cache or
changed queue/drain/admission defaults. Original/licensed scene identity art remains later #25 work; calendar scheduling
is now described below. A framed menu is not accepted identity art.

Fifteenth snapshot sources/pre25 and terminal-compatibility-baseline-manifest.json
preserve all 124 distinct #121 application/harness/reproduction inputs and its
original record. Keep fifteen snapshots and twenty-seven historical identities
fixed. New presentation measurements must not relabel the old twelve profiles.

The menu-themes-netbsd record pins 116 selected candidate build/run inputs,
three new host harness sources and three generated SVGs. Preserve all of those
actual bytes and the unchanged record before future edits. Software checks pass
154 host/native Rust, 109 host Python, 22 shared native controls and three actual
restricted SSH/native menu routes. These are not a fresh emulator/font, resident
memory, 250-user or physical-hardware certification. Keep the saved playground off.

## Seasonal login calendars (#25)

Read plan 0047, ADR 0044 and docs/theme-packs.md. Themes preloads base plus at most
8 annual overrides; each approved shared client pins only theme_index. Pass that
borrowed theme through all UI/background paths. Local/worker pin the selected
borrow for the complete session. No midnight/draft/door mutation, filesystem
reload, extra calendar thread/helper or member-selected dates. Manual season
bypasses schedule; disabled/no-match/unknown clock uses base. Explicit bad shared/
worker calendars fail before binding/access; local errors preserve the valid base.

Schedule uses version 1, explicit UTC/system timezone, start-inclusive/end-exclusive
MM-DD, higher priority wins, and overlapping equal-priority windows reject. Pack
IDs are unique simple children; validate all enabled assets before binding, with
existing ownership/encoding/size/control gates. Date is presentation only. The
existing native bridge now has a scalar/output-pointer calendar API; initialize
system-local timezone before shared credential helpers, never mutate TZ/clock or
spawn date processes. No new external registry package; active experiment locks
need the new internal faction-terminal -> faction-unix edge only.

Common identity presentation uses full imported art at ANSI 80x24, fitting ASCII
at smaller/plain geometry, or board-name fallback. Preserve approval before all
screens, termios/input boundaries and fixed 32-KiB/five-second output rules. New
seasonal templates are original Codex-assisted BSD examples, not final approved
scene identity art. Shipped calendar stays disabled; installation is explicit.
Hardening remains later and saved playground stays off.

Sixteenth snapshot sources/pre25-calendar and menu-themes-baseline-manifest.json
freeze all 122 distinct #122 build/harness/SVG inputs and its unchanged record.
Keep sixteen snapshots and twenty-eight historical identities fixed; never refresh
old measurements to these new runtime/calendar/asset bytes.

The seasonal-themes-netbsd result pins 125 selected candidate build/run inputs,
five host harness sources, the actual native seasonal driver and three new SVGs.
Preserve those actual bytes and its unchanged record before future edits. Final
checks pass 162 Rust per OS, 110 host Python, nine seasonal controls per OS,
eight actual shared SSH/native calendar controls and 22 shared regressions.
Compact ANSI identity cards wait for Enter before a menu clear; plain CLI and
board-name-only compact flow stay conservative. No clock/TZ/host activation,
new capacity or final artwork/emulator certification is claimed.

## Custom menu layouts and SAUCE/iCE (#25)

Read plan 0048, ADR 0045 and docs/theme-packs.md. Optional base-only layout=grid
requires unique explicit row 1–5 / column 1–2 coordinates for every entry. At 80+
columns arrows are spatial; Up/Down wrap within a column, Left/Right prefer the
same/nearest enabled row in the other column (upper row breaks ties). Tab keeps
array order; compact/plain keep ordered fallback. Shared focus must restore its
entry index directly, never replay Down. Core approval/actions and host Files/
Doors enablement stay unchanged; seasons cannot move/remap entries. All art/theme
memory remains in the shared startup catalog; no per-member grid/art allocation.

SAUCE00 Character/ANSi only, width 0/80, height at most 16, total input/output still
16 KiB / 80x16 cells. Metadata/EOF/comments never render; optional stale FileSize
is ignored. Non-blink flag alone permits SGR5/25 as bright-background changes,
never terminal blinking. Font/spacing/aspect hints confer no capability or rights.
Raw blink, oversized/animated/other formats and unknown controls reject. The new
grid template is original Codex-assisted BSD sample art, not accepted final scene
identity. Previews accept only ASCII plus the known CP437 glyph set.

Seventeenth snapshot sources/pre25-layouts and seasonal-themes-baseline-manifest.json
freeze 133 executed #123 inputs and its whole original result record. Keep all
seventeen snapshots / twenty-nine historical record identities unchanged.
No new capacity/hardware/emulator/pixel certification; hardening remains later.

The custom-menus-netbsd record pins 130 final selected native input matches,
seven harnesses and two SVGs. Preserve their actual bytes before future edits.
Validation passes 167 Rust per OS / 111 host Python, four local/worker custom
controls per OS, four actual shared SSH/native custom controls, nine seasonal
controls per OS, eight actual shared calendars and 22 shared regressions. Full
menus remain <4 KiB, focus <=512 bytes; no new capacity or final artwork claim.

## Long-form login art (#25)

Read plan 0049, ADR 0046 and docs/theme-packs.md. identity_ansi remains the existing
80x16 / 16 KiB importer. Optional mutually exclusive identity_ansi_long explicitly
selects <=80x256 cells / 64 KiB total file; all pages validate at startup and each
16-row canonical page remains <=16 KiB. No concatenated whole-art member output.
SAUCE dimensions use the selected profile; metadata/body controls/rights remain
bounded and separate. The shared catalog owns the complete cell assets; sessions
retain only an identity-page index in the existing state enum. Startup may use a
bounded full canvas temporarily, then trims it to actual/declared extent.

Only approved wide ANSI sessions page. Enter advances, S+Enter skips to a fitting
ASCII welcome; final art leads to the existing welcome/Enter/main-menu journey.
Plain/compact keep their existing ASCII path. Shared page commands and resize
recheck approval; shared redraw waits for pending output to drain and clears
input. Local/worker recheck approval at page boundaries and fall back on resize
at the next boundary. Output backpressure, five-second drain, one command frame,
typeahead flushing, normal eight sessions and role/host gates remain. No new
external packages, owner helper, clock operation, schema or game/hardening change.
The long template is original Codex-assisted BSD sample material, not final scene
identity or an archived-art permission grant. Saved playground stays off.

Eighteenth snapshot sources/pre25-paging and custom-menus-baseline-manifest.json
freeze 137 executed #124 inputs and its whole result. Keep eighteen snapshots and
thirty historical record identities unchanged. New paging validation must pin its
own executed inputs, never rewrite old measurements.

The login-paging-netbsd record pins 135 selected final native source matches,
nine host harnesses and three SVGs. Preserve all those actual bytes before future
edits. Validation passes 171 Rust per OS / 112 host Python, eight paging PTY
controls per OS, eight actual shared SSH/native paging controls, old menu/seasonal
controls and 22 shared regressions. No new capacity/font/final-art acceptance.

## Original Nightwire artwork (#25)

Read plan 0050 and docs/artwork-nightwire.md. New opt-in Nightwire long/short and
seasonal packs use original Codex-assisted CP437 cells and bespoke letter polygons.
The builder/checker are offline authoring tools, never runtime scripts. Default
packs/installer/runtime and all previously measured input bytes stay unchanged;
verify all saved evidence guards. No new snapshot is needed for new assets alone.
If editing any pinned input, preserve it before the edit under the existing rules.

Long set is three 16-row pages; short set is the same complete cover. Autumn/
winter preserve glyph geometry and menu action/key/position maps while changing
palette/welcome/compact card. Compact/plain stay printable ASCII. Record rights
and actual provenance, regenerate/check the declared output set and inspect the
canonical product previews; parsing tests cannot grant artistic acceptance or
font/emulator certification. Do not borrow archival art or use generic logo fonts.
No runtime/dependency/host-policy changes or new capacity claim. Hardening later;
keep the preserved playground off and use owned disposable software fixtures.

The nightwire-netbsd result pins 157 final native input matches, five harnesses,
twelve new art files and seven SVGs. Preserve their actual bytes before future
edits. Eighteen render profiles/seven local routes per OS and seven actual SSH/
native routes pass; 171 Rust per OS / 112 host Python pass. All 144 prior paging
inputs remain unchanged, with no new snapshot/runtime/default/dependency change.
Owner art acceptance, actual font/emulator review and new capacity remain unclaimed.

## Nightwire real-terminal validation (#25)

Read plan 0051 and docs/artwork-terminal-compatibility.md. New offline fixture
tools measure canonical compiled output with actual emulator cursor reports.
Keep product/runtime/theme bytes unchanged unless a defect is demonstrated;
preserve pinned bytes before any edit. At least 80x24 usable content is required
for full art, including inside a multiplexer; compact/plain remain recovery.
The private tmux fixture disables only its own status bar. Real 40x18 windows
exercise compact cards/menus separately. Raw response TTYs must explicitly model
the normal output newline processing preserved by the product adapters.

Use only distinct private reference bundle IDs, owned windows/processes and
checked reference artifacts. Never modify installed apps, personal terminal or
multiplexer settings, or request desktop capture/automation permission for this
fixture. Cursor widths and canonical SGR checks are not pixels, glyph seams, actual
font-face/palette certification or owner artwork acceptance. No new native VM,
capacity, production support or hardening claim; saved playground remains off.

The nightwire-terminals record pins 167 unchanged original Nightwire inputs and
ten actual probe/reference harness sources. Fifteen profiles pass 240 frames /
2,476 cursor checkpoints / 187 repeated-glyph observations. Actual missing-font
Alacritty was 80x20 and ran only compact authoring controls; do not claim automatic
BBS menu-layout selection or full-art/font coverage for that profile. All product
inputs and old result records remain unchanged, so no new snapshot or native run.
Keep eighteen snapshots/thirty historical identities and all thirty-two saved
evidence guards. Preserve these new measured harness bytes and the whole result
before future edits; never refresh historical hashes.

## Local operator console batch (#95/#18/#98)

Read plan 0052, ADR 0047 and docs/sysop-console.md. `operator_status` is explicitly
disabled by default; enabled shared owners publish only bounded numeric telemetry
in their existing private state directory every five seconds. No new listener,
owner helper or session/credential limit. SQL counts must retain progress/zero-busy
budgets and restore prior hooks/timeouts; unavailable is not zero. The separate
`faction-sysop` command reads only owner/root-private snapshots, never opens SQLite
or grants OS access through a BBS role. Do not setuid, force-command it for members,
replace getty, autologin root or edit personal login profiles. Fixed host diagnostics
are read-only, bounded and separate from the owner. Keep ASCII/plain/delta ANSI,
60-sample history, minimum refresh and actual geometry/stale/unknown labels.

Nineteenth snapshot sources/pre95-operator with nightwire-baseline-manifest.json
freezes all 176 distinct original artwork/terminal inputs and both whole records.
Keep nineteen snapshots/thirty-two historical identities unchanged. No new
250-user/physical/privileged-confinement claim. Hardening and host applies stay
later. Use only the owned disposable NetBSD clone; saved playground stays off.

The operator-netbsd result pins 152 actual selected native source matches and
eight harness sources. Final Rust suites: 179 per OS, host Python 115; six themed
operator UI controls per OS, ten actual shared operator controls and seven actual
SSH/native art regressions. Resource samples use the default palette and one
member: dashboard 2,396 KiB RSS / one LWP, owner 5,144 KiB / four LWPs. Keep the
transient-host-helper/whole-VM sampling caveats; never call this new 250 capacity.
Preserve these actual source/harness bytes and whole result before later edits.


## Host-stat/console completion batch

Read plan 0053 and ADR 0048. Twentieth snapshot `sources/pre95-hoststats` and
`operator-baseline-manifest.json` freeze all 162 original operator/build inputs
and the entire initial result before edits. Preserve all twenty snapshots and
thirty-three historical result identities; never refresh old measurements.

`faction-unix` now supplies fixed read-only counters: 64 scalar slots, one 64-KiB
network query buffer, at most sixteen non-loopback interfaces. No per-refresh
command/helper. RAM used includes cache/reservations; interface traffic can be
counted more than once through tunnels/bridges. CPU/network need two valid
samples; resets/scope changes/resume warm up, failures stay unavailable. Four
histories cap at sixty entries and show their newest fitting tail. Explicit
`--unicode` permits trusted width-one block graphs only in valid ANSI geometry;
plain/pipes remain ASCII. Restore the shell cursor on a reserved bottom row.

Serial/getty validation must authenticate a normal OS account on the owned clone,
launch manually, restore termios and log out. A disposable synthetic owner password
is not a BBS role or an installed profile/autologin change. Keep all credentials,
serial logs and admin argv private; public evidence contains checks/numbers only.
Peer/backup adapters are not implemented metrics. Hardening/host actions remain
later; no new 250-user or physical console/capacity claim.

The `operator-host-netbsd` record pins 155 actual selected native source matches
and nine host harnesses. 183 Rust tests per OS, 115 host Python, eight themed UI
controls per OS, ten shared controls, four actual serial/getty controls and seven
shared artwork regressions pass. Dashboard 2,440 KiB RSS / one LWP; owner 5,144 KiB /
four LWPs; minimum sampled whole-VM free 332.66 MiB. This one-member observation
has no per-refresh helper; keep all source/result hashes fixed before later edits.


## Recovery visibility batch

Read plan 0054, ADR 0049 and docs/backup-checks.md. Twenty-first snapshot
`sources/pre95-recovery` and `operator-host-baseline-manifest.json` freeze all 162
distinct prior source/harness inputs and the whole host-stat result. Keep twenty-one
snapshots and thirty-four historical identities unchanged; preserve new measured
bytes before future edits. Never refresh old hashes to fit new runtime sources.

`file_area_backup.py verify` checks a complete selected bundle without creating
restore destinations or reading/changing live DB/locks. Existing backup still
requires stopped/drained PID/transfer locks; restore still needs fresh paths and
checks the whole bundle first. Existing authorized root helper runs Rust catalog
validation as owner; no privilege or host operation enters the dashboard.
`--record-status --board PATH` is opt-in for backup/verify, refused for restore.
Only a versioned <=512-byte owner-private numeric receipt is published. Last attempt
and last verified data stay distinct; a failed recheck retains the earlier success
without making it current. No filenames, hashes, contents, credentials or keys.

D:Data is the fourth read-only panel; B/H/D/I and arrows/Tab retain compact/plain
fallback. **Board:** age refers to the service snapshot; backup ages are historical
and independent. A receipt does not prove artifact identity against live state,
continued availability, off-host copies, included themes/OS/device keys or restore
success. Peer telemetry requires actual future adapters. Host applies, hardening,
physical SBC and further 250-member experiments remain outside this batch.


`operator-recovery-netbsd` pins 159 selected native source matches and fourteen
relevant host harness/test inputs. Rust 184 per OS/Python 118, eleven bundle/check/
restore controls per OS, eight themed UI per OS, ten shared, four serial and seven
shared art checks pass. Native proof uses actual root CLI with distinct non-root
owner/file accounts; single-UID developer APIs do not prove that role boundary.
Dashboard sampled 2,436 KiB RSS/one LWP, owner 5,156 KiB/four LWPs, minimum VM free
333.37 MiB. Preserve measured bytes/whole result before later edits; keep all 35
saved-evidence guards. No new capacity, public endpoint or host/hardening claim.


## Other-BBS directory batch

Read plan 0055, ADR 0050 and docs/other-boards.md. Snapshot 22
`sources/pre27-directory` / `operator-recovery-baseline-manifest.json` preserve
all 173 prior runtime/harness/build inputs and whole recovery result. Keep all
22 snapshots/35 historical identities unchanged; no old hash refresh.

`faction-terminal::directory` holds only bounded inert local presentation metadata,
not authority or a connector. A <=32-KiB/32-listing host pack loads once per service;
shared owner retains one Arc catalog and each client a pointer/cursor. `directory`
service config and local/worker `--directory` are opt-in. Current approval/epoch
must be checked before every frame/action; pending and revoked members see no cards.
No automatic DNS/network/SSH/browser/profile/key/credential handoff. A fingerprint
is a local editorial hint, never destination trust. Remote approval stays separate.

O:Other BBSes is inserted only when enabled; custom contacts actions can choose a
free key. Full 5x2 grids with eleven optional services use the ordered fallback,
keeping 40x18 support and every action. Source templates/artwork remain unchanged
by default. Cards paginate; a non-fitting SSH command is omitted rather than
truncated. DATE?/OLD refer to editorial review windows, not online health. A
stopped restart applies removals; keep packs outside DB/bundle exports.

The unmeasured standalone shared-UI compatibility shim treats Contacts like the
existing unavailable Terminal action; never claim the old experiment implements
this feature or rewrite its frozen executed-source snapshots. No new registry
dependency, owner thread/helper, listener, credential/session limit or capacity
claim. In-BBS submissions/native handoff and protocol adapters remain follow-on;
hardening stays later. Use only owned software clones; saved playground stays off.


`directory-netbsd` pins 165 selected native source matches and seventeen host
harness/compile-compatibility inputs. 188 Rust tests per OS / 118 Python, eleven
directory PTY controls per OS and twelve real SSH/native controls pass, plus
operator/backup/serial/art regressions. Maximum 32-listing catalog/one member:
5,312 KiB owner RSS / four LWPs, minimum sampled VM free 352.36 MiB. These are not
incremental-memory or new capacity claims. Preserve this source/result set before
later edits; keep all 36 saved-evidence guards. Disposable clone/cache removed,
saved playground off. Next product increments: member submissions and explicit
native handoff; keep independent destination approval and member-held keys.


## Native board launcher batch

Read plan 0056, ADR 0051 and docs/board-launcher.md. Snapshot 23
`sources/pre27-launcher` / `directory-baseline-manifest.json` freeze all 174 prior
source/harness inputs and whole directory result. Keep 23 snapshots/36 historical
identities immutable. New client measurements require their own input/result pins.

`--boards` is an opt-in member-owned <=16-KiB/sixteen-profile-path catalog. Local
keyboard alone chooses and confirms; no remote text/frame/graphics can import or
select an endpoint, host key or device key. Reuse profile/SSH validation on each
selection. Keep independent destination registration/approval and one active SSH
child; fresh filter/Files nonce/sequence/recent links per visit. Close/reap before
connecting elsewhere. Ctrl-] B asks before dropping drafts; doubled Ctrl-] forwards
one, single times out after one second. `--profile` stays byte-transparent. Files
foreground flow and optional local Kitty gate/cleanup remain separate.

No owner/schema/listener/helper/default/dependency changes. Use two owned marked
VMs for independent-board proof: production socket remains fixed. Keep private
profiles/argv/keys outside Git, saved playground off and hardening later. Member
submission/moderation UI and directory-to-profile import remain future increments.


`board-launcher-netbsd` pins 168 matched selected native inputs and eight host
harness inputs; 110 checked server/workspace bytes stay unchanged. 191 Rust tests
per OS, 118 Python, seven launcher PTY checks per OS and ten real two-board controls
pass, plus directory/graphics/operator/art regressions. Native process inspection
must use NetBSD `ps -U ... -o pid,ppid,comm` with a skipped header, not Linux-style
combined empty headers. The actual rapid cancel/retry oracle guards the corrected
flush-before-acknowledgement order. One client SSH child per OS / four LWPs per
board; no new RSS/capacity claim. Preserve these sources and whole result before
later edits and keep all 37 saved-evidence guards. Both clones/cache removed,
saved playground off; member submissions and deliberate profile import remain later.


## Named room browser batch

Read plan 0057, ADR 0052 and docs/room-browser.md. Snapshot 24
`sources/pre21-room-browser` / `board-launcher-baseline-manifest.json` preserve
178 previous measured/room-list fixture inputs and whole launcher record. Keep
24 snapshots/37 old result identities fixed, including every affected old R-list
fixture expectation; new UI tests require their own actual input/result pins.

ANSI R now uses shared inert four-room pages, stable selected IDs, cards and U
next-unread. Main N/H/P numeric commands and plain R stay compatible. Fetch fresh
current rows/approval for every action/frame, keep no row/post cache per connection.
Card N/H/P reuses the existing reader/composer, skip only Room ID, clear stale
compose controls and label destination. Finished actions pause/return to main;
Q/resize returns directly. Selection never marks read; keep complete-message and
shared drained-output acknowledgement, current file visibility/retry/Unicode gates.

Do not widen the first-100 listing, schema, dependency, helper/session/credential
or output/drain limits. Browser frames <=8 KiB, existing shared queue <=32 KiB.
No new memory/capacity/federation or hardening adoption follows from this UX.
Use owned software clones only, keep saved playground off and hardening later.


`room-browser-netbsd` pins 169 actual selected native inputs and nine host harness
inputs; 32 core/build/frontend inputs remain unchanged. 194 Rust tests per OS /
118 Python, ten browser PTYs per OS, eleven actual 100-room SSH/native controls,
22 shared-reader controls and menu/directory/art regressions pass. One-member
100-room observation: owner 5,248 KiB RSS / four LWPs; minimum VM free 363.62 MiB,
not a new memory delta/capacity claim. Preserve new input bytes/whole result before
future edits; keep all 38 saved-evidence guards. Clean copied native workspace
artifacts before final validation: copied source hashes do not prove an old
incremental executable rebuilt. Final installed binaries matched the clean build
and destination label. Owned clone/cache removed; saved playground off. Hardening,
search/favorites/persistent current-room state and moderation remain later.


## Explicit room writing desk

Read plan 0058, ADR 0053 and docs/writing-desk.md. Snapshot 25
`sources/pre141-writing-desk` / `room-browser-baseline-manifest.json` freezes all
174 original #147 inputs and its whole record before edits. Keep 25 snapshots
and 38 historical identities fixed; never refresh old hashes.

`/edit` at the unchanged one-line body prompt opts into local room paragraphs.
Keep 1024 body bytes/32 lines, inert UTF-8/classic display, exact explicit commands,
current approval/room/parent checks and all-or-nothing freshly filtered quoting.
Preview and quote cannot submit or mark read. Only `.send` creates a submission
key and reuses the existing same-key busy retry. No new schema, listener, package,
helper/default/credential limits or private-mail authority. Drafts are in memory
and lost on disconnect; no hidden storage. Full-screen caret, naming/theming and
recoverable drafts remain #141. Compare native versus opt-in server room drafts
through the documented ownership/quota/retention/export/recovery gate first.
Every private-mail draft stays device-side. Use an owned clean-built software VM;
saved playground stays off and hardening remains later.

The `writing-desk-netbsd` record pins 174 matched selected native inputs and eleven
host harnesses, with 32 core/build/frontend inputs unchanged. Validation: 199 Rust
per OS / 118 Python; thirteen writing PTYs per OS; eighteen actual shared SSH/
native/plain controls; ten browser PTYs per OS, 22 shared-reader regressions and
existing menu/directory/operator/art checks. Explicit preview/quote leaves zero
unread rows; busy retry creates exactly one post. Native clean build and installed
hash/literal checks pass. Keep all 39 saved-evidence guards; preserve these new
actual inputs and whole record before later edits. Owned clone/cache removed,
saved playground off. No new capacity or persistent/full-screen draft claim.


## Full-screen editor and schema-8 room checkpoints

Read plan 0059, ADR 0054 and docs/writing-desk.md. Snapshot 26
`sources/pre141-full-screen` / `writing-desk-baseline-manifest.json` freezes 180
actual #148 inputs and its whole result before edits. Keep 26 snapshots/39 old
record identities fixed; never refresh historical hashes to current code.

Use the same 1024-byte/32-line room model for line/full editing. Only explicit
Save persists, with visible plaintext/sysop-readable scope. Four rooms/member,
one checkpoint/room, 30-day recovery/lazy per-member expiry, revision CAS on save/
delete, epoch/status/reply checks. Matching send cleanup is in the posting
transaction and uses the saved request ID for retry; never clear another
connection's newer checkpoint. Pending/revoked members cannot recover/send;
status changes and local owner recovery clear affected checkpoints. All private
mail drafts/keys/decryption remain member-side. Owner exports/bundles include
local room drafts; no secure-erasure/autosave/remote sync/member-export claim.
Schema 8 requires stopped/drained complete backup and fresh-path rollback.

Full-screen arrows/cell widths, paragraph keys, preview/quote, explicit SEND
confirmation and Line fallback use inert text and trusted theme controls. Optional
`writing_title` is <=24 printable ASCII bytes. Preserve decoder/scalar state across
fitting resizes: resetting a partial arrow caused `[D` to enter draft text in an
initial PTY run. Too-small/malformed input uses an Enter recovery barrier before
classic text so delayed fragments cannot become edits. Shared redraw is once per
bounded read, frames <=8 KiB, existing output queue/drain/defaults unchanged. No
new helper/listener/package or capacity/hardening claim; saved playground stays off.

`full-screen-netbsd` pins 184 matched selected native inputs and 17 host harness
inputs, with 25 dependency/access/credential inputs unchanged. Validation: 213
Rust per OS / 118 Python; seven full-screen/checkpoint PTYs per OS, eleven real
SSH/native/plain controls, thirteen legacy unsaved writer controls per OS,
22 shared-reader regressions and ordinary browser/menu/operator/art checks.
Complete-bundle eleven per OS includes actual distinct non-root owner/file CLI
proof on native NetBSD; export/restore and schema-7/8 compatibility pass. Keep
all 40 saved-evidence guards and preserve these new inputs/whole result before
later edits. Owned clone/cache removed; saved playground off. No new capacity,
autosave, private-mail plaintext, secure-erasure or hardening claim.


## Local community wall

Read docs/community-wall.md, plan 0060 and ADR 0055. Schema 9 adds optional local
wall policy, <=128 notes/edition, <=240 ASCII bytes/four <=72-column lines, at most
twelve explicitly retained previous weekly editions and one cooldown/retry row
per member. No room/chat/mail/notification/federation projection. Default disabled.
Sysop S/W enables/names/configures; changing weekly mode requires YES and clears
notes/archives. Retention and UTC-Monday rotation are lazy, without a new helper.

Current board/member/epoch/approval checks precede each frame/action. Own edit uses
expected-body conflict detection; delete uses the shown stable ID and current
owner/sysop authority. Archive edits/posting are denied, deletion allowed. Only
explicit Send makes a key, retained on Busy; no wall checkpoint/autosave. Rendering
is inert ASCII with trusted theme controls, classic commands followed by Enter.
Enter raw input before showing its prompt: initial PTY evidence caught a fast
first-keystroke loss from flushing after display. Shared resize keeps completed
text, clears unfinished input and recomputes classic capability.

The shared workflow lives in faction-wall, with core/terminal dependencies only,
no I/O/host/network authority or new registry package. Shared State allocates it
only while open. Enabled main menus use a temporary theme copy; disabled paths
retain the original. More than eleven entries scroll a nine-row ordered lightbar
window; direct hotkeys/plain all-entry menus remain. Keep frames <=8 KiB and
existing queue/drain/session/credential/helper budgets; no new capacity claim.

Owner snapshots/bundles include wall state; old schema-2..8 exports stay read-only
valid. Stopped/drained complete pre-upgrade backup and fresh-path rollback required.
Logical deletion is not secure erasure. Snapshot 27/full-screen-baseline-manifest
freezes 192 prior actual inputs and the whole #149 result; keep all forty historical
identities unchanged. Validate only owned software clones; saved playground off.


`community-wall-netbsd` pins 193 matched native inputs and nineteen selected host
harness inputs. Rust 1.96: 224 tests per OS; Python 118. Seven wall PTYs per OS,
eleven actual SSH/native/plain, twelve complete wall/file recovery controls per
OS (native actual distinct-UID CLI), room/editor/reader/menu/art regressions pass.
An actual run caught the last-archive-note redraw error: archived deletion now
returns to current with its success message. Keep all 41 saved-evidence guards;
freeze new actual inputs/whole result before later edits. Owned clone/cache
removed, saved playground off. No new capacity, hardening or federation claim.


## Member-owned local pages and .plan updates

Read docs/member-pages.md, plan 0061 and ADR 0056. Schema 10 adds optional page
policy, four ASCII fields (240/120/240/120 bytes, 4/2/4/2 lines) and immutable dated
updates (480 bytes/eight lines, at most sixteen/member, 7–365 day retention). All
lines <=72 columns. No home files/Finger service, home proof, mail, notifications
or federation. Hidden fields/updates remain plaintext/sysop-readable local data.

Default disabled. Sysop S/M enables/names, selects card/compact layout, field mask,
history retention and hide-only versus expressly enabled permanent removal. Core
rechecks current board/member/epoch/approval/roles. Foreign list/view/history omit
unpublished, moderated and revoked-author pages and hidden bodies/counts. Own
revision-checked Save/delete protects other editors and sysop hide. Moderation
never rewrites another author's text; author edits cannot clear the gate.

Use faction-pages' shared bounded workflow for local/worker/shared routes. Only
explicit Save persists. Preview bodies paginate; unsaved text has no checkpoint.
Deleted update actions address the shown stable ID, never a new row at an offset.
No room unread markers, new listener/helper/registry dependency or changed default/
credential/output/drain limits. New M action uses temporary theme copies only when
enabled. Shared resize keeps completed text and clears unfinished input.

Owner snapshots/bundles include policy/pages/retained plans, including hidden text.
Old schema-2..9 exports remain read-only valid; stopped complete pre-upgrade backup
and fresh-path rollback required. Snapshot 28/community-wall-baseline-manifest
preserves 202 prior measured/harness/compatibility inputs and whole wall result.
Forty-one historical identities stay unchanged. #150's old custom-menu exact
prompt assertion was corrected to the stable Create-room suffix; its old measured
fixture already resolves from frozen snapshots. Merge a12eb87 passed corrected CI.
Use owned software clones only, keep saved playground off and hardening later.


`member-pages-netbsd` pins 203 matched native inputs and 22 selected host harness
inputs. Rust 1.96: 235 tests per OS; Python 118. Eight page PTYs per OS and fourteen
actual SSH/native/plain controls prove different-member visibility, active hiding/
revocation, author/moderation conflicts, Busy retry and explicit deletion. Complete
page/hidden-plan/wall/file recovery thirteen per OS includes native actual distinct-
UID CLI. Existing wall/editor/reader/menu/art regressions pass. Keep all 42 saved-
evidence guards, preserve these new inputs/whole result before later edits, remove
only owned software VM/cache and keep playground off. No new capacity/hardening
or federation claim.

Own page removal must not clear moderation: delete text/updates but retain a minimal
empty moderated row with fresh revision, no shared fields and unpublished state.
A new author save/recreate cannot bypass that gate; only sysop show clears it.
Core and actual SSH negative controls cover deletion/recreation. For successive
owned VM test suites, restore the normal ANSI config explicitly: the art/plain
controls can leave a different fixture presentation. Do not mistake it for a new
application regression or change production startup policy to satisfy the harness.


## Requested persistent Ghostty playground

Read docs/test-playground.md, plan 0062 and ADR 0057. The owner now explicitly
requests a running private test login. This supersedes earlier playground-off
instructions for the new distinct preview only. Preserve the old saved image,
keys and helpers; keep preview images/credentials/profiles/logs in ignored private
storage. Do not stop the handed-over preview as disposable-test cleanup. Record
its private pointer in local/demo/CURRENT_PREVIEW. tools/playground.py controls
only a prepared owned VM, validates PID birth/command and uses loopback/pinned SSH.
The new-only seed is an example outside member entry points; never reseed an
existing board. No runtime/schema/dependency or historical evidence changes.
No new 250-user, production, real-Ghostty-pixel or physical-hardware claim.


Latest owner instruction supersedes the prior running-preview handover: the private
playground is stopped, with its disk/accounts/keys/return helpers retained. Keep it
off. Use only a separate marked owned VM for validation. Plan 0063 reconciles #94
and the current client. The optional private board catalog `mail_store` enables M
only in the local picker with no active Remote/SSH child. It opens an existing
member-side offline vault; no auto-init, BBS password reuse, remote path/trust
selection, server decryption or network-mail/roaming claim. Keep #90/#91/#88 open.
Thirty snapshots preserve both input sets and forty-three original result identities.

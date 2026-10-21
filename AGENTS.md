# Faction BBS agent guide

## Read first

**Current priority (owner, 2026-10-21): ordinary running product first.** Follow
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

## Product constraints

- Preserve the WWIV-inspired approved-login -> BBS-specific ANSI art -> sysop-defined welcome/menu session. No community access before authentication and local sysop approval.
- Use Citadel-style message rooms, IRC-style command-driven chat, BBS menus, local file areas/mail, and optional isolated game doors. Existing games are reuse candidates, not implemented support claims.
- Keep each sysop's identity, accounts/approval, content, files, games, directory, and local-only chat local. Only explicitly participating shared chat channels may federate in the future.
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
  issue #71. Improve setup and member/sysop workflows, then Lua menus/isolated
  doors. Keep existing approval, credential and access protections. Experimental
  isolation/admission/host-tuning adoption is not a first-preview prerequisite;
  preserve incomplete #70 findings without claiming its full matrix passed.

- Physical SBC test hardware is unavailable. Build/research software on developer hosts and NetBSD VMs now; the owner will load a complete product onto hardware later. Physical tests gate hardware-specific claims, never near-term software work. Keep resource measurement and recovery/security tests.
- Build the first useful local core first (`dev-docs/first-local-slice.md`, #35). Lua menus and isolated game doors are the first extension priority after it (#10); a boundary probe is evidence, not a production sandbox.
- Optional permanent/public rooms, ANSI stores/payment adapters, blockchain games/wagers and modern networks are active future research. Compare both external-member-wallet and separate sysop-service models before choosing; no provider/custody/currency adopted. Follow `docs/extensions.md` and ADR 0007.
- Automatic BBS federation stays chat-only. A public-publishing module is a separate explicit outbound category under research, never authority to export private local content. Keep external data, publication, payment requests and spending privileges distinct; no live/funded external actions are implied by prototype authorization.

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

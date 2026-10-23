# Changelog

This file records meaningful project changes. Product releases will use versioned entries after a release process exists.

## Unreleased

- Add optional personal catch-up: bounded unread/direct-reply previews, own room
  favorites, eligible published files, gatherings and sysop notices with shared
  ANSI/classic workflows and current-authority source jumps. Visits are explicitly
  saved; previews never mark posts read. Schema 13 includes preferences/checkpoints
  in complete recovery; stop/drain and verify a backup before upgrading. Fix exact
  snapshot schema checking when the catalog grows beyond the previous row cap.

- Add optional local single-choice polls with explicit preview/ballot confirmation,
  vote change/withdrawal, result policies and text-cell charts, creator close/cancel
  and sysop moderation. Preserve plain/classic terminals and resource defaults.
  First vote permanently locks wording; ballots are ordinary owner-readable data.
  Schema 12 extends complete recovery; stop/drain and back up before upgrade.


- Add optional local gatherings/chat nights: one-off UTC events, explicit preview/save,
  organizer edit/cancel, counts-only RSVPs, sysop policy/moderation, approved-login
  reminders and deliberate local room/chat jumps. Shared local/worker/SSH workflow
  keeps classic fallback. Schema 11 includes events/responses in owner recovery;
  stop/drain and verify a complete pre-upgrade backup. No external calendar.

- Integrate the previously separate native local encrypted mail vault with the current
  client, preserving board selection, terminal graphics and Files behavior. Preserve
  both source/evidence baselines before reconciliation. Network mail/roaming stays open.
- Add optional member-owned `mail_store` to the local board picker; M opens the
  existing encrypted offline vault only between SSH visits and returns locked.
  Classic 40x18 selection, standalone vault, graphics and Files paths remain.

- Add a populated private Ghostty/SSH playground workflow, guarded prepared-VM
  lifecycle controls and new-only sample community seed. Preserve the older saved
  image and all current runtime/schema/defaults; no public listener or new capacity
  claim.

- Add optional local member pages and dated .plan history: explicit preview/save,
  field/page visibility, ASCII signatures/projects, own deletion and sysop
  moderation/presentation/retention settings, with classic terminals and schema-10
  owner export/complete-bundle recovery. Stop/drain and back up before upgrade.


- Add an opt-in local community wall: short ASCII notes/art, explicit review/send,
  chronological paging, own edits/delete and sysop removal, configurable name,
  cooldown/quotas/retention and optional weekly archives, shared across classic,
  ANSI, worker and shared-service terminals. Schema 9 extends owner snapshots;
  stop/drain and make a verified complete backup before upgrade.


- Full-screen caret/paragraph room editing, themed desk names, explicit confirmed
  send, and classic fallback with resize/input recovery. Schema 8 adds explicit
  sysop-readable room checkpoints, resume/delete, four-room quotas, 30-day
  recovery and atomic send cleanup; old exports remain readable. Stop/drain and
  verify a complete backup before upgrade. Private-mail drafts remain device-side.


- Optional `/edit` room writing desk: multiline paragraphs, undo/clear, fresh reply
  quoting, paged inert preview and explicit send/cancel in local/worker/shared
  sessions. Existing one-line posts and retry/unread semantics remain. Drafts
  are connection-local; full-screen editing and reconnect recovery stay #141.


- Add a shared themed ANSI room browser: four-entry pages, arrow/hotkey cards,
  next-unread selection and New/History/Post into the selected room. Composer
  clears stale controls and labels the destination. Plain/numeric paths remain.
- Reuse chronological reader, subject/retry and complete-message unread/output-
  drain rules; current approval/room/file visibility checks remain authoritative.
- Preserve 178 prior launcher/fixture inputs and its whole result before changes.
  No schema, dependency, extra owner helper or normal-limit change; hardening later.

- Add an optional native board launcher with private member-owned profile catalogs,
  arrow/hotkey pages, explicit connection/disconnect confirmation and return after
  logout/failure. Retain single-profile behavior and classic-door escape passthrough.
- Revalidate each selected profile, keep strict SSH host keys and fresh per-visit
  filters/Files state; no server-supplied endpoint or credential/agent forwarding.
- Preserve 174 prior directory inputs and its whole result before client edits.
  Server/schema/defaults remain unchanged; native identity/mail and hardening stay later.

- Add opt-in sysop-curated Other BBSes packs and approved-member connection cards
  in local, inherited-worker and shared sessions. Arrows/hotkeys, paging, compact/
  plain and main-menu return share one bounded presenter. No automatic connection,
  credential/host-key import or remote membership/federation.
- Add offline directory validation, startup config and deliberate review/expiry/
  removal documentation. A full custom grid falls back to an ordered eleven-action
  menu when Files plus Contacts need more cells. Defaults/artwork stay unchanged.
- Preserve all 173 prior source/harness/build inputs and the entire recovery result
  before edits. The old standalone UI only gains a disabled-action compatibility
  arm; its frozen measured sources and historical claims remain unchanged.

- Add complete-bundle verify-only checks and optional private backup/check receipts.
  Restore shares the same catalog/blob verification before creating fresh paths.
- Add D:Data/Recovery operator panel with latest-attempt/last-success ages/totals,
  explicit failure/clock/unavailable labels and the existing themed classic fallback.
  No live artifact or successful-restore promise, schema, host action or hardening change.
- Preserve 162 actual host-stat execution/harness inputs and its complete result
  before this batch; historical measurements remain unchanged.

- Complete the CPU/RAM/network host-stat slice with a fixed native read-only
  bridge, interval/reset/warming semantics, bounded recent graphs and optional
  Unicode blocks. Remove the refresh subprocess; preserve plain/classic fallback.
- Polish compact history and shell return; validate real disposable NetBSD
  OS-console login/launch/exit without a login-profile or getty change.
- Preserve 162 original operator/build inputs and the whole initial result before
  runtime edits. Host applies, peer/backup adapters and hardening remain later.

- Add an explicitly enabled owner-only aggregate status feed and separate local
  `faction-sysop` console: themed ANSI/plain panels, read-only host summaries,
  pause/navigation/resize and stale/unavailable/stopped/clock labels. No new
  listener/helper/schema or member privilege; publication defaults disabled.
- Preserve both complete Nightwire records and 176 executed inputs before runtime
  changes; add aggregate privacy/SQL-budget/export and native operator validation.

- Add isolated real-terminal Nightwire probes for page/menu cursor placement,
  CP437 cell widths and actual compact windows, with explicit fallback and
  multiplexer viewport guidance. No runtime, theme asset or default change.

- Add original Nightwire CP437 artwork with bespoke slanted lettering, radio-city/
  receiver scenes, matching short cover and autumn/winter/compact ASCII variants.
- Add reproducible offline authoring source, product-based geometry/contrast/menu
  checks and preview/login validation. Packs are opt-in; runtime/default installer,
  earlier themes and measured inputs stay unchanged. Final owner art review open.

- Add opt-in long ANSI login identities with bounded 16-row paging, Enter/skip,
  approval checks and compact/plain/resize recovery in local/worker/shared flows.
- Add original three-page CP437 sample and deterministic page previews. Long
  imports cap at 80x256 / 64 KiB, each output page <=16 KiB; short imports unchanged.
- Preserve executed #124 inputs/results before changes; no new dependency/schema.

- Add opt-in sysop grid menus with explicit positions, spatial lightbars, ordered
  Tab/hotkeys and shared focus restoration; compact/plain ordered recovery stays.
- Add bounded SAUCE00 Character/ANSi and non-blink iCE background import, an
  original CP437 grid/login template and inert Unicode-cell authoring previews.
  Existing 80x16/16 KiB limits and provenance requirements remain.
- Preserve all executed #123 seasonal sources and its original result before edits.

- Add opt-in annual #25 theme calendars with UTC/system-local dates, bounded
  priorities/year-wrapping windows, manual/disabled recovery and coherent
  next-login selection across local/worker/shared sessions.
- Add original autumn/winter ANSI and compact ASCII login templates, a shared
  bounded identity presenter and deterministic date/login authoring previews.
  Calendar defaults disabled; classic/plain recovery and approval stay intact.
- Extend the existing native bridge for OS calendar conversion; no external
  dependency/helper, clock/host policy or schema change. Preserve original #122
  inputs/results before edits; final scene-art acceptance and hardening remain later.

- Polish #25 main menus in both terminal adapters: centered compact/wide panels,
  continuous lightbar focus, explicit unavailable markers, safe sysop titles and
  optional wide action descriptions. Keep arrow/hotkey authority and classic/plain fallback.
- Coordinate base/manual seasonal palettes and add local theme authoring plus
  original reference-palette previews; no archived art, runtime dependency or graphics-filter change.
- Freeze all 124 actual #121 source inputs and its unchanged compatibility record
  before changing presentation. Hardening stays later; no fresh capacity claim.

- Validate twelve real terminal/font/tmux/screen profiles (#118), 48 local/SSH/
  native BBS sessions, image retention/deletion and live SSH resize recovery.
  Qualify old GNU screen as classic-only; retain default filtering and fallback.
- Add reproducible isolated native reference/measurement tools and exact evidence;
  no runtime dependency, protocol/filter, schema, capacity or hardening change.

- Add optional T/G inline Kitty graphics (#117): positive isolated probes, one
  original 160x80 RGB calibration image, <=4-KiB chunks after output drain,
  rate/deadline limits and partial-stream cleanup with classic ANSI fallback.
- Add explicit native `--kitty` local probe and exact trusted-command regeneration
  gate; preserve default APC refusal, schema/helper/admission limits and all old
  measured evidence. No arbitrary art/image-file/PNG/Sixel support or GUI claim.

- Add optional session Terminal settings (#96): native UTF-8 room/subject text,
  cell-aware paging/clipping/backspace, 16/256/RGB choices, block/braille samples
  and bounded synchronized redraw with an isolated explicit sync probe. Keep
  classic defaults/recovery and native-client graphics/control filtering.
- Review/pin unicode-width 0.2.2 without default features; preserve schema 7,
  prior evidence and existing credential/helper/admission/output limits. Record
  modern terminal research/budgets and software fixtures; bitmap/extended input
  and later hardening remain separately scoped.

- Add schema-7 UTF-8 room subjects (#114) with one shared local/worker/service
  input editor, split-character/backspace validation, UTF-8-safe reply clipping,
  bounded inert ASCII echo/previews and subject-aware retry preservation.
  Validate old schema-6 ASCII before migration; retain public IDs, provenance and
  read-only old exports. Document stopped backup/upgrade/restore/rollback.
  Native glyph rendering stays separate; no new dependency, helper or hardening.

- Broaden #103's text codec to profile v2: UTF-8 encoded wire subjects, common
  numeric-zone dates, bounded plain-text MIME 7bit/8bit/QP/base64 input and exact
  QP output. Preserve long lines, trailing whitespace and final-newline semantics;
  keep DB schema 6/composer ASCII and track Unicode composing separately in #114.
- Prune bounded local reply ancestry to root plus fifteen recent References with
  a fixed 128-ancestor work cap. Add original INN 2.7.4 offline roundtrip/rejection
  validation on macOS/NetBSD and CI; no news runtime, publication or hardening
  activation. Preserve all earlier measured source/result identities.

- Add authored room subjects, previewed inherited reply defaults and subject-aware
  retry checks in local/worker and shared terminal sessions (#103). Keep schema 6.
- Connect approved stored posts to bounded local proto-articles with safe aliases,
  UTC, group/Path mapping, ordered references and strict duplicate/conflict
  comparison. Preserve canonical bodies, unknown legacy dates and current file
  permissions; no publication/import/NNTP runtime. Add independent MIME parsing
  and separate synthetic codec allocation measurements.

- Add schema-6 persistent post metadata (#103): stable local article IDs, generated
  subject labels and actual new-post UTC timestamps; preserve unknown legacy dates,
  body/reply/unread data and old exports. Show bounded metadata headers in both
  room readers. Document stopped-service backup/upgrade/rollback; no news feed,
  new authority or authored-subject composer is enabled.

- Capture later-phase hardening stories #106–#111: release/member signing, minimal
  VM, post-compromise containment, storage/backups and independent validation.
  No host/firewall/key changes or adopted hardened profile.

- Add a dependency-free Rust Netnews text-article codec and offline validator
  (#103 foundation), plus shared wrapped/paged room screens for local/worker and
  shared-service sessions (#21/#25). Preserve contiguous file-reference tokens,
  current approval checks and drain-before-complete-message unread acknowledgment;
  use sysop theme colors for ANSI section changes. No schema migration, listener,
  provider connection or automatic publication. Preserve the original native-client
  sources in the exact pre90 snapshot before shared UI/workspace changes.

- Capture owner stories #95–#101: local VM-console sysop dashboard, modern terminal
  capability/graphics research, NAT-aware firewall controls, NetBSD host assistance,
  integrated native XMPP unread/private UX, real IRC comparison and opt-in shared
  discussion-room protocol research. Update current product/agent trust boundaries,
  UI-polish expectations and primary-source inputs. No runtime/protocol adoption,
  live host/firewall change or automatic room publication is implemented.

- Record the owner's Netnews article direction for discussion-room posts only
  (#103), NNTP-first optional peering (#101) and read-only-first news-provider
  subscriptions (#104). Keep IRC/XMPP formats, chronological rooms, local storage
  and explicit publication boundaries; no codec, migration or adapter is enabled.

- Add a separate two-home encrypted-mail and scoped roaming research proof (#84):
  12 real OMEMO-2/SCE mail controls against independently owned Prosody 13.0.7
  processes on NetBSD, plus 11 native SSH certificate-verifier controls. Retain
  four native signing crashes, inspected reference dependency/profile findings,
  reproducible software fixtures and cleanup. No native mail/keystore/SSO product
  integration or crypto-library adoption; next build gates #90/#91/#92 and encrypted
  attachments #88 remain explicit. Maintained runtime/defaults/history unchanged.

- Add optional native Rust terminal client (#87, continuing #84): base OpenSSH,
  local PTY/terminal filtering, approved BBS login and integrated Files menus,
  upload/download/progress/cancel without leaving the session, private no-overwrite
  checksummed local saves, and recently seen room/chat links. Add schema-5 stable
  public references with send/read/open permissions, private metadata redaction,
  namespace-preserving restore and scoped transfer abandonment. Preserve normal
  SSH/manual SFTP and owner/credential/eight-session limits. Real macOS and native
  NetBSD software tests; no hardware/capacity claim. Keep device-encrypted mail
  attachments under #88 and broader XMPP/E2EE/roaming proof #84 open.

- Implement optional local file areas and separate restricted SFTP (#83): shared
  Files menu, paginated metadata/search, private-area grants, quota-reserved
  quarantine, sysop inspection/publication/removal and SHA256-verified downloads.
  Reuse base OpenSSH signatures/transport with a bounded Rust single-object SFTP
  v3 handler under a separate UID and a distinct key-lookup account. Add disabled
  NetBSD setup, two-stream/32-KiB limits, explicit partial GC and complete private
  catalog/blob backup/fresh restore. Schema 4 preserves older exports and strips
  transfer authority from new ones. Resume and universal client support remain out
  of scope; no owner thread/process/credential limits or eight-session default change.
- Record independent invited home identities and native member-side encrypted
  mail as owner requirements. No BBS server may see plaintext; no server-readable
  fallback, automatic local membership or replicated credential database. Expand
  old chat-only federation scope only for explicit mail/identity capabilities;
  XMPP/E2EE/visiting-auth interoperability remains research #84.
- Specify extension seams and retain future DOS/Amiga/other game-host adapters,
  with terminal-capability and isolation evidence required per title. No new
  native client, mail protocol, emulator or plugin runtime is implemented.

- Prove unchanged original LORD 4.06 through the maintained SSH game doorway on a
  separate native NetBSD ARM64 VM. Add a disposable private game host/oracle,
  exact artifact provenance and a small external DOSBox receive-state patch.
  Keep one active game because DOS record locking is unavailable; no original
  binary/art/data or supported production launcher is bundled.
- Handle LORD's legacy RIP probe safely and allow its bounded 255-column cursor
  return. Preserve the prior connector's measured parser/record bytes. Record
  original TWGS installation with publisher activation deferred by the owner.

- Add optional fixed-destination classic game-server doors: explicit member
  destination confirmation, bounded Telnet/CP437/basic ANSI, local Ctrl-] return,
  safe control filtering and failure/resize/revocation cleanup. The original game
  runs on a separate host; no local legacy launch or new dependency/limit increase.
- Record both accepted original-server and original-DOS reuse paths, primary-source
  title/acquisition inventory and concrete real-title follow-ups. Connector tests
  use owned fixtures; no classic title, licensed binary or compatibility host is
  bundled or claimed supported. Keep Lua installation compatible with TOML tables.


### Added

- Persistent local arcade top tens, personal bests and completed-run counts for
  Signal Hunt and Neon Salvage, an arcade rules guide, score browsing/return and
  play-again navigation. Host-owned one-shot run authority keeps member identity
  and SQLite out of Lua; completed frames carry one bounded score.
- Transactional schema-3 upgrade and private score export/verify/restore, while
  retaining read/restore compatibility for schema-2 exports. Older binaries need
  a pre-upgrade snapshot to downgrade. No new dependencies or service-limit changes.


- Add an optional Lua-driven Doors lobby and two original games, Signal Hunt and
  Neon Salvage. The shared BBS renders validated screens with sysop colors,
  arrows/Enter, hotkeys and plain fallback. State stays with its own connection.
- Run each Lua turn in a bounded, short-lived child under a separate game UID,
  with no BBS state/identity/TTY/credential descriptors or Lua OS/I/O/module APIs.
  One evaluator runs globally; the backend keeps all credential/process/thread
  ceilings and uses nonblocking peer-verified requests. Failed doors return to
  the board. Add opt-in NetBSD setup and runtime/SSH play/failure checks.
- Fix fresh-host setup parent permissions and private build-directory handling
  discovered while installing the optional arcade. Keep native/DOS legacy doors,
  game economies and hostile native hosting as separate work.

- Prioritize the ordinary small-board product over further combined 250-user
  edge-case research. Add a native NetBSD first-install helper with read-only
  preflight, protected owner/terminal accounts, restricted invitation keys,
  editable ANSI theme and eight-session defaults. Installation never starts a
  listener; boot services stay disabled until the sysop enables them.
- Fix ordinary boot availability: detach the existing shared/worker backend via
  a non-setuid helper executing as its service owner; retry the dedicated SSH
  listener's exact interface for up to 30 seconds and verify the daemon pidfile.
  No new resident supervisor, wildcard binding or host tuning. Preserve old rc
  script execution bytes and unchanged historical capacity records.
- Add the complete sysop/member quickstart and a disposable-VM SSH rehearsal for
  bootstrap, pending approval, themed arrow/hotkey menus, rooms/chat, normal
  restart/reboot and private export. Keep experimental isolation/admission and
  host tuning separate. Lead the README with the runnable product path.

- Add an opt-in listener-only global SSH admission experiment: fixed 120-byte
  persistent root budget, 32 volatile source shares, original peer sockets and
  unchanged base penalties/crypto/local approval. No proxy or resident daemon;
  remove the module before session exec, preserve budget across restart/reboot,
  and stop on HUP. Root preflight pins module/base bytes; adoption needs review.
- Add native fork/state/mapping/exec/failure controls and unchanged-penalty private
  peer separation with real approved BBS/chat and forged-source UDP controls.
  Two fresh 250-member matrices pass unchanged guards (132.86 MiB minimum stress,
  141.14 MiB under TCP pressure, zero swap, 34,611 confirmed peer deliveries).
  Preserve failed candidates/setup diagnostics and all 145 original039 inputs in
  a fourth snapshot. Defaults stay eight; no production or 250-VPN-peer claim.


- Bound the experimental SSH helper to four durable lookups per monitor birth;
  exhausted calls skip key reads, state writes/fsyncs and cohort liveness scans.
  Preserve ordinary query/signature, local approval, eight-session defaults and
  all crypto/credential/resource limits. Add explicit quiescent inode-preserving
  v1-to-v2 registry migration; legacy/corrupt/live conversion fails closed.
- Add native eight-adversary syscall accounting, signed/unknown-key/options,
  migration/birth/restart controls and two fresh 250-member whole-host matrices.
  Minimum stress headroom 130.36 MiB, zero swap; native source penalties retain
  temporary same-source denial while peer chat continues. Preserve all measured
  037/038 bytes in a third snapshot; new result 039 checks current sources.
  No global TCP/fork-rate or production-capacity claim.

- Add bounded 15-minute active/abuse workloads at 250 distinct SSH members with
  unchanged service/crypto/credential/resource settings: two fresh boots pass all
  sampled guards, 423,798 confirmed peer deliveries, post-load controls and
  recovery; minimum stress/control headroom 133.13 MiB, zero swap. Preserve
  complete native source/binary/counter evidence, actual retained fixture rows,
  rollback and cleanup. Add saved-evidence corruption controls and CI checks.
- Record 64 unsigned key queries on one reservation and temporary same-source
  admission denial without weakening base protections. #66 bounds repeated lookup
  work; source fairness and CPU/timekeeping interpretation remain adoption gates.
  Defaults remain eight; no production capacity or long-soak claim.

- Experimental base-OpenSSH key admission: dedicated helper UID, fixed private
  process-birth reservations/global budget, no fallback key files, strict native
  preflight and quiescent runtime initialization. Authenticated caps survive
  listener restart; no resident daemon or new SSH parser/crypto. Default eight;
  preauthentication/fork/CPU/source-fairness gates remain separate. Native
  admission/channel/UI/rollback checks and three fresh 250-member VM runs pass
  with 129.34 MiB lowest sampled stress headroom under the unchanged guard.
  The inetd listener's two failed memory controls and executed inputs are kept.
  No supported capacity, physical SBC or production security claim.

- Experimental NetBSD minimal noexec root and explicit stateless group/user IP
  policy, pre-main native privilege drop, isolated root preflight and a separate
  setuid frontend. Existing core/UI/credentials/limits/default eight remain intact.
  A numeric sysctl lookup avoids per-frontend name-resolution allocation. Three
  fresh-boot 250-member workloads pass with 132.71 MiB minimum stress headroom;
  both earlier failed controls and their executed sources are retained. Native
  filesystem/exec/network/privilege controls, 31 sealed frontend, 29 UI and 16
  lifecycle checks pass. Independent privileged-code review and broader isolation,
  workload and host-policy adoption remain open; no supported sandbox/capacity.

- Native shared-service/frontend process envelope: kernel peer UID checks on both
  socket ends, inherited identity/FD/environment hygiene, strict owned stdio TTY
  and completion, irreversible verified resource ceilings. No new registry
  dependency, schema or automatic host tuning. Native security/lifecycle checks
  and three fresh-boot 250-member runs pass with 137.61 MiB lowest stress
  headroom under the unchanged guard; eight sessions remain the default.
  Filesystem/network/exec isolation and wider capacity adoption stay open.
  Preserved #62 executed inputs and all nine historical records before changes.

- Maintained shared Rust terminal backend and NetBSD native terminal frontend,
  strict bounded owner configuration, opt-in rc.d integration and tested worker
  rollback. Default admission remains eight / five seconds; larger/faster settings
  require explicit experimental configuration. Full themes/rooms/chat/sysop UI,
  core authority and credential budgets are retained. Historical executed sources
  and all eight result files are preserved; no new registry dependency or schema
  change. Actual native service-path validation is recorded separately; complete
  capacity, stronger confinement and maintained host policy remain open. Three
  fresh-boot 250-member installed-service runs pass, with 135.23 MiB lowest
  sampled stress headroom under the unchanged guard; 27 UI and 11 service
  configuration/lifecycle checks pass.


### Added — 2026-10-20

- Full bounded shared UI candidate: sysop ANSI/CP437 packs/manual seasons,
  reordered arrows/hotkeys, plain/resize/editor recovery, selected room pages,
  replies/exact post retries and role-checked sysop/local-chat controls. Shared
  themes load once, output grows lazily within an encoded cap and work rotates.
  Twenty-seven native UI/fault checks and three fresh-boot 250-member NetBSD
  workloads pass the unchanged guard; complete source/binary, SSH, memory,
  control, limits and cleanup evidence is saved. Normal services remain eight;
  scoped deployment/confinement and active-workload/soak adoption stay open.

- Repeatable native 250-member headroom experiment: three fresh-boot all-service
  NetBSD VM passes with 180.68–182.69 MiB final headroom and
  137.23 MiB lowest sampled credential/control headroom under the unchanged
  guard. Identified locked NTP memory and tested its supported pageable-memory
  option with exact rollback; rejected allocator-only alternatives. Added live
  process limits, fresh owner credentials, complete calibrated ledgers and actual
  SSH re-entry regression; isolated same-name standalone Cargo outputs after a
  reproduced stale-executable pilot. Normal eight-worker services/schema/crypto
  are unchanged; UI/adoption, clock precision and broader workload gates remain.

- Versioned native allocator/transport comparison: two short functional 250-member
  SSH menu/chat workloads, with only one final headroom-guard pass, on a 2-vCPU/1-GiB NetBSD VM, with full SSH/frontend costs,
  calibrated high-load real credentials, failures/recovery, repeated fresh boots
  and exact guard/batch ledgers. Corrected chat re-entry sequencing in the
  standalone variant; historical evidence preserved. Guest-wide tuning remains
  experimental, normal service stays eight workers, and shared UI/adoption,
  full workload/soak and physical-device claims remain open. No new registry dependency,
  weaker credentials or normal application/schema/default change.

- Isolated shared-board/readiness terminal prototype and matched native SSH
  comparison, retaining base OpenSSH, terminal UID isolation and core approval/
  epoch/private-grant checks. Final guarded 1-GiB NetBSD run admitted 224 menus
  versus 168 workers, with matched 100-member menu/chat passes; earlier shared
  trial reached 232. Sixteen native boundary/backpressure/crash/recovery oracles
  passed. Default service remains eight; full shared UI/adoption and 250 remain
  unvalidated. Added calibrated persistent native credential sampling with full
  compact timing/counter ledgers: known 19/38-MiB controls captured exactly,
  unchanged Argon2, held-slot/bad/good credential bursts and recovery. Observed
  peaks remain lower bounds; private VM/keys/disks/logs/forwards removed. No new
  registry dependency/features, normal schema or production default change.

- Per-board cross-process credential work budget: two unchanged Argon2 operations,
  bounded busy denial, strict stored-cost checks and unsafe-lock regression tests.
  Added owner-only guarded session-count/launch-cadence experiments, a read-only
  NetBSD whole-VM/process/SSH profiler, independently salted synthetic-account/data
  fixture and distinct-account SSH scaling driver. Default service remains eight
  workers. Final fresh-boot NetBSD study passed 150 distinct-account SSH menu/chat
  sessions, stopped admission at 168 for headroom, reaped active workers and
  recovered. Earlier OOM/guard defect and missed authentication sampling peak are
  explicit; profiling is not production capacity or private-page attribution.

- Managed development gateway lifecycle: opt-in NetBSD rc.d scripts, locked pidfile,
  conservative stale-socket recovery, TERM/INT/HUP draining, joined/reaped workers
  and member exit after broker loss. Owner may shorten the session deadline.
  Added stalled-handoff, overload, service/worker/broker failure, SQLite contention
  and blocked-output supervision oracles; eight-worker cap remains unchanged.
  No 100–250-member capacity or production appliance claim.

- Restricted OpenSSH development access: separate database-inaccessible terminal UID,
  bounded Unix terminal handoff/broker and member-only entry point, shared ANSI/chat
  UI through inherited descriptors and owner-controlled themes. Native NetBSD 11 ARM64
  full Rust/PTY/snapshot and real application SSH oracles now pass. Added a narrow
  native CMSG bridge after reproducing a pinned libc ARM64 ancillary-layout mismatch,
  explicit FFI/descriptor ownership tests, source-tied sanitized evidence and SSH setup
  guide. Default eight-worker gate, separate BBS approval and remaining production
  load/failure/OS-confinement/appliance work are explicit; no public deployment implied.

- Owner-only private export/verify/restore: incremental committed SQLite/WAL snapshot,
  versioned current-schema artifact, exact catalog/integrity/FK/domain validation,
  owner-private staging, compact standalone file, no-overwrite publication and fresh
  restore directory/session binding. Preserves accounts/hashes/approval, rooms/posts/
  replies/unread/retry IDs and retained local chat/grants; discards live chat entries
  and receipts. Added corrupt-input/permissions/concurrency/roundtrip tests and a real
  CLI/PTY rehearsal in CI. Initial size/work limits and external-theme/encryption/
  authenticity/power-loss/NetBSD gaps are explicit; no remote backup or release claim.

### Added — 2026-10-19

- Faction BBS identity and BSD-3-Clause license.
- Current product requirements: approved membership, ANSI/BBS session, Citadel-style rooms, local content and files, IRC-style chat, optional private portal, and future chat-only federation.
- Explicit privacy/security, lightweight-operation, and no-JavaScript/TypeScript constraints.
- Codex agent guidance, GitHub contribution templates, documentation checks, and an issue-based research workflow with 35 scoped issues and two milestones.
- Verified historical archive and migration map for the retired planning system and draft seed documents.
- Architecture decision framework with mandatory gates, comparison/evidence rules, risk-based prototype requirements, refined templates, and a bounded approval-enforcement example; accepted through PR #36 / issue #1.
- Issue #3 research: private/poor/interrupted-link journeys, synthetic profile/workload controls, repeatable payload calculations with correctness tests, and explicit handoff to interaction/resource budget research. No runtime benchmark or network support is claimed.

- Issues #8/#28 research: proposed text/page/queue and tiny-host resource budgets, measured local ASCII/ANSI pacing replay, a C allocation-ownership probe, reproducible sanitized observations, and runtime/storage/operations measurement contracts. Includes compact-language, selective-assembly, specialized-OS and optional public-service hypotheses; no production stack or server benchmark claim.

- Accepted SSH-client access with optional invitation-only private-network deployment direction; clarified small hobby SBC/Pi or VM appliance goals, with MINIX considered but not mandatory.
- Proposed Rust/readiness I/O + SQLite WAL validation slice, backed by 20 native-session process runs and ten seeded SQLite recovery/export experiments. Rejected idle millisecond scans in the probe; recorded generic SQL-dump restoration failure and successful explicit record export/search rebuild. No production stack, SSH backend or appliance OS accepted.

- Accepted owner direction: Rust and NetBSD as primary target, initially validating NetBSD 11 ARM64. Native Mio/rusqlite bundled/pkgsrc SQLite checks and nine restricted OpenSSH gateway oracles passed in an isolated 2-vCPU/1-GiB guest; recorded HVF boot workaround and corrected SQLite runtime-library shadowing. No all-port, physical SBC, production security or capacity claim.

- Accepted software/VM-first sequencing: physical SBC hardware tests deferred until the owner can load a complete product; core local BBS first, Lua menus/isolated doors next. Added a concrete software build scope, modern extension/storage/network/payment research, four scoped future GitHub candidates, and a 16-oracle Rust authorization/separate Lua-worker boundary experiment. No production sandbox, blockchain, commerce, gambling or network integration claimed.

- Recorded Oblivion/2-inspired highlighted-arrow/Enter menus plus direct hotkeys, foundational sysop themes and optional seasonal palettes/login identity art. Added attributed, examined ACiD/iCE/scene ASCII references and proposed software navigation/theme/terminal oracles; no artwork bundled or menu implementation claimed. Replaced the README placeholder box masthead with text.

- First local Rust application increment: private SQLite state, one-time sysop bootstrap, bounded hidden Argon2id credentials, pending registration, current approval/role/epoch checks, revoke/reapprove and local-owner recovery. Durable chronological rooms/posts, same-room replies, unread cursors and retry reconciliation; plain-text console, Rust integration tests, PTY flow checks and selected Linux/NetBSD dependency inventory in shell-only CI. No network listener, full ANSI/themes, chat, door-game support or release yet.

- Local ANSI/theme increment: a separate bounded terminal presentation crate, arrow/Enter and direct-hotkey actions, incremental focus rows, strict versioned TOML palettes/labels/order, sysop identity/welcome/logout surfaces and manual autumn/winter overrides. Added canonical CP437/ANSI-cell import, malicious-sequence/asset rejection, real PTY identity/navigation/editor/resize/revocation tests and a documented plain fallback. No historical art, Lua/game runtime, calendar scheduler, broad ANSI emulator or network release implied.

- BBS-local chat: board/private channels, sysop create/allow/deny/kick, current approval/role/epoch/grant/join checks, lease-based presence, count-bounded since-join history, explicit slow-reader gaps and one latest retry receipt per lease. Added transactional schema 1 -> 2 migration, live draft-preserving TTY chat plus plain `/poll`, simultaneous PTY privacy/revocation/cleanup checks and a Chat menu action. No IRC wire protocol, federation, remote-login backend or capacity claim.

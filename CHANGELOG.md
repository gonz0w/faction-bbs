# Changelog

This file records meaningful project changes. Product releases will use versioned entries after a release process exists.

## Unreleased

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

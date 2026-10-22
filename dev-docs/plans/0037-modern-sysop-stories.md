# 0037: Capture modern sysop and communication stories

Issues: #95–#101/#103/#104; parents #18/#13/#25/#21/#26/#17/#84.
Status: stories and documentation captured/validated, 2026-10-21.

## Goal and constraints

Capture the owner's seven bounded outcomes: local OS-console dashboard, modern
terminal detection/graphics, NAT-aware abuse rules, NetBSD setup assistant, native
XMPP unread/private UX, real IRC reuse and optional federated discussion-room
research. Update product/agent guidance without claiming implementation, selecting
an untested runtime. Subsequent owner steering requests a stopped/preserved
playground and Netnews article formatting for discussion-room posts only. Keep
Rust/NetBSD/no-JS,
member-held private-mail keys, default-local community data and local approval.

## Progress

- [x] Read existing product/workflow and related issue contracts.
- [x] Create seven issue-linked stories with concrete acceptance criteria.
- [x] Check primary terminal/NPF/IRC/XMPP/NNTP/Matrix/ActivityPub references.
- [x] Update active requirements, boundaries, README, AGENTS and roadmap/index.
- [x] Run documentation/evidence guards; prepare the focused PR.

## Steps and decisions

Branch from current main independently of open native vault PR #94. Do not merge
#94 without owner instruction or duplicate its runtime changes. Runtime sources,
measured source hashes and host/firewall settings remain untouched. Keep the
private playground preservation separate from public repository changes.
Use docs/modern-sysop-and-communications.md for current owner stories and research
0047 for sourced facts/inferences, uncertainty and prospective experiment gates.

The owner requests local console access for operators, not a member host shell.
Advanced graphics depend on capabilities, not brand or VM placement. Firewall
rules depend on observed peers/enforcement topology; shared NAT is not member
identity. IRC wire support is under comparison; native XMPP UX cannot decrypt on
a BBS server. Room federation is authorized optional research beyond older blanket
local-only wording, not authorization to export rooms. The subsequent
clarification selects Netnews
article representation and NNTP-first validation, without adopting a production
wire runtime or changing IRC/XMPP formats.

## Validation and outcomes

Run python3 tools/check_repository.py, evidence-source/history guards and git diff
--check. Do not rerun VM/load/crypto matrices for documentation-only work. CI may
run its normal checks. No new runtime/NetBSD/Ghostty/NAT/IRC/forum interop claim.
GitHub remains canonical; linked parent updates must explicitly supersede stale
migration wording without rewriting historical measurements or closed decisions.

Repository/link/archive checks and historical/native-client/mail evidence guards
passed. Runtime, build/dependency files and historical records remain unchanged.
Seven GitHub issues and linked parent updates now hold the new scope; no new
runtime tests were needed for documentation-only changes. Required PR CI follows
the normal repository workflow. #94 remains independently open and unmerged.

## Owner follow-up: stopped playground and Netnews articles

- [x] Verify the demo QEMU/controller were already stopped; serial log records
  disk sync/unmount and its three loopback listeners are closed. Touch no other VM.
- [x] Preserve original images and create a private standalone whole-guest QCOW
  checkpoint, kernel, connection/credential helpers, existing host client and
  SHA-256 manifest. Both image-container checks and byte-equivalence comparison
  pass; no guest restore/application
  export or hardware claim. Private restart/recovery instructions stay ignored.
- [x] Confirm NNTP-style formatting covers discussion-room posts only. Create
  #103 codec/model/migration and #104 optional provider stories; narrow #101 to
  NNTP-first validation, keeping alternatives as fallback comparisons.
- [x] Update current agent/product/architecture/research/roadmap requirements.

Repository/link/archive, historical source, native-client and mail evidence
guards and diff hygiene pass for the updated PR. No runtime/dependency/schema
inputs changed and no live provider contacted.

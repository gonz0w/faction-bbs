# Faction BBS agent instructions

## Work authority

**GitHub Issues are the only backlog, plan, task status, dependency graph and
handoff.** Read the selected issue and its comments before touching code. Follow
[the workflow](dev-docs/workflow.md). No issue means no implementation: create a
finite issue first. A broad epic is context, not an executable assignment.

Use issue comments for the implementation approach, discoveries, red/green results
and continuation. Never create repository execution plans, roadmaps, TODO lists,
agent memory/status logs or a second task system. Historical archives, research,
ADRs and executed-source evidence describe past facts; they cannot assign work.
Product docs describe behavior/contracts, not sprint status.

Every development PR targets `main` and has one real `Closes #NUMBER` line per
completed executable issue. Finish all of that issue's acceptance criteria and
map them to actual tests. Partial work needs a bounded child issue **before**
implementation. Do not stretch an issue indefinitely or move its goalposts.
New bugs and discovered scope get issues before unrelated fixes. Close completed
issues on merge; an abandoned/unmerged PR leaves its issue open. Reconcile parent
epics after children complete. Use current GitHub labels, not local status copies.

## Test-first delivery

For behavior changes: derive meaningful acceptance/regression tests from the
issue, run them red before implementation, record why they fail, implement the
smallest change to green, then refactor with assertions intact. Reproduce a bug
before fixing it. Never weaken tests, skip failures or adjust resource/security
limits merely to pass CI. Record commands and actual outcomes in the issue/PR.

Documentation/research-only changes use explicit link/policy/source validation,
not artificial runtime tests. The one-time existing-implementation exception
applies only to PRs created before #181; never invent retrospective TDD evidence.
CI checks real open issue targets, completed acceptance, test mapping and TDD
metadata. It checks evidence structure, not whether a claimed red run occurred;
reviewers must inspect the tests/logs/history. See the PR template and workflow.

Run the smallest relevant tests plus required checks. UI/transport/storage changes
need appropriate real PTY/recovery/native controls; software VMs are sufficient.
Broaden tests only for changed contracts, failures or unresolved concerns. Do not
rerun every historical experiment or build a new VM for documentation changes.

## Product and implementation contracts

Read [product direction](docs/project.md), [privacy/security](docs/security-privacy.md),
[current architecture](dev-docs/architecture.md), and relevant feature docs/ADRs.
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

## Resource and evidence contracts

Preserve the normal eight sessions/5000-ms admission, two credential permits
covering queued/active/result lifetime, two kernel slots, unchanged Argon2, three
owner helpers, bounded encoded output and five-second drain. Experimental capacity
must remain explicit; old 250-member observations are not current support promises.
Keep kernel peer-UID/owned-TTY checks, opaque connection-bound sessions, descriptor/
environment hygiene and owner-only recovery operations. Never unlink live locks.
Do not silently tune allocators/NTP/sysctls, replace NPF or activate host services.

Before changing measured inputs, inspect [evidence preservation](dev-docs/evidence/README.md)
and preserve the actual previously executed bytes if not already resolvable.
Never refresh old hashes/results to current code or describe saved checks as fresh
execution. Preserve failed/rejected candidates and distinguish source checks,
software observations, graphical acceptance, hardware and production support.
Current evidence guards are discovered from CI; do not hardcode rolling counts here.

The saved owner playground stays **stopped and intact** unless the owner asks to
run it. Use only marked owned disposable fixtures, keep private data outside Git,
and prove teardown before claiming cleanup. Never publish keys, real endpoints,
member content or unlicensed games/art. Report sensitive bugs via SECURITY.md.

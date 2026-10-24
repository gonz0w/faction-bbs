# Faction engineering policy

This lasting policy adopts Agent Temper through [issue #188](https://github.com/gonz0w/faction-bbs/issues/188)
on 2026-10-23 and preserves Faction's existing owner-approved constraints.
Live assignments and continuation remain exclusively in GitHub Issues.

## Adopted method and local authority

Read the [pinned method](.agent-temper/METHOD.md). Its exact reviewed revision is
recorded in [SOURCE.md](.agent-temper/SOURCE.md); the
[SHA-256 manifest](.agent-temper/SHA256SUMS) identifies the unchanged upstream files.
The bundle retains its [BSD-3-Clause license](.agent-temper/LICENSE).

Only these profiles are selected:

| Profile | Purpose and local composition |
| --- | --- |
| [GitHub](.agent-temper/profiles/github.md) | One issue authority and finite delivery, with the local state mapping below and exact PR/check rules in the workflow. |
| [Strict test-first](.agent-temper/profiles/strict-test-first.md) | Actual red/green/refactor for behavior, including executable tooling/configuration. Preserve Faction's stronger evidence rules and fixed historical exception. |

Other bundled profiles are not active. The product/resource contracts below,
[privacy/security](docs/security-privacy.md), accepted
[decisions](dev-docs/decisions/README.md) and [workflow](dev-docs/workflow.md) are
Faction extensions; they do not become universal Agent Temper requirements.

A compatible stronger local rule remains binding. Neither an upstream update nor
file order replaces an adopted decision. A missing or conflicting required policy
stops dependent work for owner Q&A; continue independent work. Record the answer
here or in its linked canonical contract before proceeding. Settled choices persist
across agents and sessions.

## Delivery choices

| Area | Recorded policy |
| --- | --- |
| Work authority | GitHub Issues own scope, acceptance, priority, dependencies, progress, approaches and continuation. No local execution plan, roadmap, TODO/status journal or second task system. |
| Stack and support | Rust product, primary NetBSD; existing narrow C/FFI and Python stdlib/shell tooling boundaries remain. No JS/TS anywhere, including transitive/browser/tool/CI dependencies. |
| License | BSD-3-Clause; preserve third-party terms and rights. |
| Source control | Git; coherent issue-scoped branches/commits; PRs target main and close fully completed executable issues. No force push or branch deletion. |
| Verification | Strict behavioral TDD; explicit source/link/policy/example validation for prose. Never fabricate execution or weaken assertions/limits to pass checks. |
| Review and integration | Review the final change, resolve findings/conversations and pass current required checks against current main. Agents may merge accepted work under this workflow; a second maintainer's account approval is not mandated. |
| Releases and operations | No supported release or established release process exists. Explicit owner authorization is required for tags/releases, deployment, public listeners, host/security changes or new external services. Adoption grants none of these. |
| Documentation | Keep member/sysop docs in docs/ and technical contracts/research/evidence in dev-docs/. Use the pinned reader-oriented documentation guidance and original STE-informed prose for changed instructions; no formal ASD-STE100 compliance claim. |
| Decisions | The owner changes product direction, required policy and consequential authority. Agents resolve routine implementation choices within accepted scope and these constraints. |

Priorities/dependencies stay in GitHub. Select a finite ready issue in accepted
scope, respect ongoing ownership and ask about material priority conflicts. An
epic or deferred issue is not an executable assignment.

## GitHub state mapping

These equivalent names preserve existing labels; they do not migrate issue state.
Use one progress label on an open issue and none on a closed issue.

| Upstream state | Faction label |
| --- | --- |
| Backlog | status:needs-triage |
| Ready | status:ready |
| In progress | status:in-progress |
| Review | status:review |
| Blocked | status:blocked |
| Deferred | status:deferred |

GitHub open/closed state is completion authority. Keep priority:p0/p1/p2, area/type
labels and type:epic semantics. The [workflow](dev-docs/workflow.md) owns exact
readiness, PR fields, closing syntax and parent reconciliation.

## Verification and extension maintenance

Run `python3 tools/check_repository.py` and relevant tests. For this policy/checker
contract, use `python3 -m unittest discover -s tools -p test_issue_workflow.py` and
`python3 -m unittest discover -s tools -p test_check_repository.py`. Required remote
contexts are **Documentation and policy** and **Issue closure and TDD**. Their
current commands live in the [repository workflow](.github/workflows/documentation.yml)
and [issue workflow](.github/workflows/issue-policy.yml); do not copy rolling test
counts or waive failures. UI/transport/storage changes need appropriate real
PTY/native/recovery checks. Prose adoption does not require or authorize a new VM.

From `.agent-temper`, run `shasum -a 256 -c SHA256SUMS` (or the equivalent
`sha256sum -c SHA256SUMS`) and review the complete file list for unexpected additions.
Validate offline relative links and a committed source archive when provenance or
layout changes. The consumer does not need Agent Temper's Python runner.

These local extensions are versioned with Faction, outside the upstream bundle.
Their purpose is to preserve Faction's delivery, product, authority and evidence
needs. They depend on the selected profiles; verify changed contracts with their
relevant tests, current gates and separate review. The owner explicitly authorizes
replacement/removal of required policy. Routine clarifications follow bounded
issues and normal reviewed integration.

Use the [extension decision rules](.agent-temper/docs/reference/extensions.md) when
adoption exposes a useful practice or conflict. File a bounded upstream issue for
a general improvement or reusable profile; keep product-specific rules local.
Replace obsolete instructions deliberately and preserve history. Upstream acceptance
never updates this pin automatically. A framework update needs its own Faction
issue, source/manifest review and compatibility validation before integration.

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

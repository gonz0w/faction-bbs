# Worked example: where must local membership approval be enforced?

Status: completed framework exercise; architecture implementation remains unvalidated.
Date: 2026-10-19
Issue: [#1 / BBS-001](https://github.com/gonz0w/faction-bbs/issues/1)
Method: [decision framework](decision-framework.md)
Decision owner: project founder/authorized maintainer through issue and PR review.
Stopping condition: expose the tradeoffs, reject an incompatible design, and identify the next experiment without choosing a production stack.

## Question, scope, and evidence

Where should the approval check apply so an unapproved user cannot read a room or post through an alternate entry path? This exercise compares enforcement responsibilities, not languages, protocols, authentication libraries, account recovery, caching, or federation design.

| Evidence | Claim supported | Confidence and limit |
| --- | --- | --- |
| [Product requirements](../../docs/project.md) and [session](../../docs/session.md), repository revision `f16ac9c`, reviewed 2026-10-19 | Login plus local approval precedes community access, across access surfaces. | High as owner intent; no implementation evidence. |
| [OWASP authorization guidance](https://cheatsheetseries.owasp.org/cheatsheets/Authorization_Cheat_Sheet.html), sections “Deny by Default” and “Validate the Permissions on Every Request,” retrieved 2026-10-19 | Authorization should deny unmatched access and apply on each request; hiding a UI is insufficient. | High for the documented guidance; it neither selects a Faction component layout nor validates our code. |
| Architectural reasoning below | A common action boundary is a candidate way to reduce missed checks across entry paths. | Moderate as a design inference; actual coverage, resource permissions, revocation and race behavior require tests. |

Authentication and approval are necessary, not sufficient: a member still needs permission for the requested action/resource. Login, registration, and approval-status handling are separate pre-community operations. No real account data or endpoints are used here.

## Scenario and gate

Actor: an authenticated but pending local member. Stimulus: request room-read and room-post actions directly, without navigating a menu. Boundary: community action and data access. Conditions: a network-terminal entry path and a conceptual second adapter, representing a future optional console. Expected observation: both actions are denied before reading room data or committing a post; no message content is returned. Unknown/missing identity or approval state must also fail closed.

G2 (approved membership) is the discriminating gate. G1/G3–G9 remain constraints on any eventual implementation; this comparison does not assert that a conceptual boundary proves them. There is no code/dependency/transport to evaluate yet. No existing credential or remotely supplied “approved” flag is treated as local authority.

## Alternatives and gate screen

| Alternative | G2 assessment | Reasoning and tradeoff |
| --- | --- | --- |
| A — Check only during login/menu rendering | **Fail by design** | An alternate action path or a later approval change has no action-time check. UI visibility cannot establish authorization. Simplicity cannot compensate for this gate failure. |
| B — Each entry adapter enforces the policy itself | **Unknown; potentially viable** | Can enforce approval if every action/path and current state are covered. Duplicated rules can diverge or miss a new path. Shared test cases would be needed; no bypass has been measured. |
| C — Common server-side action authorization, plus adapter-level protections | **Unknown; candidate experiment** | Put the membership/action/resource decision at a boundary every community action must traverse. Adapters still authenticate/bind session context and enforce connection/input controls. Internal callers and extension entry paths must also be covered. No universal interception mechanism has been demonstrated. |

These are conceptual options. “Common” describes responsibility; it need not mean a separate service, network request, specific framework, singleton, or prescribed module layout. B remains a legitimate alternative until evidence shows otherwise.

## Qualitative comparison, uncertainty, and reversal

| Criterion | B — Adapter enforcement | C — Common action enforcement |
| --- | --- | --- |
| Security/privacy | Coverage/state-consistency burden on each adapter; untested. | Central policy may reduce divergent rules; bypass or stale state remains possible; untested. |
| Operation/recovery | Required service/component count unknown. | Does not inherently require a new service; deployment cost still unknown. |
| Resource/link/terminal behavior | Unknown; no workload or implementation. | Unknown; no workload or implementation. |
| Maintainability/reversal | Policy updates may span adapters; exact change cost unknown. | May localize policy changes but can introduce coupling; exact cost unknown. |
| Portability/extension fit | New adapters must reproduce the policy. | New callers must traverse the protected boundary; doors remain separately isolated. |

No weights, metric anchors, or numerical totals are applied: meaningful performance and implementation evidence is absent, and the applicable security gate remains unknown for B/C. A score could not override A's failure. Weight sensitivity cannot produce a numerical winner without the missing evidence; test both viable paths rather than manufacture one.

Reversal cost is **low for this document-only hypothesis**: it changes no persisted data or runtime interface. Reversal cost for a future implementation is unknown until the caller/interface/state design exists; this example does not label that future change cheap.

## Proposed falsifiable experiment — not run in issue #1

Uncertainty: can a shared action boundary cover all relevant entry paths and re-check current local authorization without bypasses? Compare B/C with the same synthetic inputs when identity/session research [#6](https://github.com/gonz0w/faction-bbs/issues/6) and validation work [#20](https://github.com/gonz0w/faction-bbs/issues/20)/[#30](https://github.com/gonz0w/faction-bbs/issues/30) define the harness. [#21](https://github.com/gonz0w/faction-bbs/issues/21) supplies session states. An internal test adapter suffices; this does not select or require a browser implementation.

Use one synthetic room, a state provider controlled by the test, and read/post operations with counters for data reads, returned content, and committed writes. Fix the action permission and state-transition semantics before running; implementations must receive identical cases. Use no live users, secrets, deployment endpoints, or persistent production data. Limit the experiment to these operations and entry paths, then remove disposable state/harness artifacts or preserve only a documented reproducer.

| Case on both entry paths | Required observation |
| --- | --- |
| Missing/unknown identity, pending, rejected, revoked, or unavailable authorization state | Deny; no room data fetched, content returned, or post committed. |
| Approved and action/resource permission granted | Read/post succeeds exactly as expected, proving the check does not merely deny everyone. |
| Approved but resource/action permission denied | Deny with no protected read/write. |
| Action invoked directly, bypassing menu | Same authorization result as menu-driven invocation. |
| Approval revoked after login but before action authorization | Next action denied on both paths. |
| Old/queued action resumes after revocation | Deny unless its explicitly defined authorization/commit point preceded revocation; record the chosen concurrency semantics before the test. |
| New entry path added | Shared authorization tests still apply; missing protection must fail the suite. |

Pass the bounded coverage hypothesis only if every denial/success case matches on both paths and no protected data effect occurs before authorization. Any counterexample fails it; investigate the missing interception/state/transaction boundary or revisit B/C. Matching cases justify further threat/race analysis, not production security certification or proof that every future caller is covered. Performance, real-network behavior, and extension isolation need separate measurements.

## Result and next action

The document exercise rejects A against an existing requirement, keeps B/C technically unvalidated, and proposes C as a hypothesis to test alongside B. **No authorization architecture, runtime, language, protocol, or storage schema is adopted.** The framework made a useful exclusion while exposing missing evidence; the next security experiment belongs in the existing dependent issues, not an expanded implementation under #1.

Reopen this analysis if a viable adapter policy eliminates the duplication risk, a common boundary has an uncovered caller, or revocation/transaction requirements change. No runtime experiment or latency/resource/security benchmark was run for this example.

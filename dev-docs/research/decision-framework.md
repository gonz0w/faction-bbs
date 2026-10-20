# Architecture decision framework

Status: proposed for adoption through [issue #1](https://github.com/gonz0w/faction-bbs/issues/1) and [ADR 0003](../decisions/0003-architecture-decision-framework.md). Updated 2026-10-19. These are project evaluation rules, not an external certification or a selected architecture.

## Use the smallest sufficient decision record

Name one question, its issue, decision owner, affected requirements, scope, dependencies, and stopping condition. Routine reversible edits need no matrix or ADR. A material change to a runtime, data model, security boundary, protocol, dependency, topology, or extension mechanism needs an ADR. A research report may conclude defer, reject, or insufficient evidence.

Use [the research template](template.md) while investigating and [the ADR template](../decisions/template.md) for the proposed decision. A short record may combine them if it contains the same information; link evidence rather than copying it. Record acceptance/adoption only after the issue/PR review establishes authority. Preserve superseded decisions.

## 1. Translate requirements into observable scenarios

Read [the product](../../docs/project.md) and [security/privacy requirements](../../docs/security-privacy.md). For each decision-critical scenario, state the actor/source, action or failure, affected boundary, operating conditions, expected response, and measurable acceptance observation. Identify whether the condition is owner-confirmed, documented, measured, inferred, or still unknown.

“Fast,” “lightweight,” “private,” and “compatible” need scenario-specific measures. Use issues [#3](https://github.com/gonz0w/faction-bbs/issues/3), [#8](https://github.com/gonz0w/faction-bbs/issues/8), and [#28](https://github.com/gonz0w/faction-bbs/issues/28) for link/terminal/resource budgets. Do not invent production targets to make a comparison look complete. Missing targets can block ranking while still allowing useful research.

CMU SEI connects architectural evaluation to prioritized quality scenarios, interacting goals, and risk discovery. We borrow those ideas without claiming to conduct its full ATAM process. [Primary source, retrieved 2026-10-19](https://www.sei.cmu.edu/library/architecture-tradeoff-analysis-method-collection/).

## 2. Apply non-negotiable gates before ranking

For every alternative, mark each applicable gate **pass**, **fail**, **unknown**, or **not applicable**. Attach a requirement/evidence reference and rationale to each judgment; distinguish a design argument from an observed test result. A scoped subcomponent need not implement every product feature, but it must not contradict a system-level gate. Explain every not-applicable entry.

| Gate | Required boundary |
| --- | --- |
| G1 — No JS/TS | No JavaScript/TypeScript in source, tooling, direct/transitive dependencies, or browser assets. |
| G2 — Approved membership | Server-side authentication, local approval, and action/resource permissions protect every community action across all entry paths. |
| G3 — Local authority | Accounts/approval and BBS content stay local. Only explicitly participating shared chat may federate; local-only chat stays local. |
| G4 — Independent operation | No mandatory public HTTP, central service, directory listing, federation, or future network adapter. |
| G5 — Privacy and ownership | Exportability and local data control; minimal opt-in telemetry; no default private-content or credential logging. |
| G6 — Domain/network separation | Addressing, discovery, routing, and connection mechanics do not redefine community semantics. |
| G7 — Useful BBS baseline | Approved-session/ANSI-identity/menu flow and useful text interaction are preserved, with capabilities and fallback explicitly specified. |
| G8 — Optional bounded extensions | Doors/extensions have permission, lifecycle, resource, upgrade, and failure-isolation boundaries; one optional runtime is not mandatory for every node. |
| G9 — Distributable inputs | Dependencies/assets have recorded provenance and terms appropriate for their intended inclusion; third-party material is not implicitly relicensed by this project. |

**Fail eliminates an option under the current requirements. Unknown is not a pass.** An option with an unknown applicable gate can be investigated, but cannot be accepted for production use until the gate is resolved. Scores cannot override gates. Changing a product constraint is a separate explicit owner decision with updated docs/ADR; the framework grants no exception.

## 3. Compare viable alternatives against declared criteria

Include the simplest credible alternative and defer/reuse where relevant. Do not create straw options merely to favor a familiar language. Record the relevant versions, operating systems, CPU/terminal/link conditions, dependencies, and actual deployment obligations.

The following **proposed starting weights** reflect this project's priorities; they are not scientific constants or weights prescribed by the cited sources. Before assessing options, the decision owner/reviewer records the applicable criteria, weights, metrics, and score anchors in the report. Explain changes. If a criterion is irrelevant, remove it for all options and renormalize the remaining weights to 100; do not remove difficult unknowns. Record correlated criteria so the same advantage is not counted twice.

| ID | Criterion | Starting weight | Decision-specific evidence to seek |
| --- | --- | ---: | --- |
| S | Security and privacy beyond the mandatory floor | 25 | Attack surface, permissions, safe failure, metadata/retention controls. |
| O | Sysop operation and recovery | 20 | Required services, setup/update steps, backup/restore, failure recovery. |
| R | Resource efficiency | 15 | Idle/active RAM, CPU, disk, startup, connection/workload limits on named hardware. |
| T | Terminal and constrained-link experience | 15 | Task bytes/latency, key/encoding behavior, plain-text/ANSI usability, interruption recovery. |
| M | Maintainability and reversibility | 10 | Change surface, testability, migration/rollback cost, coupling and upgrade burden. |
| P | Portability and dependency fit | 10 | Supported targets, library maturity/provenance, build/deployment dependencies. |
| X | Optional adapters and extension fit | 5 | Isolated doors, future transports/chat, and extension lifecycle without core complexity. |
| | Total | 100 | |

Use descriptive comparisons when measurements/anchors are missing. Where ranking is justified, define operational anchors **before scoring** on a 0–4 scale: 0 unacceptable for that criterion, 1 material shortfall, 2 meets the declared baseline, 3 useful demonstrated advantage, 4 substantial demonstrated advantage. Define what those labels mean for this particular metric and workload. A gate failure remains disqualifying regardless of this scale.

Use **U**, not zero, for missing evidence; keep confidence separate from score. Do not publish an overall numerical winner while an applicable gate or decision-critical score/target is unknown. An explicit score range may show uncertainty without implying acceptance. For complete comparable scores, `total = sum(weight * score / 4)` yields 0–100. It organizes a tradeoff discussion; it is not a probability, proof of safety, or automatic choice.

Before recommending from scores, vary the influential weights and uncertain score ranges, keeping weights at 100; record the ranges, changed rankings, and rationale. If a plausible variation changes the result, gather targeted evidence or present the unresolved preference. Do not change weights silently after seeing an unwanted winner. Explain any recommendation inconsistent with the ranking.

NASA's decision-analysis guidance distinguishes mandatory and enhancing criteria, calls for defined scales and uncertainty handling, and tailors analytical effort to the decision. Our gates, weights, labels, and workflow are a local adaptation. [Primary source, retrieved 2026-10-19](https://www.nasa.gov/reference/6-8-decision-analysis/).

## 4. Record evidence quality and claim-level confidence

| Kind | Meaning and limitation |
| --- | --- |
| Owner requirement | Establishes desired behavior/constraints; does not prove technical feasibility. |
| Primary documented behavior | Official specification, project documentation/source, or original research, with exact supported claim and version/date. It does not establish Faction integration or target performance. |
| Measured result | Reproducible command/test, revision, environment, workload, observations, and limits. A small happy-path demo is not failure/security validation. |
| Inference | Reasoning from named evidence, with assumptions and possible counterexamples. It is not a measured result. |
| Unknown | A material missing claim/target, assigned a next action or a reason to defer. |

Record URL or repository revision, retrieval date, version/section, claim, relevance, contrary evidence, and limitations. Prefer primary sources; use secondary material for discovery, not as the sole basis of a material technical choice. Recheck moving documentation before adoption if its version or applicable behavior has changed. If a primary source is unavailable, record the limitation and reduce confidence.

Use **high / moderate / low** for each important claim: high when relevant evidence covers the stated conditions without a material contradiction; moderate when sources or a bounded experiment support only part of the claim; low when substantial assumptions, extrapolation, or conflicting evidence remain. Explain the label and what would change it. Owner confidence and runtime confidence must remain separate; neither source counts nor averaged scores establish confidence. State the weakest unresolved decision-critical claim in the conclusion.

## 5. Choose documentation or prototype evidence by risk

| Question or consequence | Minimum evidence before adopting the affected technical choice |
| --- | --- |
| Reversible documentation/process convention | Trace to requirements, compare alternatives, and exercise the convention on a bounded example; no product prototype is required. |
| Documented interface or compatibility fact | Applicable versioned primary source; add a targeted probe when local integration, ambiguous behavior, or environment matters. |
| Approval, privacy, trust/exposure, untrusted input, or isolation boundary | Relevant threat model plus negative/failure experiments across affected paths before claiming enforcement. A design sketch alone is insufficient. |
| Latency, payload, RAM/CPU, throughput, startup, or “lightweight” claim | Representative measured prototype against declared hardware/workload/link budgets. |
| Durability, storage/schema, replication, data migration, or hard-to-reverse protocol | Representative failure/recovery and round-trip/migration experiments, plus rollback/export limits. |
| Legacy door, browser console, terminal encoding, constrained or future-network integration | Bounded end-to-end experiment of the disputed compatibility behavior; record unsupported combinations. |

Prototype requirements apply to **adopting the technical option**, not to completing this framework issue. An early design ADR can remain proposed with its test plan. New research may stop after disproving a gate or identifying a blocking unknown. Never introduce a production stack to build an illustrative process example.

For every experiment state one uncertainty, comparable alternatives, environment/revision, inputs and controls, pass/fail observations, failure/recovery cases, data handling, bounded effort/cleanup, and next action on each outcome. Agree on these before running it. Preserve commands/results and failed observations, not just screenshots or an unsupported “passed.”

## 6. Record reversal cost and stop deliberately

Describe what changing the choice would require: affected code/interfaces, user data, persisted formats, identity/history, deployment components, compatibility, rollback/export path, and irreversible losses. Classify **low** (localized, no persisted/user-visible migration), **moderate** (coordinated change or bounded migration), or **high** (persistent format/protocol/identity changes, broad deployment coordination, or loss risk). Explain the label; do not invent time/cost estimates without evidence. High reversal cost requires relevant migration/recovery evidence before adoption.

Conclude **recommend for adoption**, **propose an experiment**, **defer**, or **reject** with confidence, outstanding gates/unknowns, decision authority, and the evidence that would reopen it. The owner or authorized maintainer accepts a material decision through review/merge with its rationale. Record constraints/threshold changes explicitly. Close a research issue when its actual acceptance criteria are met; that alone does not accept an unvalidated runtime architecture.

## Review checklist

- [ ] One named question, scope, owner, and stopping condition; no hidden stack selection.
- [ ] Requirement/scenario traceability and gate outcomes with rationale, including unknowns.
- [ ] Credible alternatives; metrics/anchors/weights recorded before any numerical ranking.
- [ ] Primary sources and claim-level confidence; assertions distinguished from measurements.
- [ ] Sensitivity analysis where scored; no totals conceal decision-critical unknowns.
- [ ] Prototype requirement justified; pass/fail conditions and failure/recovery cases stated.
- [ ] Reversal cost, limitations, contrary evidence, reopening triggers, and next action recorded.
- [ ] ADR/research/README/issue links accurate; no unsupported capability or security claim.

The [approval-boundary example](0001-approval-enforcement-example.md) demonstrates this method without an implementation choice.

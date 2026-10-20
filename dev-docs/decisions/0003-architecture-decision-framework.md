# 0003: Use gated, evidence-based architecture decisions

Status: accepted
Acceptance: owner authorized merge on 2026-10-19; [PR #36](https://github.com/gonz0w/faction-bbs/pull/36) merged as `2b23e39`. Process adoption only; no runtime architecture accepted.
Date: 2026-10-19
Owner: project founder/authorized maintainer through review and merge
Issue: [#1 / BBS-001](https://github.com/gonz0w/faction-bbs/issues/1)
Supersedes: none; refines the workflow in ADR 0001

## Question and constraints

How should Faction compare architecture options without premature stack selection, hidden requirement violations, or untestable recommendations? Security/privacy, approved membership, local ownership, text/BBS behavior, independent operation, no JS/TS, and bounded optional extensions are product constraints. This decision covers research process only.

## Alternatives

| Approach | Benefit | Cost/risk |
| --- | --- | --- |
| Keep the short templates and use ad hoc judgments | Little ceremony. | No shared gate, scoring/unknown, confidence, or prototype threshold rules. |
| Require a full formal architecture evaluation for every choice | Structured analysis of interacting qualities. | Excessive process for small/reversible research questions; no mature architecture to evaluate yet. |
| Use a small gated framework with risk-based evidence requirements | Shared constraints and explicit tradeoffs, scaled to decision consequence. | Requires disciplined scenario definitions and review; weights can create false precision if used without evidence. |

## Decision

Adopt the third approach in [the framework](../research/decision-framework.md). Screen mandatory gates first, then compare viable options. Keep unknowns visible, tie confidence to claims, predeclare metrics/anchors/weights before numerical scoring, test ranking sensitivity, and make prototype requirements proportional to risk/reversal cost. Use the refined research/ADR templates. Routine reversible edits need no matrix.

The proposed starting weights and confidence labels are project conventions subject to review, not external standards. They cannot waive a product requirement or decide a production stack. A research conclusion can be defer/reject/experiment; acceptance of this process must not imply acceptance of a worked example's unvalidated architecture.

## Primary evidence

Sources retrieved 2026-10-19:

- [Current project requirements](../../docs/project.md) and [security/privacy requirements](../../docs/security-privacy.md), baseline revision `f16ac9c`: owner authority for product constraints, not implementation proof.
- [NASA decision analysis](https://www.nasa.gov/reference/6-8-decision-analysis/): criteria/scales, mandatory versus enhancing requirements, uncertainty, and proportionate analytical effort. Faction's exact gates/weights are its own adaptation.
- [CMU SEI ATAM collection](https://www.sei.cmu.edu/library/architecture-tradeoff-analysis-method-collection/): scenario-based quality/tradeoff/risk evaluation. This project does not claim a full ATAM assessment.

## Confidence, tradeoffs, and reversal cost

Moderate confidence that this is sufficient for the next bounded investigations: it traces the owner constraints and exposes missing evidence in the worked example. No completed stack evaluation demonstrates its long-term effectiveness. Review whether it causes duplicated documents or rewards false numerical precision.

Low reversal cost: changing a document/template convention requires no user-data migration or deployed protocol change. Preserve old rationale and explicitly revise scoring rules when needed. High-cost future technical choices need their own reversal analysis and relevant recovery/migration evidence.

## Falsifiable validation and reopening

The [approval-boundary exercise](../research/0001-approval-enforcement-example.md) must reject a requirement-violating option, distinguish owner/documented/inferred evidence, leave unmeasured criteria unknown, and specify a negative/success-case experiment without selecting a stack. Review all three acceptance criteria of #1 and run repository checks. This validates the framework exercise, not approval enforcement.

Revisit the framework if the next actual comparisons cannot produce useful decisions without missing budgets, if evidence/unknown handling hides a gate failure, or if process overhead exceeds the bounded research. Do not invent totals to force a result. Technical prototypes remain work in the relevant dependent issues.

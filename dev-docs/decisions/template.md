# NNNN: Decision title

Status: proposed / accepted / rejected / superseded
Date: YYYY-MM-DD
Owner: decision authority and acceptance/review record
Issue: GitHub issue link
Supersedes: none or earlier ADR
Research/evidence: report link or evidence recorded below

Use [the decision framework](../research/decision-framework.md). Keep the record proportionate; link shared evidence rather than duplicating it.

## Question, scope, and scenarios

Name one decision, the user problem, affected requirements/issues, dependencies, and stopping condition. State the actor/action, boundary, operating conditions, expected response, and measurable observation for decision-critical scenarios. Mark unknown budgets; do not invent targets.

## Mandatory gate screen

| Alternative | Applicable gate | Pass / fail / unknown / not applicable | Evidence and rationale |
| --- | --- | --- | --- |
| To evaluate | Framework gate ID | Unknown | What must be established? |

Map all relevant gates; explain exclusions. Reject failures. Unknown gates block production adoption, not further research. Do not trade a gate away through scoring.

## Alternatives, criteria, and comparison

Include credible simple/reuse/defer options where relevant. Define decision-specific metrics, thresholds, scoring anchors, applicable weights, and versions/environments before scoring. Explain deviations from starting weights and overlapping criteria. Use qualitative comparisons when evidence is insufficient; use U for unknowns and withhold unsupported totals/winners. Record sensitivity ranges and ranking changes if scores are used.

## Evidence and confidence

| Claim | Owner / primary documented / measured / inference / unknown | Source/revision, retrieval date and version/section | Applicability, contrary evidence, limits | Confidence and reason |
| --- | --- | --- | --- | --- |
| To evaluate | Unknown | To obtain | Unresolved | Low: no evidence yet |

Keep requirement certainty separate from runtime feasibility. State the weakest unresolved decision-critical claim.

## Decision, tradeoffs, and reversal

Conclude recommend/experiment/defer/reject, name the authority and adoption state, explain rejected alternatives and costs. Describe rollback/export, interfaces/data/deployment changes, and irreversible losses. Classify reversal cost low/moderate/high with reasons; a document-only hypothesis does not establish implementation reversal cost.

## Required validation and actual results

Use the framework's risk/prototype rules. State whether documentation is sufficient or a prototype is required, and why. For each experiment name one uncertainty, comparable inputs, environment/revision, data/resource/link conditions, measurable pass/fail observations, positive and negative/failure/recovery cases, bounded effort/cleanup, and what each result changes. Record actual commands/results and unavailable checks separately from proposed tests.

## Unknowns, reopening, and next work

Record blockers, contrary evidence, assumptions that would invalidate the choice, and existing issue links. Acceptance of research does not by itself validate an implementation. Update affected current docs and README claims when a decision is adopted.

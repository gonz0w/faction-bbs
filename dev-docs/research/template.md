# Research question

Status: proposed / in progress / completed
Date: YYYY-MM-DD
Issue: GitHub issue link
Owner: researcher and decision/review authority
Decision affected: ADR link or pending decision
Stopping condition: bounded deliverable, experiment, rejection, or identified blocker

Use [the decision framework](decision-framework.md). A completed report may leave a technical decision proposed, rejected, or deferred.

## Question, scope, and scenarios

Name one uncertainty, relevant owner requirements/issues, dependencies, and deliberately deferred work. Define the actor/action, affected boundary, operating conditions, expected response, and measurable observation. Mark unknown link/resource/compatibility targets.

## Gates and alternatives

List credible alternatives, including simple/reuse/defer options where relevant. Map applicable framework gates as pass/fail/unknown/not applicable with evidence and rationale. Reject constraint violations; unknown gates block production adoption. Do not infer a stack or protocol from an interaction requirement.

## Criteria and comparison

Declare metrics, thresholds, 0–4 anchors and applicable weights before any scoring. Explain changed/irrelevant/overlapping criteria and sensitivity ranges. Use qualitative tables and U where evidence is missing; withhold unsupported numerical winners. Separate comparative preference from requirement compliance.

## Evidence and claim-level confidence

| Claim | Evidence kind | Source/revision and retrieval date/version/section | Relevance, limitations, contrary evidence | Confidence and reason |
| --- | --- | --- | --- | --- |
| To investigate | Owner / primary documented / measured / inference / unknown | To obtain | State assumptions | Low / moderate / high, with basis |

Do not label source documentation as a measured Faction integration. State the weakest unresolved decision-critical claim.

## Prototype requirement and experiment

Apply the framework's risk/reversal rules. State why source/document review is sufficient or which prototype is required. Specify one uncertainty, comparative inputs/controls, environment/revision, hardware/link/resource conditions, pass/fail observations, positive and negative/failure/recovery cases, privacy/data handling, effort/cleanup boundary, and next action for either result. Define observations before running it.

## Actual results and limits

Record reproducible commands and results; distinguish completed document exercises, measured runtime behavior, and tests merely proposed. Include contrary evidence and unavailable checks. A small demo does not establish production security, durability, performance, or compatibility.

## Conclusion, reversal, and next work

Recommend adoption, propose an experiment, defer, or reject. Explain confidence, outstanding gates, tradeoffs, and reversal cost (interfaces, data migration/export, deployment, irreversible loss). Name authority/adoption state, reopening triggers, and existing follow-up issues. Update the ADR and affected docs/README as appropriate; do not create a duplicate backlog.

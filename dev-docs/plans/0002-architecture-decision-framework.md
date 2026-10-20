# 0002: Establish the architecture decision framework

Issue: [BBS-001 / #1](https://github.com/gonz0w/faction-bbs/issues/1)
Status: completed; process accepted by owner-authorized merge of PR #36 on 2026-10-19

## Goal and boundaries

Refine the ADR/research templates, define evidence/scoring/prototype rules, and apply them to one bounded example. Preserve the confirmed product requirements. Do not select a production stack, implement authorization, or resolve the dependent identity/access issues.

## Progress

- [x] Read issue #1, product/security requirements, workflow, current templates, and existing decisions.
- [x] Review primary guidance from NASA decision analysis, CMU SEI architecture evaluation, and OWASP authorization.
- [x] Write the framework, updated templates, proposed process ADR, and membership-approval worked example.
- [x] Check acceptance coverage, sources/claims, links, policy, and archive integrity.
- [x] Publish [PR #36](https://github.com/gonz0w/faction-bbs/pull/36) and verify CI; leave adoption to review/merge.

## Decisions and discoveries — 2026-10-19

- Separate non-negotiable gates from comparative criteria so efficiency cannot compensate for a membership/privacy violation.
- Unknown evidence must remain visible; no numerical winner is justified without decision-specific anchors and comparable evidence.
- Use risk and reversal cost to determine when a prototype is mandatory. A documentation example can test the method without validating runtime behavior.
- The worked example concerns approval enforcement boundaries, with implementation/security experiments deferred to existing issues #6, #20, #21, and #30.

## Steps and validation

Work on `research/1-decision-framework`. Run `python3 tools/check_repository.py`, the existing checker tests, and `git diff --check`. Review the example against all three issue acceptance criteria and confirm no language/protocol/schema/runtime is selected. Open a PR with the proposed framework and validation evidence; record the CI result and outstanding review here.

## Acceptance coverage and local results — 2026-10-19

| Issue #1 criterion | Delivered evidence |
| --- | --- |
| Refine ADR/research templates against product/security constraints | Both templates link the mandatory gate screen, scenario/criteria/evidence fields, risk-based validation, reversal, and adoption state. |
| Define scoring, evidence quality, confidence, reversal, and prototype thresholds | The framework defines proposed criteria/weights and 0–4 anchors, unknown/sensitivity rules, claim-level confidence, reversal categories, and documentation-versus-prototype triggers. |
| Apply to one bounded example without selecting a stack | The approval-boundary report rejects menu/login-only checks by requirement reasoning, keeps B/C unvalidated, and proposes negative/positive experiments in existing issues. No runtime architecture is adopted. |

- `python3 tools/check_repository.py`: passed for current links, foundation policy, issue mapping, and archived source hashes.
- `python3 -m unittest discover -s tools -p 'test_*.py'`: all six existing repository-checker tests passed.
- `git diff --check`: passed. Editorial review checked the proposed starting weights sum to 100, requirements cannot be traded for scores, unknowns do not become zero/pass, and the example makes no runtime/security measurement claims.
- Primary sources were actually retrieved on 2026-10-19 and attributed at their supported claims. No historical archive, product constraint, runtime dependency, or executable code was changed.
- Runtime approval tests, resource measurements, and the production stack remain unvalidated and outside issue #1. Framework adoption is proposed pending PR review/merge.

## Publication and review handoff

[PR #36](https://github.com/gonz0w/faction-bbs/pull/36) delivers all three acceptance criteria and closes #1 when merged. The `Documentation and policy` check passed for research commit `ccef803` in [GitHub CI](https://github.com/gonz0w/faction-bbs/actions/runs/39348219595). The final handoff-document revision is checked by the same required PR workflow; its live status remains in the PR. No product-code or runtime experiment was added.

The framework and ADR 0003 remain proposed until authorized review/adoption. Issue #1 stays open pending merge. Follow-on research should use existing issues #3/#8/#28 to establish journey/link/resource targets and #6/#20/#21/#30 for the approval experiment; this handoff does not start or resolve those issues.

## Adoption follow-through — 2026-10-19

The owner instructed “merge it and move on.” PR #36 was squash-merged as `2b23e39`, closing issue #1. This supersedes the pending-review state in the historical handoff above. ADR 0003 and the framework now record acceptance of the process only; the approval-boundary example remains unvalidated architecture research.

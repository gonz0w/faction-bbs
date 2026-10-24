# Issue-only agentic development

GitHub Issues are the sole work tracker. Issue bodies/comments hold scope,
acceptance, dependencies, approach, red/green results, discoveries and handoffs.
There is no repository backlog, roadmap, execution-plan directory or agent status
journal. `docs/` describes the product; `dev-docs/` holds technical contracts,
research, decisions and immutable validation. Git history preserves retired plans.

## A finite issue is the unit of delivery

1. **Find or create the issue first.** State one observable outcome, acceptance
   criteria, scope/non-goals, dependencies and concrete test requirements. A bug
   includes reproduction/expected/actual behavior; research has one question and a
   falsifiable outcome. Do not implement speculative features under an epic.
2. **Make the issue executable.** If it cannot finish in a focused PR, create child
   issues before coding and list them in the parent. Preserve original requirements
   and explain any split. Do not silently shrink acceptance or add future upgrades
   as new blockers after the agreed deliverable is complete.
3. **Take one coherent batch.** Use a branch such as `feat/184-directory-submissions`
   or `fix/185-concurrent-bootstrap`. A batch may close several related issues;
   each must be complete. Mark active work `status:in-progress` and put the brief
   approach in an issue comment. No local plan file or AGENTS progress update.
4. **Red.** Write the smallest meaningful acceptance/regression test before changing
   behavior. Execute it against the old code; record the command and observed
   failure. A bug test must expose its cause, not simply mirror the implementation.
   Missing-module failure is valid for a new module, followed by positive/negative
   contract tests. Never claim a hypothetical or retrospective failing run.
5. **Green.** Implement only the issue outcome. Keep negative authorization, bounds,
   classic fallback and failure behavior covered. Record the passing command and
   result. Keep assertions; investigate unexpected failures rather than deleting,
   skipping or relaxing them. If the acceptance itself is wrong, explain and update
   the issue before changing its oracle.
6. **Refactor and review.** Improve clarity with tests green. Review current
   authorization, data locality, dependencies, resource limits and actual support
   claims. Relevant README/product/technical documentation changes belong in the
   same PR. Preserve historical executed bytes before editing measured inputs.
7. **Open a closing PR.** Target `main`; put a separate `Closes #NUMBER` line for
   each completed open issue. Include `## Acceptance and tests` mapping and
   `## TDD evidence` with `Mode: behavior`, concrete `Red:`, `Green:` and `Refactor:`
   lines. Update the issue acceptance checklist only after evidence demonstrates it.
   Leave the issue open, with `status:review`, until merge. CI rejects nonexistent,
   closed, PR or epic targets, unchecked acceptance and missing evidence.
8. **Merge and reconcile.** The required `Documentation and policy` check must pass
   on the exact current head against current `main`; resolve review conversations.
   No second-maintainer approval is mandated for this hobby repository. GitHub
   closes the linked issues on default-branch merge. Verify that closure, remove
   stale progress labels, reconcile parent children and report outcomes/remaining
   issue numbers. A PR closed without merging leaves its issues open; return them
   to ready/blocked/deferred with a reason. No force push or branch deletion.

Newly discovered bugs or additional behavior get issues before unrelated work.
Put sanitized reproduction, dependencies and tests in those issues, then either
finish the current scope or explicitly record a block. A change needed to satisfy
existing acceptance stays in its issue; do not manufacture a ticket per line edit.
No permanent monitor/automation or new external agent service is required.

## Tests and honest exceptions

Behavior changes include application code, test/check tooling, executable scripts,
service configuration and CI behavior. Use red → green → refactor and existing
relevant tests; check more broadly only when changed contracts or failures justify
it. Actual UI/transport/storage changes need meaningful PTY/native/recovery evidence
where applicable. Software VM evidence is enough for software work; physical SBCs,
new 250-user runs and hardening are not routine feature prerequisites.

For documentation-only or research-only prose, use `Mode: docs` or `Mode: research`
and a substantive `Reason:` naming actual link/source/policy validation. These
modes cannot cover executable/configuration changes. Research prototype code has
observable behavior and uses behavior tests. The one-time `Mode: existing-implementation`
with `Exception: #181` applies only to PRs created before #181 opened, plus an
honest `Reason:`; no new work can use that cutoff. Existing failures stay visible.

The metadata gate validates structure and actual GitHub issue state; it cannot
prove a red run occurred or that tests are adequate. Review code, assertions and
recorded commands/logs. Never treat a green checkbox as a security or completeness
proof. An unavailable check is an explicit gap, not a pass or silent waiver.

## States and readiness

Use one progress label: `status:needs-triage`, `status:ready`, `status:in-progress`,
`status:review`, `status:blocked` or `status:deferred`. Open/closed GitHub state is
completion authority; closed issues need no progress label. `type:epic` identifies
parents, never a standing coding assignment. Parent checklists link real child
issues and are reconciled when a child changes; there is no second copied roadmap.
`priority:p0/p1/p2`, area and type labels support selection without extra trackers.

An issue is ready only when outcome, non-goals, dependencies, acceptance and test
plan are clear. A blocked issue names what would unblock it. A deferred issue is
not current authorization. Close already delivered historical contracts with
merged-PR/evidence links; do not falsify old PR bodies or claim past TDD. Accepted
future scope survives in explicit issues, never in local continuation paragraphs.

## Repository and CI

Run `python3 tools/check_repository.py` and relevant tests. The maintained workflow
contains offline source/policy checks and the Rust/real-terminal suite. Use the
current workflow commands rather than copying rolling check/test counts here.
Validate a clean committed source archive when provenance/link layout changes.
No JS/TS, marketplace actions, write token, elevated PR event or auto-merge bot.

The separate required `Issue closure and TDD` workflow uses the read-only GitHub token and API to verify real current
same-repository open issues and changed-file types. PR body edits rerun this lightweight gate without restarting the Rust build; issue
edits alone do not automatically rerun PR CI, so refresh the check after changing
linked issue acceptance/status before merging. API failure fails closed. Review
workflow/policy changes like other executable changes; the checker does not make
an unreviewed PR trusted. Native credentials/private fixture data never enter Git.

## Primary references

Reviewed 2026-10-24. This repository adopts a small vendor-neutral GitHub/TDD loop,
not a second agent framework or a new runtime dependency.

- [GitHub: well-scoped agent tasks](https://docs.github.com/en/copilot/tutorials/cloud-agent/get-the-best-results): concrete problem, acceptance and relevant code context.
- [GitHub: issue closing links](https://docs.github.com/en/issues/tracking-your-work-with-issues/using-issues/linking-a-pull-request-to-an-issue): closing keywords take effect on default-branch merge.
- [Anthropic: agent verification practices](https://code.claude.com/docs/en/best-practices): concrete independent verification, bounded tasks and concise persistent instructions.

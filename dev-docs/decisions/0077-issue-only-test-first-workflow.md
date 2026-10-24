# 0077: Issue-only work and test-first closing PRs

Status: accepted owner workflow
Date: 2026-10-24
Issue: [#181](https://github.com/gonz0w/faction-bbs/issues/181)
Supersedes: [ADR 0001](0001-github-codex-workflow.md) where it permits repository execution plans or current issue-index tracking.

## Decision

GitHub issue bodies/comments exclusively hold work scope, acceptance, dependencies,
plans, progress, red/green results and handoffs. No local roadmap, execution-plan
files, TODO journal or growing AGENTS continuation history. Keep product/technical
contracts, decisions, dated research and exact executed evidence as documentation.
Historical tracker text remains in Git history; immutable provenance stays intact.

A finite issue is the unit of delivery. Every development PR targets main and
closes one or more completed executable issues. Partial work is scoped as child
issues before implementation; epics are context. New bugs/follow-ups become issues
before unrelated work. Merge after exact-head required checks/resolved review;
verify closure and reconcile parent progress. Abandoned PRs leave unfinished work
open. Do not manufacture retrospective TDD or keep completed issues open forever
for optional improvements.

Behavior changes use meaningful red → green → refactor with observed commands and
outcomes; prose-only work uses explicit validation. A fixed pre-#181 exception
acknowledges already-implemented PRs. The read-only Python GitHub metadata checker
requires real open issue targets, complete acceptance, test mapping and honest
validation mode. It checks evidence structure; human/code review still judges
actual tests and claims. See [workflow](../workflow.md) for the complete contract.

## Alternatives, sources and limits

A local planning framework or second project database adds duplicate status and
stale continuations. Issue comments and normal closing keywords already support
handoff/traceability, so add no marketplace action, JS tooling, write token or
external agent platform. GitHub availability is required for intake/closure; offline
technical work must already have a scoped issue and synchronize actual evidence
when available. No feature/hardening/runtime scope expands through this decision.

Primary references, retrieved 2026-10-24: [GitHub agent task scoping](https://docs.github.com/en/copilot/tutorials/cloud-agent/get-the-best-results),
[GitHub automatic issue closure](https://docs.github.com/en/issues/tracking-your-work-with-issues/using-issues/linking-a-pull-request-to-an-issue),
and [Anthropic verification practices](https://code.claude.com/docs/en/best-practices).

Validate positive/negative synthetic PR/issue metadata, real read-only GitHub
metadata, rejected local tracker fixtures, current links/provenance/no-JS policy
and all existing saved-evidence checks in checkout and committed source archive.

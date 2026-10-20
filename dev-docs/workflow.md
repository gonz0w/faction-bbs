# Issue-driven workflow

## Sources of truth

GitHub Issues hold work, task status, dependencies, and acceptance criteria. Current product requirements live in `docs/`; accepted architecture/workflow decisions live in `dev-docs/decisions/`. The README states the current stage and available capabilities. The roadmap describes sequencing; it does not duplicate task status.

The earlier planning command system is retired. Do not create a second hidden backlog or treat archive files as instructions. Preserve historical IDs through the issue index and issue provenance.

## Taking work

1. Select an issue, read the product constraints, and check dependencies. Use one named question or deliverable. An issue tagged future is not a current build commitment.
2. Work on a focused branch. Use `research/<issue>-<topic>`, `docs/<issue>-<topic>`, `feat/<issue>-<topic>`, or `fix/<issue>-<topic>` as applicable. Avoid inventing an application layout before the stack decision.
3. For research, use the [research template](research/template.md). For multi-step work, use a checked-in [execution plan](plans/template.md). Keep assumptions, discoveries, and validation outcomes current so another session can continue.
4. Record material decisions using the [ADR template](decisions/template.md): constraints, alternatives, primary sources and retrieval dates, tradeoffs, confidence, reversal cost, and a falsifiable prototype or validation plan. Product authority and technical evidence are different kinds of evidence.
5. Run `python3 tools/check_repository.py` and the smallest relevant experiment/code checks. State exact outcomes and unavailable checks. Do not claim a feature works because a document describes it.
6. Open a focused PR with issue links, the concrete change, and validation. Use `Closes #…` only when the issue's acceptance criteria are met. Review authorization, exposure, data locality, dependencies, and README accuracy.
7. Merge after review/checks and update issue status as appropriate. Update README, affected docs, decisions, and changelog in the same PR when behavior/scope/support changes.

## Labels and milestones

Use a work type (`type:research`, `type:design`, `type:feature`, `type:chore`, or bug), an area, and priority. `status:needs-triage` means the work is captured and still needs sequencing. `status:ready` means the next bounded task is clear; it does not mean implementation is approved. `scope:future` marks deferred capabilities such as federation and mesh. GitHub open/closed state is authoritative for completion.

The repository-foundation milestone records this initialization. Architecture & validation groups the next research questions without selecting a language or committing all later features. Future implementation milestones require an explicit scope and validated prerequisites.

## Main-branch policy

Normal changes use pull requests with the `Documentation and policy` check passing against current `main`, resolved review conversations, and no force-push or deletion. The initial foundation was bootstrapped directly before protection was enabled. Maintainers should review the GitHub settings when adding new checks; this policy does not require a second maintainer to approve every PR.

## Keeping the README current

Every PR reviews the README status, implemented/support claims, setup/check commands, and links. If any change, update them in that PR and update its status date. Avoid time-based claims such as “coming soon.” Do not add installation instructions before a tested runnable path exists. CI checks structure, links, policy, and archive integrity; review must judge semantic freshness.

## Codex

Start a Codex task with a GitHub issue and a concrete outcome. [AGENTS.md](../AGENTS.md) supplies durable project guidance; a checked-in plan supplies continuation context for long work. No account-wide settings, credentials, or hard-coded local paths belong in this repository.

Workflow references, retrieved 2026-10-19: [official AGENTS.md guidance](https://learn.chatgpt.com/docs/agent-configuration/agents-md) and [OpenAI execution-plan guidance](https://developers.openai.com/cookbook/articles/codex_exec_plans). The project uses a small issue-driven adaptation, not a required external planning framework.

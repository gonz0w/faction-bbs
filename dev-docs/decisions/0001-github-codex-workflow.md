# 0001: Use a GitHub issue-driven Codex workflow

Status: accepted
Date: 2026-10-19
Owner: project founder
Issue: BBS-032 in the [issue index](../issue-index.md)
Supersedes: the earlier hidden planning command workflow

## Question and constraints

How should a public, research-stage project preserve intent and enable Codex work without relying on a separate planning framework? The founder explicitly requested GitHub issues, `docs/` and `dev-docs/`, maintained README status, and retirement of the old planning mode on 2026-10-19.

## Alternatives and decision

Keeping the hidden planning tree would preserve its conventions but retain competing backlog/state surfaces and slash-command dependencies. Deleting it would lose provenance. Adopt GitHub Issues for actionable work, Markdown product docs in `docs/`, engineering/research/decisions/plans in `dev-docs/`, root `AGENTS.md` for durable instructions, and focused issue-linked PRs. Preserve historical files byte-for-byte in a clearly non-authoritative archive with hashes and an issue mapping.

## Evidence and confidence

High confidence in product-owner authorization. Official guidance retrieved 2026-10-19 supports repository instructions and self-contained plans: [AGENTS.md](https://learn.chatgpt.com/docs/agent-configuration/agents-md), [execution plans](https://developers.openai.com/cookbook/articles/codex_exec_plans). The exact directory layout is a project choice.

## Tradeoffs and reversal cost

GitHub becomes the canonical hosted task tracker; the checked-in issue index and preserved source retain discovery/provenance, not live task-state replication. Maintenance requires updating README/docs with relevant PRs. Another tracker could later import issue data without changing product semantics. Archives retain outdated statements as historical context and must not govern current work.

## Validation

Verify every legacy candidate maps to exactly one GitHub issue, every source archive hash matches, current docs contain no active old planning references, internal links resolve, no JS/TS files/dependencies/actions are introduced, repository CI passes, and public repo/folder naming agree. Record results in [the foundation execution plan](../plans/0001-repository-foundation.md).

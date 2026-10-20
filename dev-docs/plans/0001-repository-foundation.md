# 0001: Establish the public Faction BBS foundation

Issue: BBS-032 in the [issue index](../issue-index.md)
Status: in progress

## Goal and authorization

The founder requested migration away from the earlier planning system, a public GitHub repository and issue backlog, a matching local project folder, `docs/` and `dev-docs/`, maintained README status, and Codex-friendly practices on 2026-10-19. Follow-up selected the Faction naming direction, BSD-3-Clause, and no JS/TS including browser dependencies.

## Progress

- [x] Read current planning, source documents, and all 20 captured candidates.
- [x] Preserve original source files in a hash-verified historical archive.
- [x] Write current product, security, session, workflow, decision, and contribution docs.
- [x] Prepare GitHub templates, issue specifications, and dependency-free repository checks.
- [x] Create/verify the public repository, 35 issues, 28 project labels, two milestones, and private security reporting.
- [ ] Initialize/push Git history and verify CI.
- [ ] Rename the local folder to match the repository.
- [ ] Complete checks, update outcomes, and close the foundation issue.

## Discoveries and decisions

- GitHub CLI authentication works outside the restricted network sandbox for the founder's `gonz0w` account; credentials are never written into project files.
- The old source includes duplicate generated maps and obsolete web-first/optional-terminal assumptions. Preserve them only as explicitly historical context.
- Browser console access conflicts with common JS-based implementations; preserve the requirement as an open feasibility question under the explicit no-JS rule.
- Privacy, secure terminal access, upload/input handling, local-data boundaries, and lightweight deployment need bounded research issues, not claims of existing support.
- [ADR 0001](../decisions/0001-github-codex-workflow.md) and [ADR 0002](../decisions/0002-project-identity-and-constraints.md) govern this migration.

## Validation and handoff

Run `python3 tools/check_repository.py` from the repository root. Verify archive hashes, all 20 legacy issue mappings, current-doc links, no active old planning directory, no JS/TS assets or JS-based actions, README stage accuracy, public repository identity, GitHub issue count/links, security reporting, clean Git status, and CI conclusion. Publication and local rename results are recorded here before completion.

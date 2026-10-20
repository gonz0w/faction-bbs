# 0001: Establish the public Faction BBS foundation

Issue: BBS-032 in the [issue index](../issue-index.md)
Status: completed

## Goal and authorization

The founder requested migration away from the earlier planning system, a public GitHub repository and issue backlog, a matching local project folder, `docs/` and `dev-docs/`, maintained README status, and Codex-friendly practices on 2026-10-19. Follow-up selected the Faction naming direction, BSD-3-Clause, and no JS/TS including browser dependencies.

## Progress

- [x] Read current planning, source documents, and all 20 captured candidates.
- [x] Preserve original source files in a hash-verified historical archive.
- [x] Write current product, security, session, workflow, decision, and contribution docs.
- [x] Prepare GitHub templates, issue specifications, and dependency-free repository checks.
- [x] Create/verify the public repository, 35 issues, 28 project labels, two milestones, and private security reporting.
- [x] Initialize/push Git history and verify the first GitHub CI run succeeds.
- [x] Rename the local folder to `faction-bbs`, matching the public repository.
- [x] Complete repository checks and six checker tests; record outcomes. Foundation issue closure follows the final verification.

## Discoveries and decisions

- GitHub CLI authentication works outside the restricted network sandbox for the founder's `gonz0w` account; credentials are never written into project files.
- The old source includes duplicate generated maps and obsolete web-first/optional-terminal assumptions. Preserve them only as explicitly historical context.
- Browser console access conflicts with common JS-based implementations; preserve the requirement as an open feasibility question under the explicit no-JS rule.
- Privacy, secure terminal access, upload/input handling, local-data boundaries, and lightweight deployment need bounded research issues, not claims of existing support.
- [ADR 0001](../decisions/0001-github-codex-workflow.md) and [ADR 0002](../decisions/0002-project-identity-and-constraints.md) govern this migration.

## Validation and handoff

Run `python3 tools/check_repository.py` from the repository root. Verify archive hashes, all 20 legacy issue mappings, current-doc links, no active old planning directory, no JS/TS assets or JS-based actions, README stage accuracy, public repository identity, GitHub issue count/links, security reporting, clean Git status, and CI conclusion. Publication and local rename results are recorded here before completion.

## Recorded outcome — 2026-10-19

- Public repository: [gonz0w/faction-bbs](https://github.com/gonz0w/faction-bbs), default branch `main`, BSD-3-Clause detected by GitHub.
- Created 35 issues with stable work IDs, acceptance criteria, and linked dependencies; all 20 legacy candidates map exactly once. Added 28 project labels and the Repository foundation / Architecture & validation milestones.
- Preserved 46 original non-OS source files with SHA-256 verification. Retired the active hidden planning directory. Current documentation is separated into `docs/` and `dev-docs/`.
- Enabled private vulnerability reporting, secret scanning, and push protection. Public commits use the maintainer's GitHub noreply address.
- `python3 tools/check_repository.py`: passed after migration and again from the renamed folder.
- `python3 -m unittest discover -s tools -p 'test_*.py'`: six passed, including intentional broken-link, altered-archive, JS asset/action, duplicate-mapping, and retired-directory rejection scenarios.
- Initial commit `eab9acb` passed [GitHub CI](https://github.com/gonz0w/faction-bbs/actions/runs/39347351705), using public Git fetch plus shell/Python and no marketplace actions.
- Final bootstrap policy requires PRs, the Documentation and policy check, and resolved conversations on `main`; force-push/deletion are disallowed. See the live GitHub branch settings for enforcement.
- Remaining work is architecture research in the linked issues. There is no BBS executable, supported deployment, selected production language, browser-console solution, game compatibility, or production security guarantee.

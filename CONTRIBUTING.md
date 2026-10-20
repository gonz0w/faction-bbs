# Contributing to Faction BBS

We are building the first local core for a private, terminal-first BBS alongside bounded research. Useful contributions include tested implementation increments, requirements and experiments that resolve named uncertainties.

1. Read [the product requirements](docs/project.md), [security/privacy requirements](docs/security-privacy.md), and [development workflow](dev-docs/workflow.md).
2. Find or open a GitHub issue. Check the [issue index](dev-docs/issue-index.md) for related work and dependencies. An open issue does not promise an initial-release feature.
3. Agree on the question, deliverable, and acceptance criteria. Use a focused branch such as `research/4-runtime-options` or `docs/21-session-flow`.
4. Record evidence and tradeoffs using the [research](dev-docs/research/template.md) or [decision](dev-docs/decisions/template.md) template. Use an [execution plan](dev-docs/plans/template.md) for substantial multi-step work.
5. Run `python3 tools/check_repository.py`, plus the relevant checks for any experiment. State what was actually validated and what remains unknown.
6. Open a PR linking the issue. Include the result, relevant validation, and documentation updates. Keep the README's stage and support statements accurate.

Rust/primary NetBSD direction and the first local SQLite/credential choices are recorded in the ADRs; a complete production stack remains open. See [local development](docs/local-development.md) for application build/run/tests. JavaScript/TypeScript, including third-party browser assets and JS-based tooling/actions, is prohibited. Python 3.10+ is used only for repository checks.

Keep documentation in plain Markdown. Use relative repository links, descriptive headings, readable prose, and named decision records. Product/member/sysop docs belong in `docs/`; engineering material belongs in `dev-docs/`. Historical archives are immutable and excluded from current design authority.

Do not include real private BBS addresses, credentials, user content, or third-party game/art files without explicit distribution permission. Security reports belong through [the private reporting process](SECURITY.md), not public bug reports. Contributions to this project are under its [BSD-3-Clause license](LICENSE); third-party material keeps its original terms.

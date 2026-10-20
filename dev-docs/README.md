# Development documentation

Faction BBS is establishing its architecture. There is no production stack or runnable product yet.

| Material | Purpose |
| --- | --- |
| [Workflow](workflow.md) | Issue -> bounded research/plan -> validation -> PR -> updated docs. |
| [Roadmap](roadmap.md) | Current research and future scope, without treating every issue as initial release work. |
| [Architecture](architecture.md) | Confirmed boundaries and unresolved technical decisions. |
| [Issue index](issue-index.md) | Stable work IDs, GitHub links, and migration traceability. |
| [Decision records](decisions/README.md) | Accepted/superseded decisions with evidence and validation. |
| [Research](research/README.md) | Bounded investigations and prototype criteria. |
| [Execution plans](plans/README.md) | Self-contained plans for substantial work. |
| [Migration](migration.md) | How the old planning system was retired. |
| [Archive](archive/README.md) | Immutable historical context, not current authority. |

Product/member/sysop documentation lives in [docs](../docs/README.md). Repository checks run with `python3 tools/check_repository.py` using Python 3.10+ and no third-party packages. Current Codex guidance is in [AGENTS.md](../AGENTS.md).

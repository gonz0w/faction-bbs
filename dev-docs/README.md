# Development documentation

Faction BBS has an early local Rust identity/room core and development console, including opt-in safe ANSI menus, declarative theme packs, BBS-local command chat and [private export/restore](../docs/export-restore.md). NetBSD remains primary; a restricted SSH development gateway now has native application evidence, while complete production resource/security/deployment work remains. See [local development](../docs/local-development.md) and [dependency inventory](dependencies/README.md).

| Material | Purpose |
| --- | --- |
| [Workflow](workflow.md) | Issue -> bounded research/plan -> validation -> PR -> updated docs. |
| [Roadmap](roadmap.md) | Current research and future scope, without treating every issue as initial release work. |
| [First local software slice](first-local-slice.md) | Concrete local core build sequence, VM checks and deferred features. |
| [Architecture](architecture.md) | Confirmed boundaries and unresolved technical decisions. |
| [Issue index](issue-index.md) | Stable work IDs, GitHub links, and migration traceability. |
| [Decision records](decisions/README.md) | Accepted/superseded decisions with evidence and validation. |
| [Research](research/README.md) | [Decision framework](research/decision-framework.md), bounded investigations, and prototype criteria. |
| [Execution plans](plans/README.md) | Self-contained plans for substantial work. |
| [Migration](migration.md) | How the old planning system was retired. |
| [Archive](archive/README.md) | Immutable historical context, not current authority. |

Product/member/sysop documentation lives in [docs](../docs/README.md). Repository checks run with `python3 tools/check_repository.py` using Python 3.10+ and no third-party packages. Current Codex guidance is in [AGENTS.md](../AGENTS.md).

[Versioned executed sources](evidence/README.md) preserve historical native measurements as current code evolves.

Current confinement work: [plan 0020](plans/0020-process-confinement.md),
[research 0034](research/0034-process-confinement.md) and
[ADR 0020](decisions/0020-native-process-envelope.md).

# Development documentation

[GitHub Issues](https://github.com/gonz0w/faction-bbs/issues) exclusively own work,
plans, dependencies, tests, progress and handoffs. Start with a finite ready issue,
then follow [the issue-only TDD workflow](workflow.md). No execution-plan directory,
roadmap, local TODO journal or agent status log replaces GitHub.

| Material | Purpose |
| --- | --- |
| [Engineering policy](../PROJECT.md) | Pinned method, selected profiles, lasting choices and stronger Faction contracts. |
| [Workflow](workflow.md) | Issue → acceptance tests → red/green/refactor → closing PR → verified issue closure. |
| [Architecture](architecture.md) | Current component and authority contracts, not work status. |
| [Decisions](decisions/README.md) | Accepted/superseded technical decisions and their rationale. |
| [Research](research/README.md) | Dated investigations and experimental observations. |
| [Dependencies](dependencies/README.md) | Reviewed source/license/version graphs and checks. |
| [Extension contracts](extension-contracts.md) | Current file/identity/mail/door boundaries. |
| [Validation](validation/README.md) | Named software product observations. |
| [Executed evidence](evidence/README.md) | Immutable measured bytes/results and their verification. |
| [Migration](migration.md) | Why local task tracking was retired. |
| [Historical archive](archive/README.md) | Non-authoritative provenance, never agent work instructions. |

Member/sysop/product docs live in [docs](../docs/README.md). The current runnable
software and support limits are in [README](../README.md), the
[NetBSD quickstart](../docs/quickstart-netbsd.md) and feature docs. Root
[AGENTS.md](../AGENTS.md) contains concise durable instructions, not a backlog.

[Native home-mail contract](native-home-mail.md) describes the optional direct-TLS transport, verified device codec and checkpoint boundary.

- [Native archive cursor and migration contract](native-mail-archive.md)

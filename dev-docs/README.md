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

Current confinement work: [plan 0021](plans/0021-os-isolation.md),
[research 0035](research/0035-os-isolation.md) and
[ADR 0021](decisions/0021-experimental-os-isolation.md). The prior
[process envelope](../docs/process-confinement.md) remains in both profiles.

SSH admission continuation: [plan 0022](plans/0022-ssh-admission.md),
[research 0036](research/0036-ssh-admission.md), [ADR 0022](decisions/0022-ssh-admission.md).

- [Sustained 250-member activity and bounded abuse](research/0037-sustained-abuse.md):
  longer active intervals, repeat key-query work and native source-fairness gates.

- [Bounded SSH key-lookup research](research/0038-bounded-key-lookups.md), [ADR 0023](decisions/0023-bounded-key-lookups.md) and [plan 0024](plans/0024-bounded-key-lookups.md): native finite-work budget and two whole-host controls.

- [Original classic-game reuse](research/0042-original-classic-games.md): native LORD proof, [compatibility patch](compatibility/README.md) and separate game-host limits.

- [Scoped extension contracts](extension-contracts.md) — file/identity/mail/door boundaries and selected next increments.

The optional [native client](../docs/native-client.md) integrates Files transfers and local room/chat references; [ADR 0033](decisions/0033-native-terminal-files.md) records its boundary. Mail/identity/attachments remain #84/#88 research.


[Modern owner stories](../docs/modern-sysop-and-communications.md) and [research 0047](research/0047-modern-terminal-and-federation-options.md) define #95–#101; [article/news stories](../docs/discussion-articles-and-news.md)
extend them with #103/#104 without claiming implementation.

[Personal catch-up](../docs/catchup.md): [plan 0066](plans/0066-personal-catchup.md),
[ADR 0061](decisions/0061-personal-catchup.md) and
[software checks](evidence/catchup-netbsd/README.md).

[Discussion discovery](../docs/discussion-search.md):
[plan 0067](plans/0067-discussion-search-and-bookmarks.md),
[ADR 0062](decisions/0062-discussion-search-and-bookmarks.md),
[software checks](evidence/discussion-netbsd/README.md).

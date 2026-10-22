# Architecture and project decision records

Use `NNNN-short-kebab-title.md` and the [template](template.md). Status is proposed, accepted, rejected, or superseded. Preserve earlier decisions when superseding them. Accepted product/workflow choices are not proof of technical feasibility.

- [0001: GitHub/Codex repository workflow](0001-github-codex-workflow.md)
- [0002: Identity, license, and implementation constraints](0002-project-identity-and-constraints.md)
- [0003: Architecture decision framework — accepted](0003-architecture-decision-framework.md)
- [0004: SSH and optional private-network operation — accepted product direction](0004-ssh-and-optional-private-network.md)
- [0005: Rust/readiness + SQLite validation slice — integrated gates remain; language superseded](0005-rust-sqlite-validation-slice.md)
- [0006: Rust and primary NetBSD — accepted owner direction](0006-rust-and-primary-netbsd.md)

- [0007: Software first, core first, Lua next — accepted sequencing](0007-software-first-and-extension-direction.md)

- [0008: First local core, durable room actions and credentials](0008-local-core-and-credentials.md)

- [0009: Bounded ANSI presentation, dual navigation and declarative themes](0009-safe-terminal-menus-and-themes.md)

- [0010: Bounded BBS-local chat, grants, history and schema migration](0010-bounded-local-chat.md)

- [0011: Private current-format export and fresh-directory restore](0011-private-export-restore.md)

- [0012: Restricted OpenSSH and separate-UID terminal handoff](0012-restricted-ssh-and-terminal-handoff.md)

- [0013: Managed gateway lifecycle](0013-managed-gateway-lifecycle.md)

Use the [decision framework](../research/decision-framework.md) for gate screening, criteria, evidence/confidence, and prototype thresholds. Future technical decisions need primary-source research, retrieval dates, alternatives, confidence, reversal cost, and a named validation experiment. The question and evidence must be bounded.

- [0014: Credential budget and guarded scaling experiments](0014-credential-budget-and-scaling-experiments.md)

- [0015: Shared board/readiness experiment; production adoption deferred](0015-shared-board-readiness-experiment.md)

- [0016: Native allocator comparison](0016-native-allocator-comparison.md)

- [0017: Repeatable headroom experiment](0017-repeatable-headroom.md)

- [0018: Bounded shared UI candidate](0018-shared-ui-themes.md)

- [0019: Maintained shared service and conservative admission](0019-maintained-shared-service.md)

- [0020: Native process envelope and authenticated terminal peers](0020-native-process-envelope.md)

- [0022 Experimental SSH admission](0022-ssh-admission.md) — bounded transport startup; production adoption deferred.

- [0023: Fixed SSH helper lookup allowance](0023-bounded-key-lookups.md) — explicit v2 migration and finite native lookup work.

- [0024: Listener-scoped global SSH gate](0024-global-ssh-admission.md) — unchanged penalties and separate private-identity exploration.

- [0026: Product-first small-board setup](0026-product-first-small-board.md) — ordinary eight-session preview and unprivileged boot detachment.

- [0027: Separate-UID Lua arcade](0027-lua-arcade-boundary.md) — optional menu/game turns without backend execution privileges.

- [0028: Local arcade bests under host-owned run authority](0028-local-arcade-scores.md)

- [0029: Fixed classic-server door](0029-classic-server-door.md) — original remote game execution with bounded local parsing.

- [0030: File areas and transfer adapters](0030-file-areas-and-transfer-adapters.md); [0031: Home identities and client-encrypted mail](0031-home-identities-and-client-encrypted-mail.md).

- [0032: Scoped single-object SFTP](0032-single-object-sftp.md) — OpenSSH transport, separate identities and bounded local policy.

- [0033: Native terminal transfers and local file references](0033-native-terminal-files.md).

- [0034: Encrypted-mail reuse and roaming-admission proof](0034-encrypted-mail-reuse-proof.md) — accepted experiment boundary; complete native adoption remains open.

- [0035: Native local mail vault](0035-native-local-mail-vault.md) — member-owned storage/unlock; native network mail remains open.

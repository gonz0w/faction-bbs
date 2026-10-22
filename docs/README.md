# Product, member, and sysop documentation

These describe current requirements, the local console and an early restricted SSH development gateway. There is no supported network release.

- [Lua arcade: menus, original games, setup and boundary](lua-doors.md)
- [Run a small NetBSD board](quickstart-netbsd.md) — first install, approved SSH, ANSI rooms/chat and backup.
- [Build and try the local core](local-development.md)
- [Restricted SSH member access and owner setup](restricted-ssh.md)
- [Private export, verify and restore rehearsal](export-restore.md)
- [Project direction and boundaries](project.md)
- [Login, approval, and BBS session](session.md)
- [ANSI, menus, and customization](customization.md)
- [Local chat commands, private channels and history](local-chat.md)
- [Run/configure ANSI menus and theme packs](theme-packs.md)
- [Arrow-key menus, hotkeys, sysop themes and seasonal identity](menus-and-themes.md)
- [Privacy and security requirements](security-privacy.md)
- [Optional modern extensions, public rooms, shops and games](extensions.md)

Development practices, experiments, and architecture decisions belong in [dev-docs](../dev-docs/README.md). Historical draft inputs are in the [archive](../dev-docs/archive/README.md).

- [NetBSD service lifecycle](service-lifecycle.md): managed development gateway start/stop/restart and crash recovery.

- [Shared UI candidate](shared-ui.md) — experimental shared sessions and sysop themes.

- [Maintained shared terminal backend](shared-service.md) — Rust owner, native terminal entry point and worker rollback.

- [Experimental NetBSD filesystem/exec and IP policy](experimental-isolation.md) — explicit privileged profile, narrow evidence and recovery.

- [Experimental SSH admission](experimental-admission.md) — all-transport cap, global launch budget and explicit crash recovery.

- [Experimental global SSH admission](experimental-global-admission.md) — fixed listener gate and private-source research; not production adoption.

- [Classic game-server doors](classic-game-doors.md) — optional configured terminal connections and return.

- [Original LORD on NetBSD](original-lord.md) — unchanged original DOS game, private software proof and limits.

- [File areas and restricted SFTP](file-areas.md) and [native-client encrypted mail/home identities](private-mail.md).

- [Later-phase hardening](later-phase-hardening.md) — signing, hardened VM, containment, storage and independent verification backlog.
- [Room reading](room-reading.md) — wrapped chronological messages, reply context, ANSI transitions and complete-message unread behavior.
- [Discussion articles and news connections](discussion-articles-and-news.md) — planned Netnews room articles and optional NNTP peers/providers; IRC/XMPP remain separate.
- [Modern sysop console and communications stories](modern-sysop-and-communications.md) — planned local dashboard, capability-aware UI, host operations and protocol integration.

- [Persistent post metadata](post-metadata.md) — schema-6 subject/date/ID semantics and stopped-service upgrade/rollback.

- [Authored subjects and local room articles](room-articles.md)

- [Article profile v2 and original INN validation](article-profile.md)

- [Original Nightwire ANSI artwork and login screens](artwork-nightwire.md).

- [Nightwire terminal compatibility](artwork-terminal-compatibility.md): real-emulator cell placement, viewport requirements and classic recovery.

- [Local sysop console](sysop-console.md): OS-authorized read-only BBS status, live CPU/RAM/network graphs, serial-login validation and classic/ANSI operator panels.

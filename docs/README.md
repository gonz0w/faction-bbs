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

- [Planned file areas and SFTP](file-areas.md) and [native-client encrypted mail/home identities](private-mail.md).

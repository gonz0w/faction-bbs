# A board with its own scene

Status: broader design requirements with an implemented local [TOML theme/menu baseline](theme-packs.md); scripting, scheduler, full layout/art compatibility and network deployment remain open.

Faction BBS should give sysops room to create a recognizable place: ANSI identity art, ASCII/1337 typography, colors, prompts, menus, quotes, announcements, file areas, rooms, chat, and a selection of doors. The founder's references include WWIV session flow, Citadel room interaction, Vision-X, Vision/2, Oblivion, and the personal BBS modifications they remember writing and using.

Customization must have explicit permissions, lifecycle, upgrade, and failure-isolation boundaries. Keep simple configuration/declarative layouts as the core baseline. The owner chose Lua menus and isolated game doors as the first extension experiment after the local core. The [boundary research](../dev-docs/research/0010-extensible-ansi-platform.md) includes a synthetic Rust/Lua worker probe, not a production sandbox or finalized API. No JavaScript/TypeScript is permitted. A visual theme must not be able to bypass approval, expose files, share a local channel, or execute unrestricted host commands.

ANSI compatibility is a capability to specify and test. Study character encodings, CP437-style art where appropriate, terminal dimensions, line endings, key handling, control-sequence safety, and a readable plain-text fallback. Large artwork should not destroy the low-bandwidth baseline.

Use original or explicitly distributable assets. Historical game/software names are references; existing artwork and game executables are not automatically available for redistribution. Keep per-title/per-asset provenance and terms when introducing third-party material.

The owner confirms **Oblivion/2-inspired arrow-key highlighting plus direct hotkeys** (for example `[D]oors` -> `D`), first-class sysop-defined themes and optional seasonal colors/login identity screens. [The menu/theme direction](menus-and-themes.md) specifies the intended behavior; the [attributed reference catalog](../dev-docs/research/0025-menu-and-art-direction.md) records ACiD/iCE/Razor-related works actually examined. These are visual targets, not permission to bundle historical art or implemented menu support.

The initial menu inventory is in [the session guide](session.md). The extension and terminal questions live in [the GitHub issue](https://github.com/gonz0w/faction-bbs/issues).

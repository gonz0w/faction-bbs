# 0076: A read-only guide outside existing menu hotkeys

Status: accepted bounded implementation
Date: 2026-10-23
Issues: [#21](https://github.com/gonz0w/faction-bbs/issues/21), [#133](https://github.com/gonz0w/faction-bbs/issues/133)

## Decision

Expose a themed guide at the approved main menu through F1 on ANSI terminals and
`help` on plain terminals. Preserve `?` refresh and every existing action key/grid
position. Add no new action slot. Decode only SS3 P and CSI 11~ as F1; unknown or
incomplete sequences retain existing quarantine/recovery behavior. F1 is inert in
unrelated menu/door handlers and does not open a guide over active drafts/games.

Use one canonical bounded reference with current menu keys/labels, enabled-topic
filtering and the current sysop role. Recheck approval and rebuild live feature
gates on every guide show/action. A removed topic returns to the topic list.
The guide performs no content reads, posts, unread advancement, chat joins,
transfers, host action or native store unlock. Retain the existing connection,
visit lease/heartbeat, Menu activity and output limits. No schema/dependency/helper.

Sysops may customize guide_title (24 ASCII bytes) and guide_notes (1024 bytes,
16 lines, printable ASCII plus LF) in base/season packs. Never allow raw terminal
controls or execution/template authority. The guide palette follows the theme;
trusted device screens stay independent. House notes are visible to approved
members, so contain no keys, invitation secrets or private account data.

## Alternatives and consequences

A new menu entry could conflict with crowded/custom layouts and existing hotkeys.
Repurposing `?` would break explicit refresh/idle-notification behavior. F1 plus a
plain word adds useful discoverability without either change. Read-only references
keep the return path clear; action shortcuts from inside help would need another
review of current input/authorization/draft ownership and are deferred.

Menu preview references change only for the F1 hint. Keep older executed bytes and
records immutable before editing them. Validate real local/worker/shared SSH and
native/compact routes, current gates/role, resize and no content side effects.
No new capacity, hardware, federation or hardening claim.

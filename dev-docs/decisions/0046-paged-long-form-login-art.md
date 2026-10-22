# 0046: Bounded long-form login art

Date: 2026-10-22. Status: accepted implementation direction for #25; extends 0045.

## Context and alternatives

The owner asks for long ANSI art and login paging with classic fallback. Raising
the short import limit silently would break its explicit 80x16 contract. Streaming
raw art would couple paging to untrusted cursor/control state and slow output.
An external viewer adds a process/format boundary. Choose an explicit long field
and keep the existing validated canonical cell model instead.

## Decision

Optional `identity_ansi_long` selects <=80x256 cells / 64 KiB total input, mutually
exclusive with short `identity_ansi`. Metadata dimensions fit the chosen profile;
all existing CP437/SAUCE/iCE and body-control/provenance rules apply. Store only
actual/declared extent after bounded startup parsing. Validate every 16-row page
against the 16 KiB output budget before any listener binds. Render one page on
request; never concatenate a long identity for a member. The shared catalog owns
the cells; session state holds only a page index.

Approved ANSI >=80x24 sessions use Enter-next / S+Enter-skip. Final art and skip
both lead to the normal welcome/Enter/menu journey; skip uses ASCII fallback.
Compact/plain remain unpaged. Shared commands and drained-boundary resize
recheck approval; local/worker recheck at input/page boundaries. New pages reset
the ordinary input timeout. Keep one command/response, typeahead flushing,
32 KiB shared output/five-second drain and normal admission. No new dependency,
helper, database schema or host setting. Art remains pinned with the login theme.

The 256-row cap includes the 206-row reference studied under research 0025 while
keeping startup cells bounded. Sixteen-row pages preserve the existing 24-row
login composition/output envelope instead of introducing variable-size page
budgets. Larger/animated assets and final original/licensed artistic acceptance
stay open. Original sample material is clearly Codex-assisted template linework.

## Evidence and reversal

The existing [SAUCE revision 00.5](https://www.acid.org/info/sauce/sauce.htm) reference
(retrieved 2026-10-22 in ADR 0045) defines metadata; this is Faction's narrower
presentation profile, not a new interpretation of file rights or terminal fonts.
See [plan 0049](../plans/0049-paged-login-art.md) for executed software validation.
No hardware, broader emulator/font, RSS or new 250-user support claim follows.
Revert to a short/base pack and restart after preflight; no data migration.

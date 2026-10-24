# 0081: A useful board guide and command reference

Issues: [#21](https://github.com/gonz0w/faction-bbs/issues/21), [#133](https://github.com/gonz0w/faction-bbs/issues/133)
Status: implemented and software-validated; prepared for PR review

## Batch

Give approved members a themed guide with actual current menu keys, room and
writing guidance, group/social command references, Files transfer expectations
and a deliberate return to the main menu. ANSI main-menu F1 and plain `help`
enter it; preserve `?` as refresh and every current menu key/grid position. Sysops
can customize guide title and bounded inert house notes, including season packs.
No new menu action slot, listener, DB/schema, dependency, connection or owner helper.

Show only enabled current presentation topics and current sysop permissions. No
private room/channel names, metadata/counts, mail badges or automatic member
lookup. Guides never invoke commands, post content, mark unread, transfer files
or unlock device stores; navigation is read-only. Current approval is checked
on every show/action and no guide output interrupts drafts/doors/input.
Maintain one existing lease/heartbeat and broad Menu activity. Bound pages/output
and use shared canonical text for ANSI/plain/local/worker/shared/native routes.
Unknown/partial F1 sequences retain quarantine semantics; only SS3 P and CSI 11~
are accepted. Resize recovers to a working guide/menu without losing authority.

## Progress

- [x] Preserve snapshot 48: all 335 passport inputs and its unchanged whole result.
- [x] Implement canonical guide, theme fields, decoder and adapters.
- [x] Validate enabled-module/current-key filtering, sysop visibility and real PTYs.
- [x] Merge checked #179; join verified reviewed main without history rewriting.
- [x] Record exact native software/evidence, clean owned clone/cache and keep saved playground off.
- [x] Prepare focused PR and offline guard; retain checkout/archive checks before publication and keep #21/#133 open.

## Results and continuation

Merged #179 after its exact-head required check passed with resolved review
conversations; main squash 4a21cba has the same starting tree as e53c1c0. Join that
reviewed baseline only after asserting tree equality; no history rewriting.

The canonical guide is shared across local/inherited worker, the shared owner,
restricted SSH, host-native and guest-native routes. F1 is inactive outside main
menu/guide handling; `?` still refreshes. Current theme/menu gates and current
local approval/role are rechecked before every guide show/action. Navigation holds
only a topic/page/focus cursor, uses one existing visit heartbeat, and reports Menu
activity. Guide title/notes support the existing base/season rules; notes are inert.

369 workspace Rust tests pass on macOS and native NetBSD 11 aarch64; strict host
workspace Clippy and 130 Python tests pass. The maintained host Rust/terminal
workflow passes, including recovery, existing community desks, graphics filtering,
artwork, credential/admission boundaries and classic/inherited terminal flows.
The new guide has 17 real local/worker ANSI/plain controls per OS and 10 actual
shared SSH/host-native/guest-native controls. These cover current custom keys and
roles, feature-disable recovery, paging, resize, plain help and revoked-guide
navigation. The native regression suite retains Who/availability, sustained
standalone heartbeats, optional idle notifications, writing, rooms, community,
travel/passport, vault, custom-menu and export/restore paths, plus 18 art previews.

Validation corrections were limited to test/reference setup: F1 is now a valid
sequence, so the old unknown-sequence test uses F3; the raw-screen oracle ignores
carriage returns instead of counting them as blank lines; plain worker fixtures
explicitly set TERM=dumb; Wall Disable uses D; shared settings return directly to
main; the shared theme belongs to the service owner; revocation is tested on the
next guide action. Updated menu SVGs include the new F1 hint, including Nightwire.
The runtime emits CRLF for the raw shared TTY and bounds header/body rows at 40x18.

Snapshot 48 preserves 335 exact prior passport inputs and its entire result;
all 61 historical result identities remain fixed. Guard 62 checks the new software
record, 340 exact guest input matches and 14 selected harness identities, normal
eight-session/5000-ms admission and unchanged crypto, helper/schema/output limits.
This is software evidence on a 2-core/1-GiB VM, with no new capacity, physical SBC,
GUI terminal acceptance, hardening, federation or private-mail claim.

Owned NetBSD clone, private member profiles/keys/transcripts and vendor cache are
removed after verified shutdown/listener closure. Saved playground disk metadata
and login/key/configuration hashes are unchanged; its VM remains off. All current
record teardown flags were set only after those checks passed.

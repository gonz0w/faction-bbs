# 0050: Curated local Other BBSes directory

Date: 2026-10-22. Status: selected; software validated. #27/#21.

## Decision and alternatives

Ship an optional host-curated immutable pack and shared bounded connection-card
presenter. This gives approved members community discovery without waiting for a
new persistent submission/moderation schema or a cross-board identity protocol.
The existing terminal crate already supplies reviewed TOML, safe rendering and
navigation; the separate directory module owns no domain authority or connector.

A mutable DB directory/submission queue is useful follow-on work but requires
moderation, migration/export and abuse policy. A public/central registry changes
privacy/discovery requirements. Automatic SSH/native profile handoff needs local
host-key/identity decisions and must not run from the BBS owner. Defer those paths.
Optional browser handoff stays no-JS/private-or-disabled research, not a blocker.

## Bounds and authority

32 listings / 32 KiB, exact host-owned regular file, inert ASCII fields, validated
host/login/port/date/fingerprint tokens. One startup catalog/Arc for shared owner;
per-client pointer and view cursor only. Existing local approval/epoch gates apply
before every screen/action. New theme capability is enabled by host configuration,
never by a member or a label. No new listener/helper/dependency/schema or default.

A connection card is information. It creates no remote contact, authentication,
host-key trust, account, approval or federation. Editorial dates distinguish OLD
and DATE?; they do not measure reachability. In-BBS submissions follow later;
operator-provided details require explicit local review/consent and stopped-restart
removal now. Catalog data is not copied into board exports or shared room/mail data.

## Validation and reversal

[GitHub issue](https://github.com/gonz0w/faction-bbs/issues/27) covers parser/file bounds,
full-grid fallback, actual local/worker and native shared SSH/client routes,
approval/revocation, compact/plain/resize and a no-connect trap. Measure enabled
maximum-catalog memory separately from default-disabled console regressions.
Prototype evidence remains scoped to the named NetBSD software VM, not all ports,
physical SBCs, supported 250-user capacity, visual certification or hardening.

Disable the option and restart to reverse; no database/backup-format migration or
network teardown is needed. Keep approved packs as separate installation config.
The old standalone experiment needs only a disabled Contacts enum-compatibility
arm; its prior measured bytes/results remain frozen and are not relabeled.

[Actual NetBSD software results](../evidence/directory-netbsd/README.md) pass the
planned first-slice controls; later native handoff/member-submission work remains
open. No new capacity/remote-login/federation claim follows from these tests.

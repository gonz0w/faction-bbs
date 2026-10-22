# 0038: Room article foundation and a readable terminal journey

Issues: #103 and #21/#25. Status: implemented and validated, 2026-10-21.

## Goal and constraints

Deliver a dependency-free Rust Netnews text-article codec and a safe shared
room-message presenter used by the local/worker and shared-service interfaces.
Clear ANSI sections, wrap at actual terminal width and page long messages without
marking unread content read. Preserve chronological rooms, local approval,
permission-checked file links and owner/resource defaults. No wire listener,
publication, database migration, original-date fabrication or IRC/XMPP conversion.

This branch starts on docs PR #102; native vault PR #94 stays separate. Reuse
its exact pre90 source snapshot and source-resolution helpers only, after matching
all 89 original executed inputs and the original native-client result hash. This
freezes history before changing common UI/workspace files; no vault code imported.

## Progress

- [x] Read requirements, issue contracts, current UI and source evidence guards.
- [x] Preserve historical measured source bytes before edits.
- [x] Implement bounded article profile and offline inspect/round-trip example.
- [x] Implement shared safe wrapped/paged room presentation and screen transitions.
- [x] Validate Rust, actual terminal journeys, dependencies and historical guards.
- [x] Update docs/README/changelog; prepare the focused stacked PR.

## Decisions and validation

Use existing article specifications linked in research 0047. The initial supported
profile is deliberately text-only, bounded and explicit about rejected MIME/header
features. Add no external dependencies. #103 remains open for schema/migration,
post integration and independent native NetBSD article validation. Network peering
and provider work remain #101/#104. Current legacy posts have no original date or
subject; the UI must not invent either.

Run cargo fmt/clippy/test/build, repository and source/dependency guards, real
local/worker terminal flows, and a focused shared-service room reading fixture.
Keep the preserved playground stopped; software fixtures must use disposable
state/ports and be removed after checks. Do not present old 250-user evidence as
measurements of this new presenter.

## Actual outcomes

- 114 Rust workspace tests pass on macOS ARM64 and native NetBSD 11 ARM64; build,
  formatting and host clippy pass. No new external dependencies; reviewed Linux/
  NetBSD no-JS selected graphs are unchanged.
- Real local narrow plain/ANSI reader tests pass on both systems. Existing local
  console/approval/ANSI tests pass natively and on the host; host worker/chat and
  export/restore flows also pass.
- Five real native shared controls pass via a root-orchestrated UID-owned PTY:
  approved frontend access, partial unread retention, exact wrapped output/order,
  drain-before-complete acknowledgment and revocation before the next page. This
  does not retest dedicated SSH admission or complete SFTP/capacity.
- The initial renderer split file-reference tokens. Focused coverage now preserves
  complete tokens, including prefixes crossing a wrap boundary, and charges their
  soft-wrap row cost. No per-client history cache or extra owner thread is added.
- Fixture corrections: create the owner-only board directory; provide the root
  command PATH; create 40x18 SSH PTYs at launch rather than race a later resize.
  Early VM serial attachment needed wait-on. Repository-test copies now exclude
  ignored local/ playground artifacts and mock the new source manifest.
- The private VM clone was shut down and removed with all test credentials/data;
  the owner's original stopped playground and standalone checkpoint remain intact.
  The current record pins 95 executed-source inputs; seven historical snapshots
  and nineteen old result identities remain unchanged.

New evidence: dev-docs/evidence/room-reader-netbsd/results.json and its guard.
Runtime/database migration, broader MIME/header compatibility, identity/date
creation, duplicate-ID policy, post integration and independent news interop remain
#103/#101/#104. #21/#25 remain open for the rest of the complete screen journey.
This branch stacks on documentation PR #102 and leaves native vault PR #94
independent. No issue is closed by this partial implementation batch.

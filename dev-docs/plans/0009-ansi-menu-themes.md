# 0009: Safe ANSI menu and sysop themes

Date: 2026-10-19. Status: published for review in [PR #49](https://github.com/gonz0w/faction-bbs/pull/49). Issues: #13/#21/#25/#29/#31/#35.
Owner authorized merge of PR #48 and continued implementation. #48 merged as
c7abd006896e5389dac06feaf1c3d1920c400afc with required checks passing.

## Scope

Add a separate terminal presentation crate: bounded streaming key decoding,
arrows/Enter and direct hotkeys reaching identical action IDs, non-color focus,
incremental redraw, plain fallback and resize recovery. Integrate with the existing
local console only; no listener/SSH/chat/door implementation implied.

Use versioned, strict TOML sysop packs with declarative palettes/labels/order,
identity art/welcome/logout/ASCII surfaces and provenance. Manual pack choice can
select seasonal variants for next login; no calendar scheduler or hot reload.
Validate a whole pack before replacing the built-in text fallback. Do not bundle
historical art or fabricate a generic block identity screen. ANSI import is a
named bounded subset, converted to safe cells; document unsupported formats.

## Predeclared failure cases

- Split/coalesced CSI/SS3, CRLF once, bare Escape timeout, malformed/oversized
  sequences and OSC/DCS payloads cannot become hotkeys. No menu interpretation in
  editors; flush pending input at mode transitions. Keep buffers bounded.
- Same enabled action via arrows/Enter and either-case hotkey; movement performs
  no action; sysop/disabled targets cannot activate through themes or either path.
- Focus old/new redraw stays bounded, without replaying identity artwork. Resize
  abandons ANSI geometry safely; plain fallback remains usable.
- Theme versions/unknown keys/collisions/unknown actions/missing or malicious assets
  fail atomically. File reads are bounded, regular local files; no path traversal,
  symlinks or executable theme logic. Manual seasonal packs preserve action checks.
- Art allowlist rejects OSC/clipboard/device/window controls, invalid SGR,
  out-of-canvas cursor movement and oversized inputs/output. Preserve CP437 cell
  intent for the documented subset; no raw escapes from members or sysop files.
- Actual local PTYs cover identity-before-menu, arrows and hotkeys, fallback,
  resize, pending denial, revocation and terminal cleanup. Existing core tests stay.

## Work / evidence

- [x] Merge #48 and read current product/security/menu/issue requirements.
- [x] Implement parser, pack validation, cell renderer and regression tests.
- [x] Integrate the local console and extend the disposable PTY smoke.
- [x] Review dependency changes, record ADR/docs/README/changelog and CI evidence.
- [x] Publish and attach PR #49; broad issues remain open. The PR carries the authoritative CI result.

Next native NetBSD/application and restricted SSH work remains explicit. No physical
hardware is required. No implicit merge of the next PR.

## Results and handoff

- Rust 1.96.0 / ARM64 Darwin: 28 tests (14 core, two CLI, 12 presentation)
  passed. Strict Clippy and rustfmt passed. The disposable PTY workflow covers
  both input routes, pre-approval identity denial, identity/menu ordering, editor
  context, hostile input, timeout, manual season, resize, invalid-art fallback,
  revoke/reapprove and normal echo/cursor restoration. No real artwork included.
- Renderer observation: 668-byte built-in full menu; first old/new focus redraw
  137 bytes. This is a deterministic byte result, not a server benchmark.
- Pinned TOML added ten selected packages; current NetBSD/Linux inventories are
  39/41 with declared MIT/Apache-compatible terms, no prohibited JS/TS filenames
  or manifests. Some unselected platform/derive dependencies remain in Cargo.lock.
- Version 1 is deliberately local/development-only: vertical menus, manual next-
  login seasonal overrides, fixed 80x16 CP437/CSI import, UTF-8 terminal output.
  Long art/SAUCE/iCE metadata/grids/calendar/fuzz breadth and artist review of
  actual assets remain #13/#25/#29/#31. No full archived-art compatibility claim.
- Linux CI on the linked PR must repeat all checks. Native NetBSD testing of the
  application remains pending; physical hardware is not required. The local CLI
  remains owner-capable and must never be used as an SSH forced command.
- Next useful core work remains account/room lifecycle and export/restore, local
  chat, a distinct restricted SSH adapter, native NetBSD PTYs and whole-process
  load/recovery evidence. Lua follows the useful local core, per owner direction.

# ADR 0009: Bounded terminal presentation and declarative themes

Status: accepted for the local development increment. Date: 2026-10-19.
Issues: #13/#21/#25/#29/#31/#35. Owner authorized merge #48 and continued building.
Scope: reusable terminal presentation plus local console integration; no network
adapter or production terminal-compatibility/security claim.

## Question, constraints and alternatives

How do real arrow/hotkey menus and sysop-owned identity/themes work without raw
terminal injection, executable configuration, new transport authority or JS?
Owner requires Oblivion/2-inspired dual navigation, visible focus, safe ANSI,
plain fallback, season-ready customization and approved access. Physical hardware
is not required; do not manufacture identity artwork to claim visual quality.

| Choice | Alternatives | Decision / cost |
| --- | --- | --- |
| UI surface | Framework/terminfo layer; existing termios wrappers plus small cell/menu renderer | Reuse rustix safe wrappers and a separate pure Rust presentation crate. Fewer moving runtime parts for this bounded vertical menu; broad terminal detection, grids and richer editing remain future work. No comparative performance or framework incompatibility claim. |
| Theme configuration | Hardcoded palette; bespoke parser; JSON; TOML; executable Lua | Pin TOML 1.1.8+spec-1.1.0 with std/parse/serde and no default features. Standard editable config instead of bespoke grammar, with strict bounded application schema. Adds ten selected registry packages; Lua remains separate isolated extension work. |
| Art handling | Raw passthrough; stripping escapes; trusted subset parsed into cells | Import a documented 80x16 CP437/CSI subset into fixed cells, then emit canonical controls/glyphs. Preserves supported color/cursor intent while rejecting unsupported operations atomically; many historical/long/SAUCE/iCE works need later format support. |
| Input recovery | Treat unknown bytes as hotkeys; unbounded sequence buffering; bounded state machine | CSI/SS3 arrows, CRLF once, casefolded hotkeys; 16-byte parameter buffer, 300-ms completion deadline, quarantine to newline for unknown sequences. Control strings drain to their terminator, never leak payload letters; Ctrl-C/D exits. Unknown keys can require an extra Enter instead of silently triggering an action. |
| Seasonal identity | Scheduler now; manual whole-pack changes; constrained override | Manual next-login override of palette/welcome/art only, retaining the base command map. Two example overrides prove the seam; timezone/calendar/overlap/live switching deliberately remain open. |

The `faction-terminal` crate defines validated theme objects, cells, key events,
and action IDs. It has no account/session/store/network authority. The local CLI
maps both input routes to the same existing core-checked handlers. Sysop action
permission is compiled; a `doors` entry is always disabled until real doors exist.
Theme field values cannot name owner recovery or arbitrary executable commands.

## Trust boundaries, limits and failure behavior

- Own Rust remains unsafe-forbidden; upstream/native code is not an all-safe-code
  guarantee. No JS/TS source/manifests found in the selected Linux/NetBSD source
  graphs. [Inventory](../dependencies/README.md) records versions/licenses/features.
- Themes/assets are effective-UID-owned regular local files, not symlinks or
  group/world writable. Reads have explicit 8-KiB config / 2-KiB ASCII / 16-KiB ANSI
  caps. Trusted parents/same-UID operator processes remain assumptions. No hostile
  filesystem race/sandbox claim. The whole candidate validates before activation.
- Theme version 1 fixes label/dimension/encoding bounds, semantic palettes, action
  IDs and provenance requirements. Invalid base -> built-in text fallback; invalid
  season -> unchanged base. No scripts/hot reload/member-controlled config.
- No raw sysop/member escapes pass through. Art controls build cells; generated
  output is allowlisted. SGR blink, OSC/clipboard/window/device operations and
  out-of-canvas cursor moves fail; member room text stays escaped by the CLI.
- ANSI is explicit local opt-in, with TTY/TERM/geometry gates and an ASCII/Enter
  fallback. UTF-8 output assumes a compatible single-cell font. Resize drops to
  cooked line mode; repeated redraw touches old/new focus rows. No terminal query,
  automatic legacy encoding detection or broad emulator fidelity is promised.
- Raw mode preserves CR/LF bytes, disables echo/signal/flow-control processing,
  restores on normal exit/error and flushes queued typeahead between contexts.
  The stdin descriptor is polled (Darwin cannot poll `/dev/tty`); the local adapter
  has a 100-ms sleeping timeout for resize/decoder deadlines. Future network
  scheduling/admission remains separate; this is not a per-member server design.
- An active member may retain already-displayed content after revoke, but the next
  checked action denies its old session. Theme presentation is never authorization.
  The owner-capable console still must not be exposed as SSH forced command.

## Primary sources and retrieval

Retrieved 2026-10-19, versioned crate manifests/source and license reviewed:

- [XTerm controls, patch 411](https://invisible-island.net/xterm/ctlseqs/ctlseqs.html): CSI/SS3, SGR, cursor and control-string grammar informed the deliberately narrow supported subset; not a claim of complete emulation.
- [Unicode CP437 mapping](https://www.unicode.org/Public/MAPPINGS/VENDORS/MICSFT/PC/CP437.TXT): high-byte mapping reference. The table was generated from Python's standard CP437 codec; low controls are rejected, with an explicit DOS-house treatment of 127.
- [TOML 1.1](https://toml.io/en/v1.1.0) and [toml crate docs](https://docs.rs/toml/latest/toml/): standard config and table parsing. Cargo.lock pins the actual crate source/checksum; strict schema validation follows parsing.
- [rustix poll 1.1.5](https://docs.rs/rustix/1.1.5/rustix/event/fn.poll.html): safe readiness wrapper and the documented Darwin `/dev/tty` limitation.
- [SAUCE revision 5](https://www.acid.org/info/sauce/sauce.htm): metadata/font/iCE semantics remain future explicit work; they are not silently inferred by this importer.
- [Owner art/navigation requirements and examined references](../research/0025-menu-and-art-direction.md): no artwork imported, no original DOS Oblivion installation or new artist-quality screen claimed.

## Evidence, confidence and falsification

ARM64 Darwin / Rust 1.96.0: 12 presentation regressions passed alongside 14 core
and two CLI tests. Real PTYs exercise identity-before-menu, arrows/Enter, lowercase
direct posting, editor isolation, CRLF, hostile DCS tails, Escape timeout, manual
season, resize/fallback, malformed art rejection, pending denial, active revoke
and normal echo/cursor restoration. The unchanged core persistence/recovery suite
still passes. The built-in menu is 668 bytes; first old/new focus redraw 137 bytes.
These are software/byte-count results, not capacity, pixel-perfect art or network
security guarantees. Linux CI repeats these cases on the PR. Native NetBSD execution
of this new application increment remains unperformed; no physical hardware used.

Confidence is high for the tested subset, moderate for real terminal/font breadth,
and unknown for full scene art and multi-member server resource costs. A pending/
revoked action succeeding, malformed sequence dispatching a tail hotkey, unsupported
art control reaching output, partial invalid theme activation or failed normal
TTY cleanup falsifies this increment. Add native NetBSD PTYs, broader hostile
grammar/fuzz tests, terminal/font fixtures, long art/SAUCE/iCE and integrated
SSH/chat/load validation before wider support claims. Calendar scheduling remains
under #25; the rights/provenance and quality review of actual art remains #31.

## Reversal cost

Presentation is replaceable without changing member/room data. Theme format is
versioned but development-only, with no stable pack API promise yet. Broader ANSI
or a framework can replace the renderer behind the same validated action boundary;
changes to accepted files/dimensions need explicit format/tests. Calendar overrides
must preserve the command map and switch only at safe session boundaries.

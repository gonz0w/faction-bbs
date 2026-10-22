# Custom menus and SAUCE/iCE software validation — #25

Date: 2026-10-22. [Saved result](results.json). Current candidate software only;
not a supported release, capacity benchmark or final scene-art acceptance.

## Executed

- macOS ARM64 / Rust 1.98: 167 workspace Rust tests, 111 Python tests, strict
  workspace Clippy/rustfmt, unchanged product/four-experiment dependency inventories,
  active shared-UI Clippy and its credential-permit test.
- Disposable native NetBSD 11 ARM64, two cores / 1 GiB, source-built Rust 1.96:
  167 workspace tests, release build, original INN offline fixture, normal local
  console/reader and Unicode/modern/Kitty/native-gate regressions.
- Four custom-menu PTY controls per OS: spatial Right/Enter with CP437/iCE login,
  compact ordered arrows/ASCII, plain hotkeys and inherited-worker grid.
- Four actual shared controls: restricted SSH grid focus across separate reads,
  compact SSH fallback, native-client grid/imported art and invalid-grid preflight
  before binding. Tests retain eight sessions / five-second refill.
- Nine seasonal local/worker controls per OS, eight actual shared calendar controls
  and 22 shared reader/Unicode/graphics/revocation controls on the final candidate.
- Existing six list/seasonal SVGs reproduce byte-identically. New grid/menu login
  previews use a fixed reference palette, ASCII/known CP437 glyphs and canonical
  position/SGR cells; rendered cell images were inspected locally. These are not
  emulator or terminal-font screenshots/certification.

The result pins **130 selected final build/run inputs**, each compared against
the actual guest source tree; seven host harness inputs; two new SVGs; installed
native app/service/frontend binary hashes; exact test/control counts and teardown.
No credentials, endpoints, logs or VM images are committed.

## Limits and reproduction

The menu unit-test cell oracle exercises 40/79/80/132/240 columns, full/delta
agreement and exactly one focus marker. Full output stays below 4 KiB and delta
at most 512 bytes. Import keeps 80x16 cells and 16 KiB total input/output, including
SAUCE metadata; it never emits terminal blink or metadata. Login preview including
welcome is 1,447 bytes at ANSI 80 columns, 181 at ANSI 40, 142 plain.

```sh
cargo test --workspace --release --locked
cargo build --workspace --release --locked
cargo build -p faction-terminal --examples --release --locked
python3 tools/smoke_custom_menus.py target/release/faction target/release/faction-member
python3 tools/check_custom_menus_evidence.py
```

Native builds ran unprivileged against the already reviewed offline vendor tree.
`tools/smoke_shared_custom_menus.py` requires a deliberately marked, privately
configured disposable synthetic NetBSD fixture; never run it against a real board.
It starts/stops that fixture owner but changes no clock, firewall, admission or
host tuning. Its terminal-settings Done command is line-submitted with Enter.
The reader/calendar fixtures likewise use their separate stopped synthetic paths.

The owned clone is shut down and removed, private vendor cache removed, saved
playground remains off/untouched. Prior seasonal results and 133 executed input
bytes are preserved in the seventeenth snapshot; twenty-nine historical record
identities remain unchanged. Do not replace those hashes with this candidate.

There is no fresh 250-user/RSS/CPU, physical SBC, broader emulator/font or final
artwork claim. Long/animated art and final original/licensed identity remain open;
hardening #106–#111 stays later. This profile accepts one bounded subset of
[SAUCE revision 00.5](https://www.acid.org/info/sauce/sauce.htm), retrieved
2026-10-22, rather than every artscene format or historical terminal.

# Long-form ANSI login paging software validation — #25

Date: 2026-10-22. [Saved result](results.json). Current candidate software only;
not a supported release, capacity benchmark or final scene-art acceptance.

## Executed

- macOS ARM64 / Rust 1.98: 171 workspace Rust tests, 112 Python tests, strict
  workspace Clippy/fmt, unchanged product/four-experiment no-JS inventories and
  active shared-UI Clippy/credential-permit test.
- Native NetBSD 11 ARM64, disposable two-core / 1 GiB VM, source-built Rust 1.96:
  171 workspace tests and release build, original INN offline controls, ordinary
  local console/reader, Unicode/modern/Kitty/native-gate regressions.
- Eight local/worker paging controls per OS: pending denial, exact three-page
  flow/inert invalid key, skip, compact/plain, resize, inherited worker and
  current approval recheck before another page.
- Eight actual restricted SSH/native controls: ordered pages/coalesced input,
  preloaded coherence after live file corruption, skip, compact, native three-page
  login, actual SSH resize after output drain, pending denial, revocation and
  bad-later-page preflight (combined assertions form the eight named controls).
- Four local custom-menu and nine seasonal controls per OS; eight actual shared
  calendar controls and 22 shared reader/Unicode/graphics/revocation controls.
- Eight older SVGs reproduce unchanged. Three original long-art page previews
  reproduce; known-cell images were inspected using a local fixed font/palette.
  Those are authoring artifacts, not real emulator/font screenshots/certification.

The record pins **135 selected final build/run inputs**, each compared to actual
native source bytes; nine host harness inputs; three generated SVGs; installed
native app/service/frontend hashes, exact counts, limits and teardown. No fixture
credentials/endpoints/transcripts/images are committed. Prior #124 input/result
bytes remain in the eighteenth snapshot, without rewriting historical hashes.

## Limits and reproduction

Long art is explicit: <=80x256 cells / 64 KiB total file, <=16 KiB canonical art
per 16-row page. Short `identity_ansi` stays 80x16 / 16 KiB. Every page validates
before activation; tests include inherited styles, a 206-row asset, last-page
coordinates, later control injection and per-page output amplification. A member
has no whole-art cache. Shared output remains 32 KiB / five-second drain with
normal eight-session / five-second admission defaults.

```sh
cargo test --workspace --release --locked
cargo build --workspace --release --locked
cargo build -p faction-terminal --examples --release --locked
python3 tools/smoke_login_paging.py target/release/faction target/release/faction-member
python3 tools/check_login_paging_evidence.py
```

Native builds ran unprivileged against the reviewed offline vendor tree.
`tools/smoke_shared_login_paging.py` requires the privately configured, marked
synthetic fixture; never use it on a real board. It copies/changes only its stopped
theme/config fixture, retains normal admission, tests with actual member SSH/native
routes, registers/revokes a synthetic account and leaves the owner stopped.
No host clock, firewall or tuning changes occur. Native source and installed
binary hashes attest this run; they are not release-reproducibility guarantees.

The owned clone is shut down/removed and private vendor cache removed. Saved
playground stays off/untouched. No new RSS/CPU/250-user, physical SBC, broader
emulator/font, final identity-art or hardening claim. Animation, larger formats
and final original/licensed artistic acceptance remain #13/#25/#29; hardening
#106–#111 stays later. See [ADR 0046](../../decisions/0046-paged-long-form-login-art.md).

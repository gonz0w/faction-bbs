# BBS menu/theme software validation

Executed 2026-10-22 for #25 / plan 0046. [Results](results.json) retain the
selected source hashes, installed native binaries, encoded-output measurements,
checks and qualified claims. `tools/check_menu_themes_evidence.py` validates the
saved record; it does not boot NetBSD or certify an emulator.

154 Rust tests pass on macOS 27 ARM64 / Rust 1.98.0 and a disposable 2-core,
1-GiB NetBSD 11 ARM64 VM / Rust 1.96.0. 109 Python tests pass on the host.
Strict host Clippy/rustfmt and selected Linux/NetBSD no-JS inventory checks pass.
Local/worker PTYs cover approval, hidden passwords, arrows/hotkeys, themed
wide/compact menus, manual autumn colors, room/article input and classic/plain
recovery. Five Unicode, five modern-terminal, seven Kitty and three native-gate
software controls also pass on each OS; image controls do not certify GUI pixels.

The shared native frontend passes 22 controls, including the previous nineteen
reader/Unicode/graphics/revocation controls and three menu additions: compact
40x18 autumn theme, 80x24 descriptions/fragmented arrows/Enter, and resize to plain
input. This driver uses a root-orchestrated owned PTY. A separate actual restricted
member-SSH fixture passes 80x24 and 40x18 menus, plus an 80x24 host native-client
route, using the shipped winter override. Both arrow/Enter and direct hotkeys
open the same room view. All fixtures use synthetic credentials/boards and normal
eight-session / 5000-ms admission defaults; no plaintext/private profile is public.

The 116 selected build/run inputs include the current workspace, normal native
frontend/build helper, themes and maintained terminal controls plus the new
preview example. They were byte-compared with the final guest source tree;
this does not claim every source line/file executed on both OSes. Shared/SSH
orchestration runs on the host; local Rust/PTY tests run in the guest. Three new
host harness sources are pinned separately. Historical #121 evidence is frozen
under `sources/pre25`; its twelve emulator profiles were not rerun for this change.

The independent cell oracle checks ten maximal-length entries at 40/79/80/132/240
columns: no wrap/scroll, stable unavailable markers and exact full-render versus
incremental-render agreement. Full frames measure 1311–1974 bytes and two-row
updates 170–248 bytes in this fixture; conservative regression ceilings are
4096/512 bytes. The single shared theme owns the new title/description strings;
there are no new resident Client fields, helpers, dependencies or database schema.
Queue/drain/credentials/admission bounds remain unchanged. These are not fresh
resident-memory, CPU, 250-user capacity or physical SBC measurements.

Original reference-palette SVGs derive from the actual renderer and regenerate in
CI. A locally rasterized cell preview was inspected for hierarchy/alignment; it
was not a desktop capture, a real emulator/font test or accepted identity art.
The illustrative palette isn't a promise about a member's terminal palette.
No historical scene artwork, screenshot, font binary or new graphics passthrough
is distributed. Calendar/grid/long-art/SAUCE/iCE and actual identity-art acceptance
remain #25. Hardening #106–#111 stays later.

Reproduce host checks using the root workflow and pack guide. For native software
work, copy the same reviewed workspace/dependencies into a new NetBSD fixture,
build/test as an unprivileged build user and use the normal service/native frontend.
`smoke_shared_room_reader.py --disposable-fixture --themes-dir PATH` expects the
trusted themes already in the marked guest. `smoke_member_menu.py` expects private
synthetic credentials and an existing owned restricted SSH/native profile. Never
point the destructive shared fixture at a real board. The fixture VM was stopped
and removed; the original private saved playground remains off and unchanged.

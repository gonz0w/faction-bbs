# Original Nightwire artwork validation — #25

Date: 2026-10-22. [Saved result](results.json). New opt-in assets and authoring
checks; application/default packs/installer remain byte-for-byte unchanged.

## Executed

- 171 workspace Rust tests on macOS ARM64 / Rust 1.98 and native NetBSD 11 ARM64 /
  source-built Rust 1.96; 112 host Python tests; unchanged dependency/no-JS inventory.
- Eighteen compiled-product preview profiles per OS across long/short/autumn/
  winter packs: ten ANSI pages, four compact and four plain identities. Short
  cover equals long page one; seasonal glyph geometry and menu maps remain equal;
  non-space cells have differing foreground/background; source/SVG bytes reproduce.
- Seven real local/inherited-worker login routes per OS and seven actual NetBSD
  restricted SSH/native routes: three-page art, short cover, autumn, winter,
  compact/plain cards and native/worker skip-to-ASCII recovery.
- Seven reference-palette SVGs generated from the actual canonical product
  output. Known-cell images were visually inspected for lettering, composition,
  margins and continuity. No real emulator/font screenshot or owner acceptance.

The result pins **157 selected final native source/input matches**, including the
135 unchanged prior execution sources, twelve new pack files, three native
art/check/PTY tools and seven SVG comparison inputs. Five host harness sources,
all twelve art files and seven previews are separately identified. Native installed
app/service/frontend hashes, counts, limits and teardown are included. No private
credentials/endpoints/transcripts/images are committed.

All **144 distinct prior paging inputs** and its whole result record remain
unchanged. Existing eighteen source snapshots/thirty historical record identities
are preserved; no nineteenth snapshot is needed when adding only new assets and
unmeasured authoring/docs files. Never refresh historical hashes for new art.

## Reproduce

```sh
cargo build --workspace --release --locked
cargo build -p faction-terminal --example preview_theme --release --locked
python3 tools/build_nightwire.py --check
python3 tools/check_nightwire.py target/release/examples/preview_theme
python3 tools/smoke_nightwire.py target/release/faction target/release/faction-member
python3 tools/check_nightwire_evidence.py
```

Native builds ran unprivileged in an owned disposable two-core / 1 GiB NetBSD VM
against the reviewed offline vendor tree. The fixture must include the seven SVG
inputs for comparison. `tools/smoke_shared_nightwire.py` requires a deliberately
marked, privately configured synthetic fixture; never use it on a real board.
It selects the new packs through the existing stopped config/restart lifecycle,
paces member connections and preserves eight-session / five-second admission,
32 KiB output and five-second drain defaults. No host clock/firewall/tuning changes.

The owned VM and private vendor cache are removed; saved playground remains off/
untouched. This is original Codex-assisted artwork with declared BSD rights,
not archival-art permission, a supported release, new RSS/CPU/250-user proof,
physical SBC validation, actual terminal/font certification or owner visual
acceptance. See [artwork/provenance guide](../../../docs/artwork-nightwire.md).
Hardening remains later.

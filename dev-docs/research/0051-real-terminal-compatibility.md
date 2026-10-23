# 0051: Real emulator and terminal-route compatibility

Status: completed bounded software matrix, 2026-10-22. Issue #118 after merged
#120. Decision affected: ADRs 0041/0042 remain unchanged. Stopping condition:
measure pinned native terminal profiles and actual local/member-SSH/native paths,
qualify fallback, record costs/limits, and remove owned fixtures.

## Question and gates

Can the existing native text and fixed-image preview cross real emulators,
fonts, SSH and multiplexers without inferring capability from TERM or widening
terminal controls? The product remains Rust/NetBSD, classic by default, with
current approval/credential/helper/output/admission/probe limits. No physical SBC
or new 250-member experiment is needed. Hardening remains later.

| Alternative | Evidence and outcome |
| --- | --- |
| Docs-only feature lists or mocked reports | Useful context; insufficient for route compatibility |
| Generic APC/DCS passthrough | Reject: exceeds reviewed trust/control boundary |
| Actual isolated native GUI references and synthetic BBS | Selected; named reports, cells and retention are measurable |
| Whole-desktop capture/OS permission changes | Not needed; capture unavailable, visual claims withheld |
| Automatic brand-based profiles | Reject; explicit preferences and conservative fallback remain |

Passing means every classic cell case aligns, every actual BBS route completes
its Unicode article and C recovery, and no unclassified input is replayed or
retained. Native cell mismatches qualify a route as classic-only. Missing replies
are conservative refusals. Image uploads require a real positive probe and a
renderer acknowledgement of the stored image, followed by ENOENT after C. These
criteria are declared in plan 0045 and the bounded fixture collectors.

## Primary sources and identities

- [Ghostty configuration](https://ghostty.org/docs/config/reference), retrieved
  2026-10-22: isolated config/default-file controls, font fallback, command startup
  and window state. Installed 1.3.1 reports CoreText/Metal and Zig 0.15.2.
- [Kitty 0.49.2 release](https://github.com/kovidgoyal/kitty/releases/tag/v0.49.2)
  and [graphics specification](https://sw.kovidgoyal.net/kitty/graphics-protocol/),
  retrieved 2026-10-22: pinned artifact, queries, stored-image placement and deletion.
- [Alacritty 0.17.0 release](https://github.com/alacritty/alacritty/releases/tag/v0.17.0),
  retrieved 2026-10-22: pinned ordinary-terminal reference, revision 94e7c88.
- [tmux 3.8 release](https://github.com/tmux/tmux/releases/tag/3.8) and
  [libevent 2.1.13](https://github.com/libevent/libevent/releases/tag/release-2.1.13-stable),
  retrieved 2026-10-22: private C-only build, static libevent, existing ncurses 6.6,
  optional utf8proc/jemalloc disabled. Source filename scans pass no-JS policy.
- [xterm control reference, patch 411](https://invisible-island.net/xterm/ctlseqs/ctlseqs.html),
  retrieved 2026-10-22: DECRQSS refusal is DCS 0 $ r ST. The initial observer missed
  that valid seven-byte negative reply through tmux; only the fixture grammar was fixed.

Exact artifact, binary, machine-code-section and source hashes are in the result.
GUI bundle IDs/signatures are private derivatives to isolate AppKit state; the
main ARM64 __text sections match source references. This is not untouched stock
package certification. Kitty's 16 documentation JS files are excluded before
fixture execution. No reference tool becomes a product dependency.

## Actual results

[The member guide](../../docs/terminal-compatibility.md) contains the qualified
profile table. Twelve profiles produce 336 cell/cluster observations and 48 actual
BBS routes (local, restricted NetBSD member SSH, native default, native --kitty).
Nine profiles match the selected native corpus. Three old-screen profiles are
classic-only because emoji widths differ; all classic cases and C recovery pass.
No new grapheme policy follows from raw cluster observations.

Ghostty/Kitty direct and missing-font profiles retain and delete the exact product
image on local/SSH/native-opt-in routes: twelve positive retention/deletion pairs.
Native default never uploads graphics and never passes sync; tmux and screen
suppress the image probe, Alacritty declines it. No passthrough wrapper is used.
Real SSH PTY resize controls recover plain menu input or close/delete an active
image. The fixture's OS-window resize request is ignored by the tested emulators.

Each cursor query's first reply is below 1 ms in this local setup. The fixed
300-ms checked collection window is not a GPU refresh measurement. Sampled GUI
RSS ranges 100,528–139,968 KiB, excluding child/GPU/kernel costs and never charged
as a server/member capacity measurement. Observer replies retain at most 128 bytes;
research relay pending output is capped at 64 KiB and its synthetic transcript
at 512 KiB. Existing server 32-KiB/five-second and one-frame/second image bounds stay.

151 Rust tests pass on macOS and native NetBSD; 107 Python tests, format/clippy,
selected dependency inventories and historical byte guards pass. 115 current
application/build inputs match the native checkout; seven separately pinned
harness sources identify host-only measurements. Fourteen source snapshots and
twenty-six original records remain unchanged.

## Failures, confidence and reversal

Early GUI setup reused AppKit state and wrapped arguments incorrectly. Fresh
fixture identities, an owned start script and guarded focus restoration resolved
startup. Relay geometry must be read before fork, and local 80-column ANSI login
has a welcome pause. These were fixture corrections, not product patches. A
preliminary contaminated keyboard run was discarded; accepted runs have zero
unclassified input. Known reports alone can cross the relay, never arbitrary keys.

Confidence is high for the named byte/cell/retention and recovery paths, with actual
contrary screen-width evidence. Screenshot permission is unavailable; pixel/glyph
appearance, GPU FPS, all OS/font/terminal revisions and physical key mappings are
unverified. Continue visual/artwork checks under #25 before broader aesthetic or
GUI conformance claims. No host/firewall/extended-input hardening is adopted.

C/reconnect or omitting --kitty reverses presentation choices with no data/schema
migration. Temporary references, mounts, owned GUI processes and the synthetic
NetBSD clone are removed; the original saved playground remains off/private.

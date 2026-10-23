# ADR 0042: Opt-in trusted inline RGB preview

Status: accepted for the software prototype, 2026-10-22, #117.
See [research 0050](../research/0050-kitty-graphics.md) for alternatives, primary
protocol evidence, bounded costs, confidence and falsifiable gates.

Keep classic ANSI/plain as default. Add explicit approved-member T/G preview
with a dedicated 300-ms/128-byte positive Kitty query state. Use only an original
160x80 read-only RGB asset, thirteen <=4-KiB commands, one frame/second, one
image ID/session and current approval checked per shared output chunk. Stream
after drain under existing work/32-KiB/five-second limits; add a whole-transfer
five-second deadline. Never hold an encoded whole frame per member.

The native client needs explicit local `--kitty` and its own positive local probe
before SSH, plus an exact-command/order/rate/deadline allowlist that regenerates
only this fixed asset. Ordinary native filtering remains conservative. Queries
and graphics never grant membership, file controls, keys or host authority.

Retain classic recovery and best-effort image-specific teardown, including partial
APC termination. Reject arbitrary remote art, image paths, shared memory, PNG,
compression, network fetch, clipboard, Sixel and extended input. No new package,
helper, schema, admission/default change or host/hardening policy is needed.

Software byte/PTY validation supports this narrow preview, not actual GUI
certification, universal multiplexer compatibility, hardware or 250-member capacity.
The fixed sigil is calibration artwork, not final identity art. Disable with C/D
or the absent native flag; no persisted data rollback is needed.

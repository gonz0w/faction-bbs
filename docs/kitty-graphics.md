# Optional inline graphics preview

Working software preview for #117, 2026-10-22. Classic ANSI/plain remains the
default. After approved login, choose **T** (Terminal), then **G** (Graphics).
An explicit, bounded Kitty query must succeed before one original RGB calibration
image is transmitted. The image is a frequency sigil for testing the new path,
not replacement BBS identity art or an importer for scene archives. Existing
sysop ANSI/CP437 themes remain the normal welcome and menu presentation.

**C** clears the image and restores classic preferences. **D**, logout and cleanup
remove it. Reconnecting starts classic. Unknown/unsupported terminals simply
return to settings with a fallback message. Plain mode sends no graphics probe.
Stay idle during the brief query; malformed/incomplete/extra replies close the
session rather than become menu or posting input. Probe windows are 300 ms after
query drain, with a 128-byte reply ceiling. These are checked deadlines, not
hard realtime guarantees or automatic brand/TERM detection.

## Native client

Enable the member-side gate deliberately on your own device:

```sh
faction-client --profile /absolute/private/board.toml --kitty
```

The client first probes **its local terminal**, before starting SSH. A refusal or
no reply keeps filtering graphics. A positive result enables only the exact
Faction query, fixed image's ordered chunks, and deletion of that image ID.
The filter regenerates trusted commands; it never forwards arbitrary APC.
Then log into the board and choose T/G normally. Without `--kitty` the existing
APC/query filter remains active. File-transfer controls, local profile/host-key
verification and BBS approval are independent and unchanged. Sync queries still
stay filtered by the native client.

## Limits and recovery

- One 160x80 RGB image: 38,400 immutable raw bytes, 51,200 base64 payload bytes,
  51,410 total upload bytes in thirteen commands; largest command 4,054 bytes.
- One 4,096-byte chunk allocation at a time, no whole-frame encoded temporary,
  per-session image cache, decoder, new dependency or owner helper. The static
  artwork is read-only executable data; it is not allocated per shared member.
- Uploads stream only after each preceding shared output queue drains, consuming
  rotating work credits with current approval rechecked for each chunk. Existing
  32-KiB queue and five-second drain limits remain; a whole upload also has a
  five-second checked deadline. Standalone output uses bounded nonblocking writes.
- At most one new frame per second. The native gate independently enforces rate,
  order, size and unfinished-upload timeout. A partial command is held until
  complete verification. Cleanup terminates a partial escape and requests deletion
  of only the fixed Faction image ID; broken transports can require a terminal reset.
- No terminal-side files/shared memory, PNG/compression, URL fetch, clipboard,
  Sixel, animation, arbitrary remote/member pixels or game-art passthrough.

The fixed image uses cell placement 20x5. Real fonts, scaling, cursor/scrollback
behavior and terminal implementations can differ. Resize during shared graphics
closes conservatively and attempts cleanup. SSH carries inline bytes without a
shared filesystem. Multiplexers may suppress the query; no tmux passthrough
wrapper or brand-based override is sent. Actual GNU screen 4.00.03 returned no
reply in the owned fixture. Ghostty/Kitty GUI and tmux combinations remain #118.

See [ADR 0042](../dev-docs/decisions/0042-kitty-graphics.md),
[research 0050](../dev-docs/research/0050-kitty-graphics.md) and
[software evidence](../dev-docs/evidence/kitty-graphics-netbsd/README.md).
Normal eight-session/helper/credential/schema limits stay; no new capacity,
hardware, public-listener or later-phase hardening claim is made.

# Kitty graphics software evidence

2026-10-22, #117. The final run validates 151 Rust workspace tests on macOS
ARM64 and native NetBSD 11 ARM64, seven real local/worker graphics PTY controls
and three native member-side gate controls per host. Existing five modern and
five classic Unicode PTY controls, room-reader and full-console regressions pass.
Final source identity and disposable-VM teardown are recorded in results.json.

Nineteen shared NetBSD UID/PTY controls include the original room/subject/unread/
revocation checks, positive chunked graphics, unknown/refused fallback, malformed/
incomplete/late response isolation and session interruption/cleanup after upload
starts. This fixture is orchestrated over admin SSH with the native confined
frontend; it does not claim new member-listener/admission or actual GUI testing.
Its preflight initially refused the unmarked clone. Only after checking the owned
QEMU disk/process identity was that disposable clone marked; no real board was
used and no production preflight/default was weakened.

The real native local-gate PTYs create private test keys/profile and use an owned
refused loopback destination. They verify no probe by default, explicit local
positive/no-reply/refusal, malformed/incomplete rejection and TTY restoration.
Independent Rust filter controls validate byte-fragmented exact remote chunks,
ordered/rate/deadline gates and rejection of arbitrary paths/shared memory/PNG/
wrong sizes/truncated payloads. Together these test the component contracts,
not a complete GUI/native-client/member-SSH interoperability matrix.

One read-only original image is 38,400 bytes. The measured transfer is 51,200
base64 / 51,410 wire bytes in thirteen commands; maximum command 4,054 bytes,
chunk allocation capacity exactly 4,096 bytes. Probe struct is 40 bytes plus at
most 128 response bytes; native gate struct is 64 bytes. The native parser holds
a 4,096-byte APC buffer and one expected 4,096-byte chunk, emitting verified
chunks as separate events. There is no per-member image cache, full encoded
frame or new dependency/helper. See [research 0050](../../research/0050-kitty-graphics.md)
for bounded transient buffers and costs excluded from these measurements.

Classic ANSI/plain stays default. One image/session and one frame/second are
independently bounded; upload and queue drain deadlines are five seconds.
Shared output remains 32 KiB with approval rechecks and rotating work. Normal
admission remains eight/5000 ms; the fixture paces new sessions. Cleanup first
terminates partial APC, then deletes only the Faction image; broken transports
can require emulator reset. Actual GNU screen 4.00.03 returned no Kitty reply,
so fallback is measured. Real Ghostty/Kitty fonts/display and tmux stay #118.

112 exact final application build/run inputs are byte-compared to the native
checkout. The installed-screen query runs only on the host, with its source
copied for identity. All thirteen frozen snapshots and twenty-five older result
identities are unchanged. Format/clippy, 104 Python tests, dependency no-JS
inventories and historical evidence guards pass. The disposable 2-core/1-GiB
VM is removed; the saved original playground remains off and private. No
physical SBC, supported 250-user capacity, arbitrary image import, Sixel, new
mail/federation, public listener or later hardening is claimed.

Reproduce with workspace release tests/build and tools/smoke_kitty_graphics.py,
tools/smoke_native_graphics.py and existing modern/Unicode/reader/console PTYs.
Build/run the faction-terminal graphics_budget example for actual sizes.
tools/probe_screen_kitty.py optionally runs an installed screen in a private
detached session. tools/smoke_shared_room_reader.py requires a marked disposable
NetBSD fixture and destroys synthetic state; never run it on a real board. The
saved-evidence checker verifies recorded identities, not a fresh VM or GUI.

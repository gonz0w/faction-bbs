# External compatibility experiments

These are optional game-host experiments, outside Faction's backend/frontend/Lua
identities and build. They do not bundle games or adopt a production launcher.

[dosbox-0.74-3-inherited-rx.patch](dosbox-0.74-3-inherited-rx.patch) fixes the
inherited-socket receive state in upstream DOSBox 0.74-3. Its constructor schedules
receive polling while leaving `rx_state` disconnected. Initializing it to
`N_RX_IDLE`, as the normal client-connect path does, enables original LORD's
keyboard input. The original game remains unchanged. The patch applies to
GPL-2.0-or-later DOSBox source; that upstream license governs the modified emulator,
not Faction's separate BSD-3-Clause code. No emulator binary is redistributed.

Source: [official release](https://sourceforge.net/projects/dosbox/files/dosbox/0.74-3/),
with SHA512 checked against [pkgsrc distinfo](https://cdn.netbsd.org/pub/pkgsrc/current/pkgsrc/emulators/dosbox/distinfo).
Release archive: `dosbox-0.74-3.tar.gz`, 1,326,339 bytes.

```text
SHA512 3770f1578e71730168fba01809585f2a05cf8c4d420524687ba28791d242faad294401bb9cd268bba725798e1db853551f274dc00a4024331f5dcef9f87f8f5f
```

The native NetBSD game VM built it under a separate build UID with base C++,
GNU make and the pkgsrc SDL development interfaces:

```sh
patch -p1 < /path/to/dosbox-0.74-3-inherited-rx.patch
SDL_CONFIG=/usr/pkg/bin/sdl-config \
CPPFLAGS=-I/usr/pkg/include \
LDFLAGS='-L/usr/pkg/lib -Wl,-rpath,/usr/pkg/lib -pthread' \
./configure --prefix=/opt/faction-dosbox --disable-opengl --disable-sdltest
gmake -j2
```

The owner installed only the resulting `src/dosbox`, non-setuid, at
`/opt/faction-dosbox/dosbox`. SDL dummy video/audio avoid an X server during play;
SDL's linked libraries remain dependencies. OpenGL is disabled in this source
build. Do not attribute any footprint difference solely to the receive fix.

Stock DOSBox 0.74-3 also rejects DOS INT 21h/AH=5Ch record locking. This experiment
therefore serializes the entire original game with an OS file lock. Do not enable
simultaneous original DOS nodes based on successful sequential sessions.

See [original-game research](../research/0042-original-classic-games.md),
[the member/sysop explanation](../../docs/original-lord.md) and
[validation](../validation/netbsd-original-lord.md).

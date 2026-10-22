# Real terminal compatibility evidence

2026-10-22, #118. results.json pins twelve actual macOS terminal/font/multiplexer
profiles, 336 cell/cluster observations, 48 real local/NetBSD member-SSH/native
BBS sessions, twelve image-retention/deletion pairs and two live SSH resize controls.
Nine profiles match native cells; three old-screen profiles qualify only for
classic text. Every route completes a synthetic Unicode article and C recovery.
No mocked terminal response establishes a real-profile result.

Reference versions are Ghostty 1.3.1, Kitty 0.49.2, Alacritty 0.17.0, private tmux
3.8/static libevent 2.1.13/ncurses 6.6 and installed GNU screen 4.00.03. Requested
Menlo and intentionally nonexistent-font profiles are separate; measured TTY
dimensions remain authoritative. Private GUI bundle identities/signatures isolate
configuration/window state. Their main ARM64 machine-code sections match source
references. Kitty HTML/JS documentation is excluded; selected reference files and
all unchanged product dependency graphs contain no JS/TS filenames.

Native client default filtering remains; only explicit --kitty plus its local
positive probe allows reviewed fixed-image commands. Sync stays filtered there.
Tmux/screen image refusal and classic screen recovery are actual observations.
A fixture-only placement request checks stored product pixels and subsequent
ENOENT, not visual appearance or a new runtime command. No DCS passthrough exists.

115 application/build inputs match the final native checkout; seven host harness
sources and separate reproduction sources have hashes. Native frontend and base
SSH binaries are pinned. 151 host/native Rust tests, 107 Python tests, strict
format/clippy and unchanged evidence/dependency checks pass. Fourteen snapshots
preserve twenty-six original result identities. Private credentials, transcripts,
images, binaries and configurations are never committed.

Reproduce on macOS with tools/prepare_terminal_reference.py (new private output
folders), tools/build_terminal_multiplexer.py (private C build), and the
faction-terminal profile_text example. Build tools/native_terminal_window.m with
Cocoa for guarded focus restoration. tools/run_real_terminal_profiles.py runs
fresh fixture copies with --missing-font, optional private --tmux and a private
--bbs-config describing synthetic local/SSH/native routes. It requires fixture
bundle IDs, serializes runs, restores focus only from its own new instance, and
retains launch diagnostics privately. The BBS config contains generated credentials
and must be private; use a marked disposable NetBSD setup from the existing SSH
validation/quickstart, never a real board. tools/probe_terminal_resize.py separately
tests owned SSH PTY SIGWINCH while a real emulator consumes output. No desktop
capture/automation permission is requested. The saved checker does not rerun GUIs.

Reports, cell positions, image retention and bounds are software measurements;
pixel/glyph quality, physical keys, GPU refresh FPS, every font/OS revision,
physical SBC and 250-member capacity are not certified. GUI sampled RSS excludes
GPU/children/kernel and is not a NetBSD server budget. Hardening remains later.
The owned VM, GUI sessions/mounts/reference files are removed; the saved original
playground remains off and untouched.

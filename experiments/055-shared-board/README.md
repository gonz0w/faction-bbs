# Shared board/readiness experiment (#55)

This standalone Rust package is an opt-in experiment, not the normal gateway or
a supported deployment. It uses only existing pinned workspace/path dependencies.
Build with Rust 1.96 and a native C compiler:

```sh
cargo build --manifest-path experiments/055-shared-board/Cargo.toml --release --locked
```

The fixed owner-only invocation takes `--socket PATH --terminal-uid UID --state-dir
PATH`. Require an already initialized private board and the same protected Unix
socket directory/group and separate non-root terminal UID as the development
gateway. It must run instead of the normal broker on that endpoint. Never install
it as the SSH forced command. The existing `faction-ssh` and dedicated OpenSSH
configuration stay unchanged. Normal service continues to default to eight workers.

One readiness loop owns up to 250 terminal/control descriptor pairs, one board
connection, opaque sessions and chat leases. Two fixed credential threads use two
additional board handles; the existing cross-process credential locks still apply.
A fixed handoff thread has a two-second receive deadline and eight-entry admission
queue; its experimental launch budget is eight plus one refill/100 ms. Auth uses
a zero-capacity rendezvous, two fixed workers and a two-result channel. IDs are
assigned internally, monotonically and never accepted from terminals. Disconnect
discards matching auth results. Control bytes after handoff terminate a connection;
there is no member RPC, session import, serialized identity or owner-tool endpoint.

Each loop reads at most 256 input bytes per ready terminal. Input is bounded to
1152 bytes (password 128); output 8 KiB, five-second pending-output deadline,
120-second prompts and 1800-second session lifetime. Output and terminal reads are
nonblocking; no database transaction spans a terminal write. Slow or flooding
connections close individually. Graceful stop restores terminal state, closes
leases and joins fixed workers. SIGKILL closes controls; the unchanged frontend
restores its saved terminal. Abrupt backend death can leave leases until expiry.
The persistent pidfile lock and socket inode guard mirror normal broker policy.

Implemented comparison surface: approved identity, built-in ANSI arrows/hotkeys,
room list, first bounded Lobby history/unread page, Lobby post, #lobby chat and
join/read/names/quit. This is deliberately incomplete UI: no theme-pack/season
loading, plain/resize fallback, complete room navigation/reply/retry workflows,
sysop menus or full chat command editor. Do not adopt it for members until those
behaviors and fairness/maintenance gates pass. Core authorization/grants remain
mandatory on the implemented actions. No owner bootstrap/recovery/export/restore
command is accepted through a terminal.

`tools/check_shared_scaling.py` compares full real SSH transports on a marked
disposable NetBSD VM. `tools/check_shared_oracles.py` and `terminal_probe.py`
exercise approval/epochs, failed foreign connection, forged control, actual guest
PTY backpressure, disconnect/crash, durable post and recovery. These scripts use
synthetic accounts and private generated credentials, never a real member board.
See the recorded research and issue #55 before interpreting historical numbers.

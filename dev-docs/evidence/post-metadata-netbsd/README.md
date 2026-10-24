# Schema-6 metadata software evidence

121 workspace Rust tests pass on macOS ARM64 and native NetBSD 11 ARM64. Native
build and local console/reader PTY controls pass. Five shared native UID/PTY
controls include visible subject/UTC headers, partial/full unread behavior,
ordered pages and current revocation. The shared fixture uses root-orchestrated
UID-owned PTYs, not a new dedicated SSH admission or complete SFTP matrix.

Six metadata tests cover new-post atomicity/idempotency, legacy dates/replies/read
state, old/new export/restore, current approval, malformed/missing metadata rejection
and migration conflicts. The header renderer has bounded inert metadata coverage.
Host fmt/clippy/build, real owner export/restore and worker flows pass. Current
source identities are in results.json; previous #105 inputs/results are separately
frozen, without refreshing historical measurements.

Run the documented Rust/PTY checks and tools/smoke_shared_room_reader.py only on
an explicitly marked disposable NetBSD fixture. See [GitHub issue](https://github.com/gonz0w/faction-bbs/issues/103)
and [upgrade requirements](../../../docs/post-metadata.md). No physical SBC,
new capacity, live NNTP, PGP implementation or adopted hardening claim. The original
playground remains off; the private test clone and synthetic state are removed.

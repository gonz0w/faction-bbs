# Authored subjects and local article software evidence

127 Rust workspace tests pass on macOS ARM64 and native NetBSD 11 ARM64. Five new
core cases cover authored/default subject retry/restore, approval/revocation,
privacy-safe article projection, unknown legacy dates and inaccessible links,
and bounded/cyclic references. A codec test checks normalized duplicates and
conflicts across every field/body. Existing migration/metadata tests remain.
Host format/clippy/build, 104 Python tooling tests, real console/plain/ANSI reader,
export/restore and member-worker checks pass. Native builds and local console/
reader flows pass with the final code.

Seven shared native UID/PTY controls pass: approved member, partial unread,
complete wrapped text, output-drain acknowledgment, inherited subject, authored
reply override and revoked-next-page denial. Admin SSH hands an owned PTY to the
native frontend; this is not a new dedicated SSH admission or full SFTP matrix.
The fixture is a private disposable 2-vCPU/1024-MiB NetBSD clone. No physical gate.

Independent Python standard-library email/MIME parsers on both hosts accept
synthetic emitted headers, UTC date, ordered References and UTF-8 text. They are
separate parsers, not independent news servers or NNTP peer controls.

The synthetic 16,384-byte body produces 17,123 wire bytes. Both hosts measure
65,745 bytes of extra peak requested live allocation while parsing and 32,256
while serializing, above the already allocated fixture baseline. Results include
1,000-iteration elapsed means; these are single observations, not throughput
budgets. Excludes allocator overhead, stack, RSS, SQLite/SSH/board state and
concurrent-member costs. No new capacity claim. The example's counting System
allocator is measurement-only and not linked into product binaries.

The record pins all 95 final executed source inputs. Earlier results, including
#112's 90 inputs and original metadata result, remain unchanged in the ninth
snapshot. The disposable VM is shut down and removed after testing; the stopped
private playground/checkpoint remain untouched.

Reproduce with release workspace tests/build, tools/smoke_local_console.py,
tools/smoke_room_reader.py, tools/smoke_shared_room_reader.py on a marked disposable
NetBSD fixture, faction-articles emit_fixture/profile_articles examples and
tools/check_article_interop.py. See [plan 0040](../../plans/0040-authored-subjects-and-room-articles.md)
and [product limits](../../../docs/room-articles.md). No public room publication,
remote insertion, live NNTP or activated hardening in this increment.

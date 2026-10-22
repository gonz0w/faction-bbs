# Room article and reader batch: native software evidence

The current results.json records 114 workspace Rust tests on macOS ARM64 and
NetBSD 11 ARM64, native build/local PTY checks, and five actual shared-service
controls through the native UID-checked frontend. The shared fixture uses a
root-orchestrated PTY under the dedicated terminal UID; this is not a retest of
dedicated SSH admission. The board and credentials were freshly generated.

The 2-vCPU/1-GiB disposable clone was tested privately while the owner's preserved
playground remained off. Historical source/result hashes are unchanged; the exact
pre90 source snapshot protects the earlier native-client record. The current record
pins this batch's tested source bytes. No new load/250-user, physical SBC, live
NNTP, full SFTP, Unicode graphics or encrypted-mail claim.

## Reproduce on a disposable NetBSD fixture

Build the workspace and run cargo test --workspace --release --locked, then run
`python3.13 tools/smoke_room_reader.py /path/to/release/faction` and the existing
local-console smoke. For the shared fixture, install the ordinary quickstart's
service users, rc wrapper and native frontend, mark it explicitly disposable and
use tools/smoke_shared_room_reader.py with --disposable-fixture, --admin-argv and
--bin-dir. It installs supplied binaries/configuration and creates a synthetic
board; never point it at a real installation. All SSH keys/argv and transcripts
stay private. See [plan 0038](../../plans/0038-room-articles-and-reading.md).

The final checks cover partial-message unread retention, full-page output and
exact word order, drain-before-read acknowledgment and current revocation at the
next page. Codec tests include a hand-authored article fixture, UTF-8 body/trailing
blank lines, header/MIME limits, controls/injection, dates and reference validation.
Unsupported news variants are rejected; independent provider/news software interop
and schema migration remain #103/#101/#104.

## Retained limitations and fixture corrections

An early serial-attachment race lost the VM's initial userconf prompt; starting
with serial wait-on allowed the existing boot controller to attach before boot.
Fixture setup also needed an owner-only board directory, the root tool PATH and
40x18 geometry set at SSH PTY creation. The initial host tool tests copied ignored
local playground artifacts; the fixture now excludes local/. Temporary copies were
cleaned up. The extra source manifest is mocked in resolver tests. These setup
failures preceded the final passing runs and do not establish product regressions.

The renderer's wrap regression split public file-reference tokens; a focused test
caught it. Final rendering keeps those tokens contiguous while charging soft-wrap
rows, preserving native recent-file discovery without granting download authority.

Packaging corrections before review: preserve the hand-authored article's exact
CRLF bytes with a path-specific Git attribute. The initial source ledger also
listed eight ignored Finder .DS_Store files copied with the fixture; remove those
non-source entries. The final ledger has 87 real source inputs, all with their
original tested hashes. No runtime source/test result or historical record was
rewritten.

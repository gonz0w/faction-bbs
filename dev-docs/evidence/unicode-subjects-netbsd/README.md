# Unicode subject software evidence

2026-10-22: 140 workspace Rust tests pass on macOS ARM64 and native NetBSD 11.0
ARM64. A disposable 2-core/1-GiB software VM hosts only synthetic board data and
an admin loopback SSH endpoint; the saved owner's playground stays off/unchanged.

Five real PTY controls on each host cover local plain/ANSI and inherited-worker
input, split UTF-8 scalars, complete/incomplete scalar erase, one inherited Re:
label, exact 160-byte subjects and malformed/control/overflow closure with TTY
restoration. The plain control explicitly starts with ISTRIP set; the subject
editor must temporarily clear it so UTF-8 bytes reach the parser intact.

Twelve real native shared-service/frontend controls include the previous seven
reading/subject/unread/revocation checks plus Unicode input, byte/cell clipping,
Unicode reply defaults, malformed-input closure and revocation during a Unicode
draft. They use the existing native UID-checked frontend and admin-orchestrated
PTYs, not a new dedicated SSH admission or full SFTP test.

Migration tests reject unknown schema-6 metadata catalogs and non-ASCII data in
that old contract, retain IDs/provenance/bytes through read-only old exports and
new restore, enforce the schema-7 SQL byte bound, reject malformed stored UTF-8,
and preserve busy retry/conflict/active-approval behavior. Existing body/paging/
unread/file checks remain. The independent Python MIME parser still passes.
Host format/clippy/build, dependency policy and 104 Python tooling tests pass.

The separate results.json pins 101 actual build/run inputs, byte-compared with the
native checkout. The optional INN build is not repeated in this increment; its
original results and all 100 original inputs remain fixed in sources/pre114.
All eleven snapshots/twenty-three historical record identities stay unchanged.
No new dependency, owner helper/thread, credential/default change, native Unicode
glyph rendering, physical hardware, public endpoint, live feed/provider, hardened
profile or 250-user measurement is claimed. Teardown removes the disposable clone
and synthetic state without touching the saved private playground.

Reproduce with cargo test/build --workspace --release --locked, then
`tools/smoke_unicode_subjects.py`, `tools/smoke_room_reader.py` and
`tools/smoke_local_console.py` against built executables. The shared helper
`tools/smoke_shared_room_reader.py` requires a marked disposable NetBSD fixture;
never point it at a real operator board. `tools/check_unicode_subjects_evidence.py`
only checks the saved record/source identities; it does not rerun the VM.

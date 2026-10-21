# Native file-area software evidence

Date: 2026-10-21. [Results](results.json) pin 74 actual current source inputs.
Run `python3 tools/check_file_area_evidence.py` to verify the saved observations
and source identities. That command does not rerun NetBSD, SSH or SFTP.

A disposable **NetBSD 11.0 ARM64/evbarm VM**, QEMU virt/HVF, two vCPUs and 1 GiB
RAM, used serial-verified host keys and loopback-only host forwards to separate
admin/member/file SSH ports. Official image/kernel provenance follows the signed
[dependency experiment](../../../experiments/023-netbsd-dependencies/README.md);
the release manifest signature was verified with the recorded NetBSD release
fingerprint. Vmt was disabled for this software VM's compatibility; no product
installer applied host tuning, experimental profiles or public network policy.
Rust 1.96.0, Python 3.13.14 and base OpenSSH 10.3 were observed in this guest.
No private keys, endpoints, transcripts, artwork or real member files are bundled.

Native offline release `cargo test --workspace --release --frozen -j 2` and
`cargo build --workspace --release --frozen -j 2` passed, plus native C terminal/
startup builds. The 90 native Rust tests include eight core file-policy tests,
lookup/transfer-role separation, bounded framing, file aliases/locks and menu
capabilities alongside existing approval/chat/score/export tests. The developer
host also passed 90 Rust and 101 Python tests, fmt/clippy/dependency checks and
existing real PTY console/chat/export/member/broker/credential workflows.

`tools/smoke_file_areas_ssh.py` executed 13 named checks in results.json: disabled
installation/private UIDs, pending-member denial, unchanged BBS exec/SFTP/
forwarding denials, private-area grants, multi-chunk real SFTP upload, one-use
replay denial, sysop-only quarantined download/review, publication/search/hash,
real checksummed download, store hardlink/symlink refusal, traversal/oversize/
active-area revocation, interrupted-data privacy, exact paired-hardlink crash GC,
live backup refusal, corrupt-bundle rejection, complete byte/catalog restore and
restart requiring fresh transfer authority. Payload: 131,328 deterministic bytes.
The result's longer labels group related observations; thirteen is the number of
named workflow checks, not the count of individual protocol requests/assertions.

Core tests additionally exercise wrong keys/leases/forged IDs, current epoch and
approval changes, expiry, area deny/regrant, unsafe metadata, concurrent quota
reservation, export authority stripping and restored board identity. File-store
and backup tests reject symlinks, hardlinks, unsafe modes and non-regular objects.
No member-selected filesystem path reaches an OS open operation.

A single **observational RSS sample** during one active transfer and two terminal
sessions is in results.json. It is not a peak, an idle baseline, summed physical
memory, a whole-host/SSH budget, a stress run or a supported-capacity claim. Compare
configured streaming/disk/rlimit caps separately. No physical SBC or 250-user run
was performed for this change. Independent parser/authority review, broader SFTP
clients, resume and hostile native-code/all-family network isolation remain outside
this preview. Database-only export is not a complete file backup.

The final helper rejects root or mismatched real/effective UID/GID at startup;
actual SSH key lookup and data transfers passed with that check. No owner process,
thread, credential slot/parameter or eight-session default was raised. Current
source pins must be preserved before future measured-source edits, never refreshed
into old observations. Historical five snapshots/seventeen records remain fixed.
Task services/children and VM were stopped, forwards verified closed and private
inputs removed, as recorded by teardown fields. Services were never auto-enabled.

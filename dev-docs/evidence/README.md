# Executed source snapshots

Historical native records remain immutable. Before integrating the shared service,
the source bytes they measured were captured under `sources/09b506e`, anchored to
merged PR #61 (`09b506ea4f6da2830a0f2f480918ffb5048749aa`). The manifest pins every
file and every historical result; the resolver pins the manifest itself. Never
update old measurements to new application bytes.

The second 90-file snapshot `sources/627a824`, anchored to merged PR #62
(`627a8241bf736db2371938641d8bef88785d374a`), preserves the maintained service
inputs measured by result 034. All nine historical result files remain unchanged.
Create a new snapshot/record when a later executed revision needs preservation.

These are read-only build/research inputs, not current agent instructions or a
second backlog. The original workspace/Cargo lock, crate graph and standalone
experiments can be built from the snapshot with a separate target directory and
the recorded Rust toolchain/dependencies. No private VM, vendor cache or credentials
are included. Current product builds use the repository root. Historical checkers
resolve pinned bytes here; new service evidence must check current source directly.

Run `python3 tools/check_evidence_sources.py`. It checks actual bytes and unchanged
record hashes; it does not rerun native measurements. Source keeps its BSD-3-Clause
license. Existing older planning archives remain separate and untouched.

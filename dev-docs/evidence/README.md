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

[Result 037](../research/results/037-netbsd-admission.json) preserves the selected
base-SSH key helper, three passing 250-member trials, three rejected inetd-mode
trials (two memory failures), and a live-ledger instrumentation diagnostic.
`admission-ledger/` retains the actual differing diagnostic driver/collector;
`ssh-inetd/` retains changed rejected-candidate input versions. Both manifests
resolve exact bytes rather than refreshing earlier hashes. Current 035/036/037
source pins and all nine historical records/two complete snapshots stay fixed.
Before changing a measured source, preserve its actual executed version properly.

The sustained/abuse follow-up uses new drivers and a new bounded synthetic reset
variant. It leaves every previously measured source/result byte unchanged.
All current 035/036/037 source pins and the historical snapshots/overrides still
apply; preserve executed bytes before changing any source they reference.

[Result 038](../research/results/038-netbsd-sustained-abuse.json) retains both
15-minute active 250-member matrices, bounded unsigned-key/source-penalty probes,
raw post-load CPU controls, actual fixture counts and verified rollback/cleanup.
It pins 137 project execution sources; the new evidence checker validates all
905-sample active streams as well as the original short credential/control streams.
CPU interpretation and combined worst-case pressure remain open gates.

The third buildable snapshot `sources/71491cc` preserves all 137 current execution
inputs of results 037/038 at merged PR #67. The separately pinned
`key-lookup-manifest.json` freezes those bytes and both complete result files;
the original manifest/two snapshots/nine records remain byte-for-byte unchanged.
Historical 037/038 checkers now resolve their measured source versions here.
New bounded-key evidence validates current source bytes directly. Run the same
source check: it now verifies three snapshots and eleven historical records.

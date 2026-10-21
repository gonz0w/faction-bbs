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

The fourth snapshot `sources/5b62d0a` preserves all 145 executed result-039 inputs
at merged PR #68. `global-admission-manifest.json` freezes those bytes and the
complete unchanged record. The resolver pins that manifest independently; all
four snapshots and twelve historical records are verified. New result 040 checks
its current source bytes directly. Records 035/036 retain their unmodified direct
pins; neither historical records nor their original manifests were refreshed.

The small-board boot fix changes the normal rc script. Its actual old bytes
(`6d5fe76f…`) are already present in the fourth snapshot. Results 035, 036 and 040
now resolve their historical execution maps through the existing source resolver;
their complete unchanged JSON records are independently pinned by
`check_evidence_sources.py`. Four source snapshots and fifteen result identities
are checked. New ordinary-install/boot observations are documented separately;
none of the historical capacity matrices is claimed to have measured the new rc
script or startup helper. Preserve other still-current measured inputs before
future changes. No old record or snapshot hash was refreshed.

The original LORD experiment changes only the maintained classic parser. Its
previous measured bytes are preserved under `classic-doors-netbsd/pr80/`, with
the complete old connector record pinned by `sources.json`. The original JSON
remains unchanged. `tools/check_original_games_evidence.py` checks those pins
alongside the new actual-title record; it does not rerun native gameplay.


The fifth snapshot `sources/pre83` preserves 220 execution inputs at the merged
#82/#85 base `daae757a549003a55c23b32593ffe66f4f351b33`. Its independent
`file-areas-baseline-manifest.json` also pins the unchanged original LORD and
classic connector result identities. Together the source checker verifies five
snapshots and seventeen historical record identities. This is an execution-input
snapshot; older buildable snapshots retain their original meaning. No prior
measurement/hash was refreshed to describe the file-area implementation.

The sixth snapshot `sources/pre84` preserves 74 measured file-area inputs from
merged #86 (`c85dbd1d60e9c8bb8646ed937f2ee4973e58f5f1`). Its independently pinned
`native-client-baseline-manifest.json` retains the complete original file-area result.
The earlier five snapshots/eighteen historical record identities remain fixed.
[Native-client evidence](native-client-netbsd/README.md) records new current-source
workflow tests separately; historical file-area tests were not relabeled as new runs.

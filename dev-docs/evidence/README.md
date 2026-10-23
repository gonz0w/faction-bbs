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

[Encrypted-mail/identity evidence](encrypted-mail-netbsd/README.md) adds a separate two-home protocol/native verifier record and retains native signing failure. No maintained executed input or older record was changed, so no historical snapshot is replaced or relabeled.

- [Room reader and article batch](room-reader-netbsd/README.md) — native codec tests and UID/PTY shared-reader controls; no new capacity claim.

- [Post metadata evidence](post-metadata-netbsd/README.md) — schema-6 migration/restore tests and real native metadata headers; no hardened deployment claim.

- [Authored subjects and local article NetBSD software evidence](authored-articles-netbsd/README.md)

- [Broader article profile and original INN software evidence](broader-articles-netbsd/README.md)

The fifteenth snapshot `sources/pre25` freezes all 124 distinct source inputs
from the original [real terminal record](terminal-compatibility/results.json):
115 application/build, seven harness and two reproduction files. Its pinned
`terminal-compatibility-baseline-manifest.json` also locks the whole record.
Fifteen snapshots and twenty-seven historical result identities remain unchanged;
menu changes do not inherit #118's emulator/image results as fresh measurements.

The sixteenth snapshot `sources/pre25-calendar` freezes all 122 distinct inputs
and three generated previews used by the original menu record (#122); those
previews are included within the 122 total. `menu-themes-baseline-manifest.json`
pins both the source tree and unchanged result. Keep sixteen snapshots and
twenty-eight historical result identities fixed when extending calendars/art.

The seventeenth source snapshot, `sources/pre25-layouts`, preserves 133 distinct
executed #123 application/harness/SVG inputs. The pinned
[seasonal manifest](seasonal-themes-baseline-manifest.json) also fixes the whole
original seasonal result record. Seventeen snapshots and twenty-nine historical
record identities remain unchanged; do not refresh their hashes to later code.

[Custom-menu NetBSD software validation](custom-menus-netbsd/README.md) records
130 final guest input matches, 167 Rust per OS / 111 host Python, four custom
PTY controls per OS and four actual shared SSH/native controls, with seasonal/
reader regressions. No fresh capacity, terminal/font or final artwork claim.

Eighteenth snapshot `sources/pre25-paging` and the pinned
[custom-menu baseline](custom-menus-baseline-manifest.json) preserve 137 executed
#124 inputs and its original result. Keep eighteen snapshots / thirty historical
record identities unchanged when adding long-form login art.

[Login paging NetBSD software validation](login-paging-netbsd/README.md) records
135 final guest input matches, 171 Rust per OS / 112 host Python, eight paging
PTY controls per OS and eight actual shared SSH/native paging controls, plus
calendar/reader regressions. No fresh capacity/font/final artwork claim.

[Original Nightwire artwork validation](nightwire-netbsd/README.md) records eighteen
render profiles and seven local/worker routes per OS, seven actual SSH/native
routes and 157 native input matches. All 144 prior paging inputs remain unchanged;
new assets require no new frozen snapshot. Owner artwork acceptance and real
emulator/font certification remain open, with no new capacity claim.

[Nightwire real-terminal measurements](nightwire-terminals/README.md) record
canonical artwork/menu placement and CP437 widths through private real macOS
emulators, including actual compact windows. The original native Nightwire
record and its 167 distinct inputs remain unchanged. No new source snapshot,
native/capacity, pixel/font certification or product runtime change is implied.

`nightwire-baseline-manifest.json` and nineteenth snapshot `sources/pre95-operator`
preserve 176 measured inputs and both complete Nightwire records before the local
operator runtime change. Nineteen snapshots / thirty-two historical identities
stay immutable. See [operator evidence](operator-netbsd/README.md) for new results.

[Other BBSes directory software evidence](directory-netbsd/README.md) records
188 Rust tests per OS, eleven directory PTY controls per OS and twelve actual
SSH/native controls, with operator/backup/serial/art regressions. Snapshot 22
`sources/pre27-directory` freezes all 173 prior recovery inputs and its complete
record; 22 snapshots/35 historical identities remain unchanged.

[Native board-launcher evidence](board-launcher-netbsd/README.md) records 191 Rust
tests per OS, seven launcher PTY checks per OS and ten real two-board controls
with independent member approval and host keys. Snapshot 23 freezes 174 prior
directory inputs and its complete record. All 23 snapshots/36 historical record
identities remain immutable; no new server/250-capacity/hardening claim.

[Named-room browser software evidence](room-browser-netbsd/README.md) records
194 Rust tests per OS, ten local/worker controls per OS, eleven real SSH/native
controls with 100 rooms and 22 shared-reader regressions. Snapshot 24 preserves
178 prior sources/room-list fixtures and the whole launcher result; 24 snapshots/
37 historical record identities remain unchanged. No new capacity/hardening claim.

[Room writing desk software evidence](writing-desk-netbsd/README.md) covers the
optional local-room multiline writer, preview/send/cancel and actual SSH/native
routes. Snapshot 25 freezes 174 prior inputs and the complete #147 room-browser
record; 25 snapshots/38 historical identities remain unchanged. No new capacity,
physical SBC, persistent drafts or hardening claim.

[Full-screen/editor checkpoint evidence](full-screen-netbsd/README.md) records
213 Rust tests per OS, seven full editor/recovery PTYs per OS, eleven actual
SSH/native/plain controls and distinct-UID backup/export checks. Snapshot 26
freezes 180 prior inputs and the whole #148 record; 26 snapshots/39 historical
identities stay fixed. Schema 8 holds explicit local-room plaintext checkpoints;
no private-mail, new capacity or hardening claim follows.

[Local community wall software evidence](community-wall-netbsd/README.md) covers
224 Rust tests per OS, seven wall PTYs per OS, eleven actual SSH/native/plain
controls and twelve complete wall/file recovery checks per OS. Snapshot 27 freezes
192 prior measured/harness/compatibility inputs and the whole #149 record; all
forty historical identities remain unchanged. Schema 9 adds bounded local wall
state; no private-mail, capacity, physical SBC or hardening claim follows.

[Local member-page evidence](member-pages-netbsd/README.md) records 235 Rust tests
per OS, eight page PTYs per OS, fourteen actual SSH/native/plain controls and
thirteen complete member-page/wall/file recovery checks per OS. Snapshot 28 keeps
202 previous inputs and the whole wall result; forty-one old identities remain
fixed. No new capacity, physical SBC, federation or hardening claim.

The pre94 integration snapshots preserve 213 current member-page/client/harness
inputs and 93 original vault inputs, plus both whole original records. Thirty source
snapshots now preserve forty-three historical identities. Neither measurement is
a claim about the combined client; integration gets fresh software checks.

Current combined [client/vault software checks](client-vault-integration-netbsd/README.md)
keep the preceding two results immutable and use fresh current input pins.

[Personal catch-up software evidence](catchup-netbsd/README.md) records 280 Rust
per OS, eight local/worker PTYs per OS, eleven actual SSH/native/plain and sixteen
complete-recovery controls per OS. Snapshot 33 preserves 238 previous inputs and
the whole poll result; all 46 prior record identities remain unchanged. The new
record pins 240 matched native inputs and 21 host harnesses; guard 47 checks it.
No new capacity, mandatory visit telemetry or server-side private mail is claimed.

[Discussion discovery evidence](discussion-netbsd/README.md) covers 292 Rust tests
per OS, eight local/worker Find PTYs per OS, eleven actual SSH/native/plain and
seventeen complete-recovery controls per OS. Snapshot 34 preserves 248 prior
inputs and the whole catch-up result; 47 historical identities remain unchanged.
The new record pins 250 matched native inputs and 23 host harnesses; guard 48 checks
it. No new capacity, private-mail search or automatic query history is claimed.


[Curated gallery evidence](gallery-netbsd/README.md), issue #144, records 302 Rust
checks per OS, eight local/worker PTYs per OS, eleven actual SSH/native/plain
controls and seventeen existing recovery controls per OS. Snapshot 35 freezes
258 prior inputs/whole discussion record; 48 historical results remain exact.
The new record pins 269 matched native inputs/25 host harnesses/thirteen unchanged
inputs. Guard 49 checks it offline. Owned clone/cache removed, saved preview off;
no capacity, raw-art, public-publication or hardening claim.


[Native travel evidence](travel-netbsd/README.md), the first #159 increment, records
304 Rust tests per OS, 130 Python, six local travel PTYs per OS, nine actual
SSH/native/restricted-SFTP controls and seventeen existing recovery controls per OS.
Snapshot 36 preserves 277 prior inputs and the whole gallery record. New evidence
pins 272 matched native inputs/29 host harnesses/thirteen unchanged inputs; guard
50 checks it offline. Owned clone/cache removed, saved playground stopped/preserved.
No wire-budget/offline/route/throughput/physical-radio/capacity claim.


[Curated library evidence](library-netbsd/README.md), the first #140 increment,
records 313 Rust tests per OS, 130 Python, eight local/worker PTYs per OS, ten
actual SSH/native/SFTP and seventeen recovery controls per OS. Snapshot 37 freezes
282 prior actual inputs/whole travel record. The new record pins 282 matched native
inputs/31 host harnesses/thirteen unchanged inputs; guard 51 checks it offline.
Owned clone/cache removed, owner playground off/preserved. No new request desk,
transfer authority, public publication or capacity claim.


The thirty-eighth source snapshot `sources/pre140-file-requests` and
`library-baseline-manifest.json` freeze 292 previous library execution/harness
inputs and its whole original result. Fifty-one historical result identities
remain fixed. The current [file request record](file-requests-netbsd/README.md)
pins 288 matched native inputs, 321 Rust tests per OS, six request PTYs per OS,
eight actual shared SSH/native/SFTP and 18 recovery controls per OS. Guard 52 checks
its evidence and teardown offline. No new capacity, listener or transfer authority.


Snapshot 39, `sources/pre143-magazine` and `file-requests-baseline-manifest.json`,
preserves 298 previous request inputs and its complete original result. All 52 old
result identities remain unchanged. The current [magazine record](magazine-netbsd/README.md)
pins 294 matched native inputs, 327 Rust tests per OS, seven magazine PTYs per OS,
nine actual shared SSH/native/SFTP and eighteen recovery controls per OS. Guard 53
checks the reader-only scope, original edition and teardown offline. No new schema,
capacity, authenticated submission/consent or public-publication claim.


Snapshot 40 (`sources/pre143-editorial`) and the editorial manifest preserve 304
previous reader inputs and its whole result. Snapshot 41 (`sources/pre137-social`)
and the editorial baseline manifest preserve 310 editorial inputs/whole result.
Their [editorial](editorial-netbsd/README.md) and [chat lounge](social-netbsd/README.md)
records remain unchanged; guards 54/55 retain their original scope.

Snapshot 42 (`sources/pre137-boardwide`) and `social-baseline-manifest.json`
preserve 316 prior input files and the entire social result. All 55 historical
result identities remain exact. The current [shared visit lounge evidence](boardwide-social-netbsd/README.md)
adds availability and safe pages outside chat; legacy standalone adapters remain
chat-only. Guard 56 verifies its current native inputs, real idle/editor/Lua waits
and teardown independently. The saved owner playground stays off/intact; no new
capacity, physical hardware or hardening claim.


Snapshot 43 (`sources/pre137-standalone`) and the independently pinned
`boardwide-social-baseline-manifest.json` preserve 326 previous input files and
the complete boardwide record. All 56 historical identities stay unchanged.
The current [standalone lounge evidence](standalone-social-netbsd/README.md)
adds local/inherited-worker visit parity; guard 57 checks its registered/approved
PTYs and sustained menu/full-screen/plain waits. Shared runtime source identity
is retained separately from earlier shared live observations. Normal budgets,
schema 17 and the stopped saved playground remain; no new capacity/hardware claim.

Snapshot 44 (`sources/pre137-idle`) and the independently pinned
`standalone-social-baseline-manifest.json` preserve 329 prior input files and the
whole standalone record. All 57 historical identities remain exact. The current
[optional idle notification evidence](idle-notifications-netbsd/README.md) covers
explicit visit consent, safe idle ANSI/plain indications and protected draft/game/
probe input across local/worker/shared SSH/native access. Guard 58 checks current
source matches, real sustained regressions, recovery and owned-fixture teardown.
Schema/resource defaults remain; the saved playground stays stopped and intact.

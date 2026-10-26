# CI schema-oracle correction

The first [PR #221 repository run](https://github.com/gonz0w/faction-bbs/actions/runs/40325681281)
on `b45e0323c15a204ca7a8d2c757b7b5789bc51559` passed its optional mail,
documentation/archive, Lua, strict Clippy and Rust tests, then failed at the
playground seed's remaining schema-18 assertion. The same expectation remained
in the writing-desk PTY. Both were reproduced on the host before correction;
both failed specifically on their schema assertion.

Only those two expected values change to 19. All other content, approval,
password, overwrite/symlink, shared-parent, editor, reply and ledger assertions
remain. The seed check and all 13 local/worker writing-desk controls then passed
against the same schema-19 binaries. The source audit found no other current
schema-18 database assertions; historical/downgrade evidence stays unchanged.

| Harness | Observed red source | Observed green source |
| --- | --- | --- |
| smoke_playground_seed.py | [`99c8630987e5`](source-objects/99c8630987e5dc850672c92f1abe944d32aae06e74319fc5da68012ff498411a) | [`e7991c17782f`](source-objects/e7991c17782f24154836f12bddee8c2dfbc928d09ea34b4fece817dfebd3e668) |
| smoke_writing_desk.py | [`dbd82891c378`](source-objects/dbd82891c37865b553c7731a56b67e51cea804dbe86c264943341ffe94b1d178) | [`d0ee6f50e47e`](source-objects/d0ee6f50e47e96fd8ab6a9cbb61f53b559556daa292aff9795fec61b01ee076e) |

These are host/CI harness corrections, not new native execution. The original
qualification of 411 tests per OS and 705 native inputs and its complete pinned result stay
unchanged; they retain the exact old harness versions that were captured.
No product Rust, dependency, schema, service setting or test limit changes.
The saved owner playground remains stopped. This correction used a separate
marked synthetic host fixture, removed after source extraction and independent
review.

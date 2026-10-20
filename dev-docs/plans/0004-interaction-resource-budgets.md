# 0004: Small interactions and small hosts

Issues: [#8](https://github.com/gonz0w/faction-bbs/issues/8), [#28](https://github.com/gonz0w/faction-bbs/issues/28)
Status: research deliverables published in PR #38, pending review/merge. Budgets proposed; no production stack selected.

## Goal and constraints

Define falsifiable interaction and resource targets, measure a bounded ASCII/ANSI replay, and investigate compact implementation strategies. Preserve approved local membership, private/disabled HTTP, safe text fallback, isolated optional doors, no JS/TS, and local ownership. PR #37 was merged at `6df1c65294e1860fba135e306b39d7d303959a90`, closing #3.

Owner clarifications on 2026-10-19: 1–2 cores / about 1 GiB host RAM; 100–250 connected members; prioritize very small memory, scalable fast languages and selective assembly if justified. Historic 640K BBSes motivate efficiency; no literal modern host or security budget was specified. Explore unusual technology and optional public services/blockchain without weakening existing privacy/independence constraints.

## Progress

- [x] Read requirements, dependencies, and acceptance criteria; merge authorized PR #37.
- [x] Author and run bounded local experiments; preserve sanitized results.
- [x] Record proposed budgets, evidence, tradeoffs, unknowns, and follow-on tests.
- [x] Update docs, validate, publish a reviewable PR and issue evidence.

## Steps and commands

Work from repository root. Use Python stdlib for measurement tooling, not a product language choice. The text replay uses local socket pairs with application pacing/delay; it does not emulate TCP, crypto, loss, or a terminal. The C allocation probe compares two buffer ownership strategies, not languages or complete servers. Compile outside the checkout; no binary artifacts or actual user/network data.

## Validation and outcomes

- `python3 tools/check_repository.py`: passed, including archive hashes and current links/policy.
- `python3 tools/model_link_budget.py --check`: passed; #3 model unchanged.
- `python3 tools/measure_text_replay.py --run`: 120 exchanges, actual bytes/fallback/caps checked; raw observations and generated report saved. Repeated after adding raw-observation validation to keep source provenance exact. These are application-paced socket-pair observations, not real network/terminal/server performance. Sender rate undershot configured controls in some short cases; report includes actual calibration and scheduling limitations.
- `python3 tools/measure_text_replay.py --check`: passed; source SHA-256, all workload cells, positive finite timings, received-byte counts and generated report match. No timing threshold asserted in CI.
- `python3 tools/measure_buffer_probe.py --run`: C11 compilation with all warnings treated as errors succeeded; five fresh processes per strategy, identical delivery checksums. Requested allocations: 16,386,000 versus 577,536 bytes. No RSS or server claim.
- `python3 tools/measure_buffer_probe.py --check`: passed, including source provenance and saved allocation/checksum consistency; compiler not required by CI.
- `python3 -m unittest discover -s tools -p 'test_*.py'`: 19 tests passed. Added tests cover viewport/byte caps, fallback equivalence, continuation content and corrupt/missing/duplicate/non-finite observations.
- `git diff --check`: passed.

No kernel shaper, real terminal client or constrained SBC was available/exercised in this session. No auth, durable storage, 250-user server, door, public service or blockchain integration was tested. Exact follow-on methods are specified, not reported as results.

| Acceptance criterion | Evidence |
| --- | --- |
| #8 minimum tasks, payloads, pagination, encodings/reconnect | 0008 report: J1–J8 caps, T0/T1/T2, row/byte limits, backpressure and reconciliation oracles. |
| #8 measured ASCII/ANSI with readable fallback | Local paced replay with calibrated actual sender rates, injected delay, 120 received-byte observations and fallback/cap tests. Real packet/terminal tests explicitly deferred to candidate integration. |
| #8 optional ANSI/game and private/disabled portal boundaries | 0008 alternatives and separate rich-art/door/no-JS portal validation. |
| #28 workloads, budgets, connections, growth, component count | 0028 owner envelope, C0/C100/C250 controls, tiny full-charge targets, payload ledger, storage/growth and service counts. |
| #28 reproducible read/post/chat/render/storage/door measurements | 0028 measurement contract, failure oracles, required environments/repetitions and publication fields. Full-server harness remains later work. |
| #28 comparison criteria/security/portability/deployment tradeoffs | 0028 mechanisms/source table, gate states, proposed anchors and bounded allocation evidence; no language/storage/provider selected. |

## Publication and handoff

[PR #38](https://github.com/gonz0w/faction-bbs/pull/38) delivers this research and closes #8/#28 on merge. Both issues now contain criterion evidence and explicit experiment limitations; #4 records the owner clarification and resource/technology inputs without adopting a language.

The required `Documentation and policy` check passed for research commit `b2d9dd3564c4d2d0c4ad8b23eea0064c5890b4ec` in [GitHub CI](https://github.com/gonz0w/faction-bbs/actions/runs/39351534629). This publication-only revision uses the same required check; final status is tracked in the PR. PR #38 remains open for review, not authorized for automatic merge by the instruction to merge #37. Issues stay open until merge; numeric budgets remain proposed research inputs. Next runtime/storage and full-server prototype work is in the existing linked issues.

# 0003: Research constrained-network user journeys

Issue: [#3 / BBS-003](https://github.com/gonz0w/faction-bbs/issues/3)
Status: research deliverables merged in PR #37; issue #3 closed; numeric controls remain proposed research inputs

## Goal and scope

After owner-authorized merge of PR #36, define representative approved-session, room, posting, chat, file, door, and reconnect journeys with measurable link/terminal/payload assumptions. Keep production budgets, protocol/rendering choices, browser feasibility, hardware/runtime measurements, and future mesh implementation in their existing issues.

## Progress

- [x] Verify PR #36 merged and issue #1 closed; start from updated `main`.
- [x] Read current requirements, framework, and related issues #8, #21, and #28.
- [x] Obtain owner clarification: mainly modern private networks with occasional poor connections.
- [x] Retrieve primary Citadel, netem, and RFC 4838 references.
- [x] Write eight journeys, three synthetic profiles, twelve workload traces, proposed observations, and a reproducible analytical model.
- [x] Validate the model, generated results, current docs, and archive integrity.
- [x] Publish [PR #37](https://github.com/gonz0w/faction-bbs/pull/37) and verify CI; leave new research adoption to review.

## Decisions and limits — 2026-10-19

- PR #36 was merged at `2b23e39`; owner instruction accepts the decision framework, not any runtime authorization hypothesis. Update its adoption status as a follow-through change.
- The primary journey target is modern private networks; degraded/interrupted profiles test graceful behavior. Retro radio/mesh is not the minimum initial interactive baseline.
- Numeric rates, fixture sizes, and impairments below are researcher-defined synthetic controls, not measured user-network characteristics or production SLOs.
- Python is repository research tooling only. No product stack, access protocol, credential mechanism, renderer, or dependency is selected.

## Validation

Run the payload model and its focused correctness tests; make the generated report reproducible in CI. Run repository checks and the existing validator tests. Review all three issue criteria and ensure modeled serialization is never reported as runtime/security/network performance. Record actual results and PR handoff below.

## Acceptance coverage and local results — 2026-10-19

| Issue #3 criterion | Delivered evidence |
| --- | --- |
| Approved-login, room catch-up/posting, chat, files, doors, reconnect journeys | J1–J8 with state boundaries, functional oracles, degraded behavior, and checkpointed interruption cases. |
| Measurable payload, latency, interruption, and terminal assumptions | P1/P2/P3 controls, T0/T1/T2 capabilities, synthetic workload JSON, counted byte/round-trip model, generated report, and explicit measurement plan. All controls are proposed research inputs, not production targets. |
| Separate optional/private HTTP and future mesh from the minimum terminal experience | Baseline with HTTP disabled; private no-JS portal and mesh/Reticulum/shared-chat research remain their own issues. |

- `python3 tools/check_repository.py`: passed; current links, policy, issue provenance, and original archive hashes verified.
- `python3 tools/model_link_budget.py --check`: passed; generated table matches fixture byte counts, nominal-rate/RTT arithmetic, and fixture SHA-256.
- `python3 -m unittest discover -s tools -p 'test_*.py'`: all 14 tests passed (six existing repository tests plus eight focused model tests).
- `git diff --check`: passed. Reviewed bit/byte and KiB/MiB units, CRLF/ANSI byte counts, sequential versus full-duplex assumptions, one-/four-turn comparison, and P3 connected-only labeling.
- No real networking, traffic shaper, authentication/session prototype, terminal renderer, game, or file transfer was executed. No production performance/security result or target support is established.

Remaining publication/review state is tracked in the linked issue/PR; no following issue has been implemented by this work.

## Publication and handoff

[PR #37](https://github.com/gonz0w/faction-bbs/pull/37) delivers all three criteria and closes issue #3 when merged. The required `Documentation and policy` check passed for research commit `f4db7c2` in [GitHub CI](https://github.com/gonz0w/faction-bbs/actions/runs/39349385396). The handoff-only revision runs the same workflow; final status is tracked in the PR. Issue #3 remains open pending merge, and the new profile/workload controls remain proposed inputs.

PR #36 is merged and issue #1 closed. The process is accepted; no runtime architecture was adopted. Next bounded work should establish #8 interaction budgets and #28 hardware/resource workloads from these inputs, with actual measurements and security constraints; those issues are not completed by this report.

## Merge follow-through — 2026-10-19

Owner-authorized PR #37 merged at `6df1c65294e1860fba135e306b39d7d303959a90`; issue #3 is closed. Earlier publication notes above record the state at handoff. #8/#28 now continue in [plan 0004](0004-interaction-resource-budgets.md); no production protocol/runtime or support claim was adopted by the journey merge.

# 0003: Research constrained-network user journeys

Issue: [#3 / BBS-003](https://github.com/gonz0w/faction-bbs/issues/3)
Status: in progress

## Goal and scope

After owner-authorized merge of PR #36, define representative approved-session, room, posting, chat, file, door, and reconnect journeys with measurable link/terminal/payload assumptions. Keep production budgets, protocol/rendering choices, browser feasibility, hardware/runtime measurements, and future mesh implementation in their existing issues.

## Progress

- [x] Verify PR #36 merged and issue #1 closed; start from updated `main`.
- [x] Read current requirements, framework, and related issues #8, #21, and #28.
- [x] Obtain owner clarification: mainly modern private networks with occasional poor connections.
- [x] Retrieve primary Citadel, netem, and RFC 4838 references.
- [x] Write eight journeys, three synthetic profiles, twelve workload traces, proposed observations, and a reproducible analytical model.
- [x] Validate the model, generated results, current docs, and archive integrity.
- [ ] Publish an issue-linked PR and verify CI; leave new research adoption to review.

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

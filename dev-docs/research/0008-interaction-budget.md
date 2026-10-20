# 0008: Useful text within a small interaction budget

Status: research deliverable complete; proposed budgets, no adopted client protocol.
Date: 2026-10-19. Issue: [#8 / BBS-008](https://github.com/gonz0w/faction-bbs/issues/8).
Owner: Codex research; founder/maintainer reviews adoption through PR.
Decision affected: terminal/client contract, pending #13/#21/#24.
Stopping condition: bounded tasks/caps, actual ASCII/ANSI comparison with fallback, explicit remaining compatibility tests. Follow the [decision framework](decision-framework.md).

## Question and constraints

Can a useful approved BBS session retain ANSI identity while avoiding large screens, excess round trips, and unbounded queues? Use [#3 journeys and P1/P2/P3 controls](0003-constrained-user-journeys.md): primarily modern private networks, occasionally poor/interrupted ones. This report defines *proposed test targets*, not support promises. It selects neither SSH/Telnet nor an application framing protocol. Authentication, approval, safe terminal processing, and persistence are still unimplemented.

## Proposed baseline and budgets

Counts below are encoded application bytes, including CRLF and authored escapes, before any transport/framing/encryption overhead. Future tests must report both application and on-wire bytes. KiB = 1024 bytes. Caps apply to text interactions; files and optional games have separate policies.

| Task from #3 | Proposed test input/output limit | Interaction and degraded behavior |
| --- | --- | --- |
| J1 registration/pending | A prompt/status response <=1 KiB; ordinary command line <=1 KiB | Show registration status only before approval; no room/chat/file metadata. Secret input has separate echo/retention rules; do not infer a credential format from this cap. |
| J2 identity/welcome/menu | Compact approved identity + welcome/menu <=2 KiB | Preserve this BBS's identity before menu. Full ANSI art is optional above this cap; permit compact mode and skip. Revalidate approval before content. |
| J3 discover/enter/read rooms | Each page <=2 KiB and <=visible-row budget | List rooms and read new/post history incrementally. Split long messages into continuations; count displayed portions before advancing unread state. No fixed messages/page. |
| J4 post/reply | Draft body <=16 KiB for initial experiments; outcome <=512 bytes | Allocate drafts only while editing; avoid per-keystroke network round trips. One submit/outcome turn where possible; uncertain outcome triggers reconciliation before retry. Draft limit is proposed, not a permanent product restriction. |
| J5 chat | Ordinary rendered record <=512 bytes, test nominal 256 bytes | Command help, join/leave local channels, send, receive and distinguish gaps/history from live traffic. Long content requires bounded continuation or explicit rejection. |
| J6 files/mail/directory | Metadata page <=2 KiB | Show authorized metadata before optional transfer/handoff. Transfers stream bounded chunks; never buffer whole files per member. Mail/directory use the same paging requirements, not tested implementations. |
| J7 doors | Handoff/status <=1 KiB | Capability/resource check before launch, return safely to menu after failure. Door output is outside the basic text envelope; unsupported door is unavailable, not broken community access. |
| J8 reconnect | Resume/status page <=2 KiB | Reauthenticate as needed and check current local approval and resource access. Resume at safe content position; a connection is not durable authorization. |

For P2 (64,000 bit/s each direction, nominal RTT 400 ms), a 2 KiB response plus a 1 KiB request consumes 384 ms of serialization + 400 ms nominal RTT before framing/server/impairment costs. Proposed end-to-end test goal is p95 <=1 second for a normal page after authorization, with server work separately budgeted in [#28](0028-small-host-budgets.md). This leaves little headroom for repeated prompts. The prior [analytical model](0003-link-budget-results.md) shows four-turn submission costs; those are not measured transport performance. At most one page is sent until requested in plain mode.

### Viewports and encoding

- **T0 plain:** seven-bit printable ASCII, CRLF, line commands, no cursor-addressing prerequisite. Default test viewport 80x24; ask/allow manual size and plain mode without relying on terminal probing. Test backspace/delete, Enter and fragmented input before protocol adoption.
- **T1 minimal ANSI:** opt-in/confirmed clear-display, cursor-home and a small SGR color/reset subset. Essential text/actions remain identical with color removed. Unknown capabilities fall back to T0. Identity artwork needs its own validated bounded profile; this fixture is not an ANSI art engine.
- **T2 narrow:** 40x20 reference. Wrap below the final column to avoid incidental autowrap; reserve four rows for status/footer/cursor. Thus body capacity is 20 rows at 80x24 and 16 at 40x20; further prompts must fit those reservations. Resize restarts from a logical continuation boundary, not stale screen offsets.
- Keep stored content intact. ASCII presentation must visibly escape/substitute unsupported characters according to a documented policy; never silently rewrite originals. UTF-8, combining/wide characters and CP437 door isolation are separate compatibility profiles requiring #13 tests. No Unicode safety or CP437 compatibility was measured here.
- Untrusted member/file/remote-channel text must not acquire authored ANSI privileges. The fixture strips only its known, authored sequences to compare content; its regex is **not** a production sanitizer. Escape injection, control strings, malformed input and terminal query handling require negative tests in #13/#19/#21.

This row-based approach corrects a fixture limitation in #3: five messages are not guaranteed to fit 24 rows. Message length and viewport determine pagination. Partial reading must remain visible as a continuation rather than falsely marking an entire message read; exact durable cursor semantics belong to #12/#21.

### Backpressure and interruption

Proposed per-session application output cap: **8 KiB**, including queued/shared payload retained on behalf of that session; reserve 1 KiB within it for status/control. Input command accumulator <=1 KiB, drafts separately limited above. A shared immutable pool also needs a **global** cap and bounded reference lifetime; per-session counts alone do not bound total retained bytes.

At P2 nominal rate 8 KiB takes 1.024 seconds to serialize. Test a two-second oldest-queued-record age threshold as well as the byte cap, so low-rate stalls cannot accumulate forever. When exceeded: stop optional artwork/transfers, pause catch-up, and explicitly report missed chat/history or disconnect safely if no control progress is possible. Never silently claim complete chat delivery or successful posting. Keep authorization checks and request outcome reconciliation active during resume.

P3 adds a ten-second outage or forced close at each journey checkpoint. Required follow-on oracles: bounded memory, no stale approval after revocation, no duplicate post after uncertain acknowledgement, no lost unread cursor, and no peer stalled by a slow consumer. These recovery cases are specified, **not executed** by the pacing replay.

## Alternatives, gates and criteria

| Approach | Tradeoff and present evidence |
| --- | --- |
| Row/byte-bounded ASCII pages + optional minimal ANSI | Authored fixtures preserve text and meet caps; same task can use either presentation. Proposed baseline. |
| Full-screen rich ANSI as a prerequisite | Reject as baseline under G7: useful plain text must remain. Keep rich artwork/games optional, with separate capability/size tests. |
| Command prompts for every field/key | Easy interaction model but repeated RTT costs; batch commands/drafts when safe. No editing protocol chosen. |
| Mandatory browser console/public portal | Reject under G1/G4 if it requires JS or public HTTP; private/disabled portal must leave terminal tasks available. Browser feasibility stays #24. |

Scoped artifact gates: G1 **pass** (Python stdlib, authored assets); G2 **unknown** (no login/action server); G3/G4/G5/G6 **pass for experiment design only** (synthetic local data, local socket pairs, no service, telemetry or domain addressing); G7 **unknown overall**, authored content fallback passes but real terminal/identity/resize behavior remains untested; G8 **not applicable** (no extension executed); G9 **pass** (repository-owned fixture/code under BSD-3-Clause). These do not establish any production gate as passed.

Use framework S25/O20/R15/T15/M10/P10/X5 for a later client choice. Current comparative metrics are bytes/page, required turns, row fit, fallback content equality and measured replay time; remaining safety/compatibility/operations are **U**. No score/total winner is justified. First declare per-metric 0–4 anchors against the proposed caps and real terminal/security oracles before ranking; vary T/R weights after collecting that evidence. This report provides inputs rather than retrospectively scoring a preferred protocol.

## Experiment and actual results

Uncertainty: how much an authored minimal ANSI page adds versus equivalent ASCII, under two explicit rate/delay controls. Before replay, success meant all pages <=2 KiB, row/column fit at both viewports, byte-identical text after removal of the authored control subset, and exact received bytes. Fixture checks also reject unsupported sizes/cases and verify every message identifier survives continuation. Failure means revise the fixture/budget before any protocol recommendation.

Run from repository root:

```sh
python3 -m unittest discover -s tools -p 'test_*.py'
python3 tools/measure_text_replay.py --run
python3 tools/measure_text_replay.py --check
```

[Measured report](0008-replay-results.md) and [raw observations](results/008-text-replay.json) capture 120 exchanges: five repetitions, two profiles, two sizes, three cases, two presentations. Tool source SHA-256 in raw data identifies the exact source; sanitized OS/architecture/Python only, no hostname/member content/endpoints. Sockets/threads close per exchange. Outputs are repo research fixtures, not production data.

Controls: P1 at 1,000,000 bit/s and 40 ms injected RTT; P2 at 64,000 bit/s and 400 ms. These reuse the nominal #3 controls **without** jitter, loss or P3 outages. Application writes are paced cumulatively in <=128-byte chunks through local socket pairs, and one nominal RTT is injected after the request. Measured sender rate is recorded as calibration; this does not reproduce kernel packet queues, TCP, encryption or terminal rendering. No `tc`/SBC was available; no privileged network settings changed.

All fixture/content/cap observations passed. Minimal ANSI adds **18 bytes/page**, analytically 2.25 ms serialization at P2. Actual first-page medians for the 80x24 room were **549.14 ms ASCII / 553.48 ms ANSI**; differences vary between cells and overlap scheduling variation; no reliable speed advantage is established. The 80x24 room used two pages; 40x20 used three. Only the **first page** was timed, not full catch-up or concurrent use. Poor-profile measured sender rates span roughly 53.4–63.6 kbit/s, below the configured 64 kbit/s due to application scheduling. Rich 64 KiB ANSI artwork remains about 8.2 seconds of nominal serialization at P2, from #3 arithmetic, not this measured fixture.

## Evidence and confidence

| Claim | Kind / source, retrieved 2026-10-19 | Limits and confidence |
| --- | --- | --- |
| Terminal-first, approved members, optional/private HTTP, no JS | Owner; [product requirements](../../docs/project.md) | High as intent; unimplemented behavior. |
| Small authored ANSI subset and plain content can share tasks within proposed caps | Measured; source/raw/report above | High for these fixtures; low for arbitrary terminals, user controls or Unicode. |
| Clear/home/SGR are documented terminal controls | Primary; [xterm maintainer control sequences, CSI/SGR](https://invisible-island.net/xterm/ctlseqs/ctlseqs.html), moving reference | Documented semantics only; no universal emulator compatibility claim. |
| Repeated turns/large art cost delay | Analytical #3 and measured bounded replay | Moderate direction; full transport costs unknown. |
| Real network impairments need a different experiment | Primary; [netem manual](https://man7.org/linux/man-pages/man8/tc-netem.8.html), rate/delay/loss limitations | Documented, not run. Kernel pacing/placement effects must be calibrated. |

## Conclusion and next work

**Propose** bounded text-first pages with optional ANSI presentation, row-aware continuation, explicit queue caps and outcome-aware reconnect. Adoption authority is the maintainer; numbers remain researcher-proposed even if the report merges. Reversal is low now (fixtures/docs only), moderate once clients/read cursors depend on a contract, high if uncertain posting outcomes become persisted duplicates.

Weakest critical evidence: secure real-terminal behavior and reconnect are unknown. #13/#21 must run positive/negative terminal and permission cases; #11/#20 must replay full journeys through a candidate server using calibrated real link shaping and two terminal implementations. Measure on-wire bytes/latencies, loss/outage recovery, resize/encoding, menu/art skip, and HTTP-disabled operation. #24 independently investigates no-JS browser feasibility; #34 validates per-game profiles. Reopen caps if real secure framing, sysop artwork or accessibility evidence shows useful tasks cannot fit. No client/protocol is selected here.

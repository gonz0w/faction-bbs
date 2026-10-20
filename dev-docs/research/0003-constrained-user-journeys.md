# User journeys on private, poor, and interrupted links

Status: completed scenario and analytical-model research; proposed test inputs, no runtime experiment or support claim.
Date: 2026-10-19
Issue: [#3 / BBS-003](https://github.com/gonz0w/faction-bbs/issues/3)
Owner: project founder/authorized maintainer reviews the proposed research inputs.
Method: [accepted decision framework](decision-framework.md)
Stopping condition: define repeatable journeys, link/terminal/payload assumptions, and falsifiable follow-on experiments without selecting a protocol, language, renderer, or production SLO.

## Question and scope

What must members be able to do when a primarily modern private-network BBS encounters a poor or interrupted connection? The founder clarified on 2026-10-19: **“Mainly modern private networks, with occasional poor connections.”** This sets the initial focus. It does not define bandwidth, hardware, a specific access protocol, or a minimum supported link.

The [product](../../docs/project.md) and [session](../../docs/session.md) require local approval, BBS identity art before the menu, Citadel-style rooms, IRC-style chat, local files/mail, and optional doors. This report turns those requirements into research scenarios. It does not complete [#8](https://github.com/gonz0w/faction-bbs/issues/8)'s final interaction/encoding contract, [#21](https://github.com/gonz0w/faction-bbs/issues/21)'s full screen/state design, or [#28](https://github.com/gonz0w/faction-bbs/issues/28)'s hardware/resource budgets.

## Evidence and confidence

| Evidence, retrieved/reviewed 2026-10-19 | Supported claim and limitation | Confidence |
| --- | --- | --- |
| Founder clarification above and current product/security docs; starting revision `2b23e39` | Modern private-network focus, local approval, BBS flow, local content, optional HTTP, and chat-only future federation are owner requirements. No runtime behavior is demonstrated. | High for intent. |
| [Citadel room/message documentation](https://citadel.org/room_and_message_commands.html), GOTO / MSGS / SEEN sections | Room entry, new-message retrieval, and read state are useful reference behaviors. No adoption of its protocol, schema, or all UI semantics follows. | High for documented behavior, moderate for its transfer to our still-unspecified room experience. |
| [tc-netem manual](https://man7.org/linux/man-pages/man8/tc-netem.8.html), OPTIONS / LIMITATIONS | Rate, delay, jitter, loss, and seed controls support a future isolated impairment experiment. Kernel timing and placement affect results. It has not been run here. | High for documented tool capabilities; unknown for Faction integration. |
| [RFC 4838](https://www.rfc-editor.org/rfc/rfc4838.html), informational DTN architecture | Intermittent end-to-end availability warrants explicitly different delivery assumptions. No bundle protocol or mesh support is selected. | High for this distinction, unknown for any future transport fit. |
| [Synthetic fixture](fixtures/003-workloads.json), [model](../../tools/model_link_budget.py), [generated results](0003-link-budget-results.md) | Exact fixture-byte counting and arithmetic under stated assumptions. Sized blocks and profile numbers are research controls, not captured users, artwork, games, traffic, or deployment measurements. | High for checked arithmetic; low/unknown for predicting a future implementation. |

The weakest decision-critical evidence is actual end-to-end behavior: no BBS runtime, secure-session handshake, terminal renderer, transfer protocol, or disconnect-recovery implementation exists to measure. Modeling cannot establish their feasibility, security, or performance.

## Synthetic operating profiles

These values are **proposed experimental controls**, chosen to distinguish normal operation from degradation. They are not asserted to represent the distribution of private networks, modem speeds, radio payload rates, product requirements, or measured user links. P1 is a deliberately throttled private-network control, not a claim that modern links are only 1 Mbit/s. Recalibrate with owner workloads and real measurements in #8/#28 before adopting production budgets.

| Profile | Nominal capacity, each direction | Nominal end-to-end RTT | Future one-way impairment control | Purpose |
| --- | ---: | ---: | --- | --- |
| P1-private | 1,000,000 bit/s | 40 ms | Uniform jitter parameter 5 ms; no injected loss | Interactive reference with HTTP disabled. |
| P2-poor | 64,000 bit/s | 400 ms | Uniform jitter parameter 50 ms; independent random loss 1% per direction | Slower/variable connection, including costly extra prompt exchanges. |
| P3-interrupted | P2 while connected | P2 while connected | P2 plus one scheduled 10-second outage; also force a session close at the same logical point in a separate run | Distinguish temporarily stalled transport from session loss and ambiguous outcomes. |

For later network experiments, configure both directions and verify actual observed capacity/RTT, rather than adding a full target RTT independently to both legs. Record unshaped baseline, actual shaper/kernel/version/placement, queue limits, packetization, and random seed. Suggested comparative seeds are 7, 23, and 101; they are reproducibility controls, not a representative reliability distribution. Use an isolated test environment, never the operator's live interface. No shaper commands or host network changes were executed for this research.

The Python model uses only nominal rates and RTT. Jitter, loss, outage, handshake, compression, encryption/framing, retransmission, queuing, server work, and rendering are **not simulated**. P3's numerical table shows only its connected portion. Do not add ten seconds to it and call that a recovery benchmark.

## Terminal and workload controls

| Control | Proposed experimental assumption |
| --- | --- |
| T0 plain text | 80 columns × 24 rows, single-byte ASCII fixture text, CRLF line endings, line-oriented input. This is a compatibility probe, not a final encoding/protocol contract. |
| T1 minimal ANSI | Same dimensions with explicitly supported clear/home/color sequences in purpose-written identity art. Compare against T0's readable identity equivalent; unsupported control sequences need safe handling in #29. No CP437, Unicode width, or full ANSI support is presumed. |
| T2 narrow/resized | 40 × 20 and a resize during room reading/compose. Test wrapping, navigation, and visibility without changing stored content; exact terminal behavior belongs in #13. |
| Accounts/data | Synthetic pending, approved, rejected, and revoked local members; one local room, one local-only chat channel, one local file area, one isolated door fixture. No real credentials, users, endpoints, or historical game/art assets. |
| Room load | Ten messages; five per synthetic page; 256 single-byte body characters per message plus headers/CRLF. This controls the experiment, not a content limit or average-user claim. |
| Posting | A 512-byte synthetic body. Compare one submission/confirmation exchange with a four-prompt exchange to isolate round-trip cost; input labels are not adopted commands or a wire format. |
| Chat | A small line/echo; a burst of 32 records at 256 bytes; separate arrival streams of 1 or 32 such records/s. These are workload contrasts, not community-size estimates. |
| Files/art/doors | 64 KiB and 1 MiB file-body stand-ins, optional 64 KiB artwork stand-in, and three 80 × 24 single-byte screen stand-ins. KiB = 1,024 bytes; MiB = 1,048,576 bytes. Actual encodings/protocols add or alter bytes. |

Actual public fixture text contains no authentication secret. Authentication is an abstract prerequisite, not a mock password mechanism. The welcome/reconnect traces start **after** fresh authentication and approval; real connection and authentication cost must be measured separately and never optimized by weakening security.

## Journeys and observations

The observations below are proposed test oracles. No row reports a passed runtime test. Numeric fixture sizes are experimental inputs; #8/#28 must establish and validate actual task/resource budgets.

| ID / actor and goal | Sequence and boundary | Proposed success / degradation observations |
| --- | --- | --- |
| J1 — Applicant requests entry | Connect, choose registration, submit required local information, receive pending status; reconnect later to check status. | No room/chat/file/mail/door/directory data before local approval. Show a bounded useful status response; operator approval time is human time, not network latency. Lost submission acknowledgement must not silently create repeated requests. Required fields/rate limits remain #6/#30. |
| J2 — Approved member enters this BBS | Authenticate and verify current approval; show BBS identity art, welcome/quote, then menu. | Preserve the identity-before-menu order. Compare compact identity with opt-in larger art; record bytes and time to first visible feedback and input-ready menu. Investigate skip/fallback so large art does not indefinitely obstruct the menu; policy is not finalized here. |
| J3 — Member catches up in a room | List accessible rooms/new counts; enter one; read ten new messages through two controlled pages; leave/return. | Enforce room permissions. Compare bounded pages with full-history/full-screen replay as a stress alternative. New/read position must not silently advance over unseen content; define the actual acknowledgement point in #12. Keep private counts/content out of unauthorized views. |
| J4 — Member composes and posts | Compose a 512-byte body; submit one logical intent; receive or reconcile the result. | Do not echo a second committed post merely because acknowledgement was lost. Preserve an explicit uncertain outcome until reconciliation is available; an unsent/unsaved draft is a separate case. Draft storage/privacy, durable intent identity, commit semantics, and retry protocol remain #5/#6/#9/#12. |
| J5 — Member uses local live chat | Enter channel, issue illustrative commands, send/read lines, return to menu; apply quiet and busy streams. | No local-only channel traffic leaves the BBS. Measure lag and backlog; compare incremental lines with repeated full screens. Congestion must not masquerade as current live state. Bound queues, disclose truncation/gaps, and allow leaving/rejoining; exact backpressure/history policies remain #8/#26. |
| J6 — Member discovers/transfers a local file | Read metadata first, request an authorized body, monitor progress, cancel or resume/restart deliberately. | Room/menu navigation must not require downloading the body. Partial files must not appear as complete. Verify content equality after completed transfer; if resume is unavailable, disclose restart and ask before repeating a large transfer. Transfer/multiplexing/integrity/access mechanisms remain #22. |
| J7 — Member enters an optional door | Authorize entry, hand off a bounded session, exchange illustrative screens, exit/return to BBS. | A failed or disconnected door must not terminate the core, leak another session, or monopolize resources. Show an explicit interrupted state on return; original-game save/resume behavior is per-title, not guaranteed. Runtime/isolation/lifecycle tests remain #10/#18/#34. |
| J8 — Member reconnects after an interruption | Restore or replace the connection, reauthenticate/recheck approval as needed, re-establish safe terminal state, request a bounded room/submission/transfer summary. | Reconcile committed versus uncommitted actions before retry. No bulk replay of private history, blind resubmission, or carry-over approval after revocation. An unresolved outcome is shown as unresolved. Reconnect alone grants no remote membership or channel federation. |

File/mail/door capabilities remain candidate scope for the initial useful slice, not implied prerequisites for J2/J3. Mail, directory visits, and sysop administration need later dedicated designs; their access boundaries still apply to these tests.

## Interruption matrix and persistence boundaries

Apply each applicable case both as P3's temporary outage and as a forced close. Put the event at a logical checkpoint, not an arbitrary wall-clock delay, so comparisons act on the same state. Record whether a submitted action reached the server, crossed its defined commit point, and returned an acknowledgement. A new session must use current local authority.

| Checkpoint | Observation to test / unknown to resolve |
| --- | --- |
| Registration accepted, acknowledgement not delivered | Reconcile the existing request rather than claim failure/success from silence; define retry identity in #6. |
| Room page partially delivered | Define read-state semantics; do not mark unsent/unseen messages read merely because a page was queued. |
| Draft not submitted or durably saved | Disclose possible loss; do not promise recovery or silently store private drafts. #9 defines any opt-in/private draft mechanism. |
| Post committed, acknowledgement not delivered | A retry/recovery query must establish the original outcome without a duplicate logical post. Mechanism remains unselected. |
| Post not committed | Show or recover the pending draft only under the designed local privacy policy; no fabricated commit acknowledgement. |
| File at 50% of body | Keep incomplete data distinct, offer supported resume/restart behavior, and verify completed output. |
| Door active | Clean up or retain session under bounded explicit policy; no cross-user reuse. Resource/timeout targets remain #18/#28. |
| Approval revoked while disconnected | Deny subsequent community actions; no stale cached approval grants access on reconnect. Race/commit semantics remain #6/#30. |

“Asynchronous discussion” means members need not be present together. It does **not** establish offline client caching, disconnected authentication, automatic draft sync, or federated boards. When a link is unavailable, keep truthful local status and recover under explicit semantics once the BBS is reachable. A standalone BBS may continue serving locally reachable approved members without an Internet/federation service.

## Analytical results and architectural implications

The [generated table](0003-link-budget-results.md) is reproducible with the checked-in fixture and Python standard library. It counts textual/control bytes and sized blocks, then models sequential application exchanges. The values below are rounded **model results**, not Faction response times:

- The compact post-approval identity/menu fixture is 201 downlink bytes; its P2 modeled exchange is about 0.428 s. A hypothetical 64 KiB artwork block plus menu becomes about 8.607 s under the same controls. This motivates testing a compact identity path without removing BBS identity or rich-art options.
- Ten fixture room messages in two pages plus discovery total 3,067 downlink bytes and three exchanges, modeled at about 1.588 s on P2 before human reading and omitted overhead. Actual long posts, rendering and read-state semantics need measurements.
- The 512-byte submission examples model about 0.473 s for one exchange and 1.679 s for four on P2. They illustrate sensitivity to application round trips, not a recommendation to replace a secure terminal protocol or an assumption that each keystroke waits one RTT.
- A 1 MiB file body plus the fixture metadata models about 131.480 s on P2; the bare body alone requires 131.072 s of serialization at 64,000 bit/s. Metadata-first navigation and cancellable optional transfers deserve explicit tests.
- A hypothetical stream of 32 × 256-byte records/s is 65,536 payload bit/s: more than P2's entire 64,000 bit/s downlink even before framing. No buffer can make that stream indefinitely live at that capacity. Queue limits/history/gap policy must be designed and tested; the model does not simulate them.

No numeric winner or production SLO is adopted. The supported inference is narrower: optional bulk output, extra sequential exchanges, and sustained arrival load can dominate an impaired connection even when tiny text actions fit comfortably in the nominal arithmetic. Real secure-session/protocol overhead and failure behavior could change the comparison.

## Gate screen, alternatives, and reversal

G1–G9 remain unchanged. No JS/TS dependency, mandatory HTTP/central service, content federation, unapproved community access, or executable game/source is introduced. Passing a repository/model check cannot establish runtime approval, privacy, terminal safety, isolation, recovery, or compatibility. Those gates remain unknown for any eventual implementation.

Compare three interaction hypotheses in #8/#13: (A) rich/full redraw and chat-first output everywhere, (B) bounded text tasks plus optional ANSI/bulk features, (C) offline/store-and-forward as a mandatory initial experience. B is a useful prototype candidate; A needs measured poor-link behavior and fallback; C assumes extra client/cache/delivery behavior beyond the initial private-network requirement. None is selected as an implementation here. A future private portal may share the journeys only if #24 resolves its no-JS feasibility and preserves approval.

Reversal cost is low for these document/fixture controls: no persisted user data, wire format, or shipped runtime changes. Changing a future protocol/schema or session recovery policy could be expensive and needs a separate reversal analysis. Do not treat these scenario IDs or synthetic command labels as a permanent public API.

## Falsifiable follow-on experiment and handoff

Uncertainty: can a candidate terminal/session approach preserve the required journeys under P1/P2/P3 with bounded output and truthful recovery? After the relevant identity, terminal and storage designs exist, run one vertical-session harness with synthetic data, T0/T1/T2, the profiles above, and controlled checkpoints. Use the same fixture content and authentication/authorization policy for each alternative; collect only counts/timings and sanitized synthetic results.

Record connection/authentication, first feedback, and input-ready times separately. For each action record application bytes, actual wire bytes where observable, round trips, returned records, read position, committed intents/posts, partial/completed files, and retained door resources. Human think/approval time is excluded from service latency and recorded separately when relevant. Compare cold versus warm sessions. Report sample count, failures, median/p95/max with the percentile method; start with a small debugging trace, then use a declared repetition count/seed set before publishing a comparison. Do not discard failures or advertise a small sample as a production SLO.

Pass a scenario's **functional** oracle only when approval/data-locality and each listed state outcome hold across applicable paths/checkpoints. A missing recovery mechanism or violated invariant fails that hypothesis. Keep performance acceptance unknown until #8/#28 set justified budgets; report measured values rather than invent a pass. Keep original game compatibility and portal/mesh behavior outside the baseline result.

Bound the first experiment to this fixture set, one room/channel/file area and a door stub, not a complete BBS or legacy game port. Choose an isolated shaper environment and capture-tool versions before execution; no runtime/shaper experiment was available or run in this issue. Delete synthetic account/session files and impairment state after the run; do not capture live credentials or user content. If these controls do not match actual operator needs, revise them explicitly before comparing stacks.

Handoff: [#8](https://github.com/gonz0w/faction-bbs/issues/8) turns task/link observations into an interaction budget; [#13](https://github.com/gonz0w/faction-bbs/issues/13)/[#21](https://github.com/gonz0w/faction-bbs/issues/21) define terminal/session semantics; [#28](https://github.com/gonz0w/faction-bbs/issues/28) adds hardware/concurrency/resource targets. Access, persistence, files, doors, and validation remain in #6/#9/#20/#22/#30/#34. Reopen if owner workload, target terminal, real overhead, or failure evidence changes the assumptions.

## Optional web and future networks

Run the initial baseline with HTTP disabled: network-terminal access must stand alone. A later private/optional browser portal has no right to bypass approval, require public exposure, or introduce JS/TS; feasibility remains #24. The synthetic second access path in security tests is not proof of a browser console.

Reticulum [#15](https://github.com/gonz0w/faction-bbs/issues/15), mesh/store-and-forward [#16](https://github.com/gonz0w/faction-bbs/issues/16), and subscribed shared chat [#17](https://github.com/gonz0w/faction-bbs/issues/17) need their own reachability, payload, ordering, identity, and disconnected-delivery studies. P2/P3 are ordinary impaired-link controls, not representative radio networks or DTN. Do not make a permanent connection, replicated boards, or simultaneous live chat across disconnected nodes a hidden requirement.

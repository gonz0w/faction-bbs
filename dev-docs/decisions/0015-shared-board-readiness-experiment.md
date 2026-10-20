# ADR 0015: Compare shared board ownership through readiness terminals

Status: experiment choice; production adoption deferred. Date: 2026-10-20.
Issues: [#55](https://github.com/gonz0w/faction-bbs/issues/55),
[#56](https://github.com/gonz0w/faction-bbs/issues/56).

## Question and constraints

Can a bounded shared board/session owner make progress toward 250 real SSH users
on two cores/about 1 GiB without removing approval, terminal isolation, durable
local authority or base OpenSSH? [ADR 0014](0014-credential-budget-and-scaling-experiments.md)
measured 150 completed short stages/168 admitted menus with headroom protection;
the per-member database/process model repeats costs. Full application budgets,
production capacity and the maintained appliance remain unaccepted.

## Alternatives

| Alternative | Benefit | Cost / gate |
| --- | --- | --- |
| Keep workers; reduce each SQLite cache / allocator footprint | Smallest behavior change; failure isolation remains | Configured cache is a ceiling, not each connection's actual resident charge; measure matched working set, preserve query behavior. Does not remove worker/process/kernel duplication. |
| Shared board RPC plus per-member UI worker | Retain complete UI; reduce repeated SQLite ownership | Still one UI process/member; adds framing/versioning, opaque connection capabilities, secret handling and bounded dispatch. Worth comparing if measured remaining gap is small. |
| Shared readiness terminals and board owner | Removes member processes and per-session SQLite handles; no serialized identity RPC needed | Larger common failure domain, shared event-loop fairness; complete theme/plain/resize/UI parity and integrated grants/failures required before adoption. Chosen isolated prototype. |
| Replace base OpenSSH with embedded SSH | Could remove repeated SSH processes | New security/protocol/update/dependency decision; not authorized by a memory result. Deferred. |
| Raise RAM or relax credential/security policy | Changes experiment target | More RAM can be an operator option; weaker credentials/isolation do not satisfy equivalent behavior and are rejected. |

## Experiment decision

Use a standalone, opt-in Rust package with the already reviewed dependency graph.
The unmodified `faction-ssh` hands an actual terminal descriptor to a separate
trusted UID through the existing native CMSG bridge. One bounded readiness loop
owns each connection, core opaque session and lease. No member identifier, role,
approval, opaque session or owner command crosses a member RPC. Authentication
results carry an internal, never-reused connection generation and are discarded
after disconnect. Two fixed credential threads return opaque core sessions for
the same board; the core rechecks current status/epoch before identity/actions.
The cross-process two-slot credential budget still coordinates other owner tools.

The loop finishes each bounded core operation before any nonblocking terminal
output. Explicit caps cover 250 connections, input/output, handoff/result queues,
two credential tasks, prompts, session lifetimes and stalled output. See the
[experiment specification](../../experiments/055-shared-board/README.md).
Only the implemented comparison surface may be measured as working. Full room
navigation/replies/retries, theme/season loading, plain/resize behavior, chat
editing/sysop menus and every integrated grant behavior remain adoption gates.
The terminal UID still cannot read private SQLite state. Shared-owner compromise
or a blocking/fatal backend fault affects all its sessions; this is a material
tradeoff, not OS isolation or a production sandbox claim.

## Credential measurement

Use a persistent native collector with explicit monotonic markers, sample query
times and absolute scheduled intervals. Calibrate known touched 0/19/38-MiB
allocations before actual eight-prompt credential bursts, failed credentials,
two held-slot saturation and release/recovery. Keep Argon2id v19 m=19456 KiB,
t=2,p=1 unchanged. Report measured maxima as lower bounds; neither cumulative
success count nor lifetime RSS sums determines concurrent KDF peaks.

## Validation and adoption gate

Compare current workers and the prototype on the same guest configuration,
independently salted distinct accounts, dataset, full SSH transports and explicit
kernel limits. Reset kernel/cache history before each model. Apply the same
conservative memory/process/file guard; failed offers are never users. Record
100/250 offers, actual completed menu/chat stages, delivery, CPU, anonymous/file/
pool/free counters, full mandatory process costs, cleanup, recovery and failures.

Adopt only after matched useful-work tests meet declared whole-host and application
budgets with margin for two real KDFs, equivalent complete member behavior and
approval/grant/durability semantics, repeat/soak/fairness, bounded failure cleanup,
maintainable service lifecycle and separate review. A short 250-connection result
alone does not pass. Default normal service remains eight workers. Physical SBC
tests gate later hardware claims, not this software experiment.

Confidence: high in measured process structure and core checks, bounded by the
actual recorded workloads; low in production capacity until the gates pass.
Reversal cost: low while standalone; deleting/retaining the experiment leaves the
normal broker/member path and schema unchanged. Revisit if the shared owner fails
fairness/recovery or meaningful memory savings; then compare shared RPC or trimmed
workers before expanding this implementation.

## Sources

Retrieved 2026-10-20: [NetBSD poll(2)](https://man.netbsd.org/poll.2)
(readiness/error/hangup semantics), [NetBSD nanosleep(2)](https://man.netbsd.org/nanosleep.2)
(monotonic absolute deadlines), [OpenSSH sshd_config(5)](https://man.openbsd.org/sshd_config)
(ForceCommand/MaxSessions/MaxStartups boundaries). APIs are primary documentation;
native executions, frozen source/binary identities and sanitized results provide
the implementation evidence. The new experiment does not alter OpenSSH admission.

## Recorded outcome

The [native report](../research/0029-shared-board-and-credential-peaks.md) records
224 approved shared menu sessions versus 168 workers at the guard, both with a
matched 100-member menu/chat pass; an earlier shared trial reached 232. No 250
stage passed. Sixteen boundary/failure oracles passed. Calibrated persistent
sampling captured the short real credential rise at measured 21–22-ms median
cadence; those are lower-bound observations under unchanged two-slot Argon2.
Continue the isolated candidate and per-transport attribution; full member UI,
fairness, resource workload/repeat/soak and deployment adoption remain open.

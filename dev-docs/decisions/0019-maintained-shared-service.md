# 0019: Maintained shared service with conservative admission

Status: implementation proposed for PR merge; production acceptance deferred.
Date: 2026-10-20. Authority: owner requested merge of #61 and shared-backend
integration into the service while preserving 250-member NetBSD headroom.
Issue: [#55](https://github.com/gonz0w/faction-bbs/issues/55), related #18/#20/#28.
Supersedes: the development service selection portion of ADR 0013; its worker
implementation and historical evidence remain valid rollback/reference inputs.

## Question and gates

Make the measured full UI a maintained service without granting terminals owner
commands, weakening authentication/approval, changing schema or automatically
raising capacity/host settings. Preserve two credential permits and cross-process
slots, current role/epoch/grant checks, bounded terminal work and complete SSH
accounting. Operational integration is separate from supported capacity or a
hostile-code sandbox. Physical SBC tests remain a later hardware-claim gate.

## Alternatives and primary evidence

Keep standalone executables and ad-hoc start commands: preserves experiments but
does not satisfy maintained configuration/lifecycle or installed-path validation.
Keep workers as the only service: useful rollback with smaller per-member fault
scope, but the existing 1-GiB worker experiment stopped below the target. Rewrite
SSH or replace the credential/storage stack: adds separate security/dependency
decisions without addressing this bounded integration request.

Promote the full shared owner to a unique workspace binary with strict configuration
and measured native terminal entry point. This reuses the measured core/UI boundary
and keeps worker rollback/schema compatibility. [Research 0033](../research/0033-shared-service-integration.md),
[results 034](../research/results/034-netbsd-shared-service.json),
[maintained service sources](../../crates/faction-service/src/main.rs) and
[rc.d integration](../../deploy/netbsd/faction_broker), generated/retrieved
2026-10-20, are primary executed/source evidence. Earlier source/result bytes stay
preserved in the [versioned evidence snapshot](../evidence/README.md).

## Decision and tradeoffs

Select `faction-service` in the opt-in NetBSD rc.d path. Keep eight sessions,
five-second launch-token refill and explicit enablement; higher/faster admission
requires an experimental flag in strict root/owner TOML. Keep the canonical
socket/lock path, existing private database, base OpenSSH, terminal/owner UID split,
safe theme snapshot, rotating work and encoded output deadlines. No new registry
dependency, schema or JavaScript/TypeScript enters the graph.

Maintain the terminal-only native C frontend as `faction-tty`, built natively with
hardening and validated descriptor/TTY behavior. Its narrow C boundary requires
manual bounds/lifetime review and is currently NetBSD-only. It does not parse
host commands, authenticate BBS members, hash passwords or access SQLite. Keep
the Rust frontend available for developer-host/worker comparisons.

Shared failure scope is larger; the owner remains trusted with local database
access. Explicit nonblocking UI states need parity tests as local features evolve.
The default cap is an application-session limit, not a cap on authenticated idle
SSH transports. The memory guard is a fixture admission policy, not implemented
as a production service controller. Ordinary scripts do not tune allocator/NTP/
sysctls. Host-policy scoping, clock quality, active mixes, soak and complete budgets
remain explicit unknowns. Short software VM passes are not support guarantees.

## Reversal and falsifiable validation

Stop the dedicated listener and backend, select `workers`, retain installed worker/
broker binaries and restart with the same schema. Root-managed pack flags remain
worker-only; shared mode rejects them. Status/stop find the actual backend after a
selector edit. Never unlink a live PID/credential lock. Frontend rollback requires
an explicit root ForceCommand edit/test/restart; no schema migration is needed.
Experimental guest-wide host-policy rollback is separate from backend selection.

The actual installed service must pass default-eight/ninth rejection, strict
config negatives (including nonregular files), expiry/HUP/restart/rollback,
complete UI/core authorization/TTY/SSH/crash/slow-reader checks, three distinct
fresh-boot 250-member short workloads under the unchanged sampled guard and
full native source/binary/SSH attribution. Record all outcomes, restore/read back
host settings, verify fixture integrity and remove private test material. A future
source change invalidates current evidence until re-executed or properly preserved;
historical snapshots never substitute for new service bytes.

Confidence is high only for the recorded development-service behavior on the
named NetBSD VM. Broader security/capacity/appliance acceptance remains gated by
#55/#18/#20/#23/#28. Lua menus and isolated doors remain next after the useful core.

# First local software slice

Status: working local core; small-board setup under #71, 2026-10-21. Owner accepts core first,
software/VM validation now, Lua menus/isolated doors next. The owner now prioritizes the ordinary running product over further edge-case
research. The [NetBSD quickstart](../docs/quickstart-netbsd.md) is the first-preview
path. Further 250-user combined-pressure/profile adoption stays deferred;
existing approval/credential/access protections remain.

First increment: [local identity/room core and console](../docs/local-development.md),
[ADR 0008](decisions/0008-local-core-and-credentials.md). Steps 1 and 3 have partial
implementation and tests; approval, sessions, durable posts/replies/unread/retries
work locally. A local vertical ANSI/theme/dual-navigation baseline now covers part of step 2
([pack guide](../docs/theme-packs.md), ADR 0009); full art/terminal support, account
lifecycle and room ACLs remain. Step 5 now has a [restricted SSH development gateway](../docs/restricted-ssh.md)
and [native full-application tests](research/0024-restricted-ssh-netbsd.md), plus
[managed service/eight-session failure checks](research/0026-service-lifecycle-load.md); 100/250
load/failure accounting and maintained deployment gates remain. Step 3 now has
[private export/verify/restore](../docs/export-restore.md) for current durable state
(ADR 0011); full installation/member interchange and power-loss gates remain.
Step 4 now has a [local-console chat baseline](../docs/local-chat.md), including
private grants/joins, bounded history/presence and simultaneous PTYs (ADR 0010);
server-level fan-out, rate limits and network output enforcement remain open.

## Build sequence and observable completion

| Step | Deliverable | Software acceptance evidence / existing issue |
| --- | --- | --- |
| 1 | Local identity, registration/pending, sysop bootstrap/approve/revoke and explicit session identity mapping | #6/#12/#21/#30: pending/revoked users cannot read/post/chat; permission changes affect active sessions; persistence/restart and owner recovery tested |
| 2 | Approved session -> safe board identity/art -> sysop welcome -> menu; useful ASCII fallback | #13/#21/#25/#29: malicious controls cannot execute terminal actions; arrow highlight/Enter and direct hotkeys route identically, sysop theme surfaces exercised; bounded input/output, screen limits, partial input and disconnect cleanup tested |
| 3 | Local Citadel room list/enter/read/unread/post/reply baseline with durable IDs | #9/#12: transactions and current room/membership checks, retry-after-lost-ack reconciliation, export/backup/restore and restart tests |
| 4 | BBS-local command chat and a second simultaneous member | #26: channel ACLs, bounded fan-out/slow readers, membership revoke and reconnect behavior; no federation required |
| 5 | Ordinary SSH access and sysop-run VM setup/recovery | #18/#23: real identity mapping, no member host shell/exec/SFTP/forwarding; optional HTTP/overlays absent; named NetBSD VM setup, normal restart/reboot and recovery; 100/250 capacity is a separate research target |
| 6 | Extension seam and first Lua menu/door | #10/#11: scoped host commands, explicit activation/revoke/version/cleanup, bounded helpers; a broken door returns safely without corrupting local data |

Steps 1–5 define the first useful local core. Step 6 is the first extension track,
following the owner's priority. Initial extensions should be sysop-installed; member-uploaded/native code remains
out of scope until its trust and confinement model is validated. Keep core functionality independent of Lua; a
declarative menu baseline can ship first. The latest Lua boundary probe is prior
architectural evidence, not completion of step 6 or a reason to skip steps 1–5.

Rust/primary NetBSD direction is accepted. Readiness/Mio and rusqlite/SQLite have
compatibility evidence; finalize the narrow application choices and real SSH/member
mapping as implementation begins. Do not wait for every future provider, network,
browser or game comparison before writing and testing the local software slice.

## Test environment and budgets

Use repeatable software tests and a named NetBSD VM. Keep the 1–2-core/about-1-GiB
host envelope and 100/250 connected workload as measurement targets. Measure the
complete app plus SSH/helpers/cache, bounds and failure recovery; report misses
and fix/revise them explicitly. Microbenchmarks are not capacity promises.
Physical SBC work is deferred until the owner has a complete product to load;
it blocks hardware-specific support claims, not development or VM-only software
validation. No hardware purchase or request is part of this plan.

## Explicitly outside first-core scope

Full file/mail suites, original-game compatibility, portal/browser terminal,
chat federation, mesh/Reticulum/onion/I2P, wallets, shops, permanent rooms and
blockchain games remain separate work. Keep interfaces small enough to attach
optional adapters later, with no speculative all-purpose plugin framework required
before the first room post. Nothing external gets implicit private-data or spending
authority. An empty/disabled extension directory must leave the local core useful.

GitHub #35 owns milestone acceptance and implementation intake. #10 owns generic
extension research; the new candidate issues capture optional future ideas without
making them prerequisites for a complete first local BBS.

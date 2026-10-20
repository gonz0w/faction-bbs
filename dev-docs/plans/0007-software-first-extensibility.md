# 0007: Software-first core and optional modern extensions

Date: 2026-10-19. Status: local work complete; publishing for review. Issues: #5/#10/#18/#20/#35, plus new owner-derived candidates after overlap review.

## Owner direction

Physical SBC hardware is unavailable. Do not gate software research/building on it;
the owner will load a complete product onto hardware later. Keep NetBSD as primary,
validate software in VMs/developer environments, label hardware claims untested.
Core comes first; keep room for adventurous ANSI/TTY extensions. The owner chose
Lua menus/isolated game doors as the first experiment after the local core and wants
both external-member-wallet and sysop-payment-service models compared before choice.
Permanent-message rooms, blockchain text/ANSI games/wagers and ANSI stores are
research candidates; no live publishing, signing, spending or deployment authorized.

## Bounded work

1. Read existing issues and sources; change near-term gates to software/VM evidence.
2. Define a concrete first local software slice and extension host responsibilities.
3. Research permanent storage versus proofs/pinning, payments, ANSI games/commerce,
   Lua and optional network helpers using dated primary sources; record tradeoffs.
4. Build a small Rust authorization/extension-dispatch probe with a separate Lua
   worker, fixed bounded protocol, deny-by-default APIs, current-approval checks,
   allocator/instruction/output bounds and parent deadline/crash cleanup.
   Predeclare: positive menu/post; pending/revoked/missing-capability rejection;
   missing OS/I/O/module APIs; oversized/control-byte output; memory/loop failure;
   child crash/stall containment and a healthy subsequent child.
   Use synthetic data only. No financial/network operations, wallets or smart contracts.
5. Capture distinct GitHub candidates with dependencies, prototype oracles and provenance;
   update existing extension/network/core issues instead of duplicating them.
6. Update README/docs/ADR/roadmap/changelog/index; run relevant checks; publish a
   focused PR on #40's branch without merging #38/#39/#40 implicitly.

The probe tests an architectural boundary, not a production plugin sandbox or BBS.
No NetBSD Lua execution, real room persistence, protocol adapter, money handling,
real gambling or permanence guarantee may be claimed without actual evidence.
Document the single-host/compiler/library versions and exact limits. Stop bounded
research after a report, executable evidence and explicit next build steps exist.

## Progress

- [x] Read repository/current issues; no existing specific chain-room/payment/store/wager items.
- [x] Owner chose Lua first after core; compare both payment models.
- [x] Build/run software probe: 16 developer-host oracles passed, allocator cleanup confirmed, native stalled/crashed children reaped; no NetBSD Lua or OS-sandbox claim. Native binaries were temporary and cleaned up.
- [x] Research, software-first core plan and backlog captured. New issues #42–#45; existing #5/#9/#10/#17/#18/#20/#28/#35 updated rather than duplicated. All repository/saved-evidence checks and 23 existing unit tests pass.
- [ ] Publish PR and verify CI.

## Handoff

First core implementation follows `dev-docs/first-local-slice.md` / #35. Lua remains
first extension after it. The new future candidates preserve permanent rooms, both
payment models, ANSI commerce and game economies without making them core gates.
No physical SBC tests, real game/provider, live publication or funded transaction
was performed. The native worker/supervisor builds and test children were cleaned up.

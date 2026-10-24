# 0007: Software first, local core first, Lua extensions next

Status: accepted **owner sequencing/product direction**; technical host/provider choices proposed.
Date: 2026-10-19. Authority: explicit owner instruction and clarification replies.
Issues: #5/#10/#18/#20/#35 and the new candidates linked in the research report.
Supersedes: any near-term physical-SBC testing dependency in ADR 0005/0006 plans.
Rust/primary NetBSD, security, privacy and resource measurement requirements remain.
Evidence: [research](../research/0010-extensible-ansi-platform.md),
[software scope](https://github.com/gonz0w/faction-bbs/issues/35), [product direction](../../docs/extensions.md).

## Decision

Build and validate software now using developer hosts and NetBSD VMs. The owner
has no physical SBC test hardware and plans to load a complete product later.
Do not make obtaining hardware a build/research gate. Reserve hardware-specific
compatibility/performance claims until actually tested; keep host/resource targets
as software measurement inputs rather than forgetting the small-system goal.

The first useful local BBS core comes first. The first extension experiment after
that core is Lua menus and isolated game doors. Keep Lua optional, permission-scoped,
bounded and failure-isolated. The small existing Lua probe supports studying this
boundary; it does not adopt a production runtime, sandbox or extension API.

Explore permanent-message rooms, modern networks, ANSI shops/payment integrations
and blockchain text/ANSI games including wager modules as optional future work.
Compare both member-controlled external wallets and a separate sysop-run payment
service; no currency/provider/custody model is selected. No JS/TS dependency or
browser requirement may be introduced for these features.

Private local data remains local by default. Automatic BBS federation stays limited
to explicitly participating shared chat. A new public-publishing room/module needs
its own deliberate disclosure/authorization contract; this research direction does
not allow automatic export of existing private content. Live publishing, financial
transactions, custody and real-value gambling are outside the current experiments.

## Alternatives and tradeoffs

Waiting for physical boards stalls authorized software work without resolving
current identity/authorization/terminal questions. VM evidence advances those
questions while leaving real driver/media/power/performance gaps visible.
Building every network/economy before the core would obscure the first usable
system. Ignoring extensibility until late would force local domain assumptions
into external protocols. A small local core with explicit extension seams and
bounded future experiments balances these costs; no general framework is required
before the first local post.

Confidence is high in the owner's sequencing, moderate in the proposed boundaries,
and low in unbuilt providers/security/capacity. No numerical ranking is fabricated.
Reversal is low while prototypes/interfaces remain local and disposable; it rises
for irreversible publication, private data disclosure or custody. Fake providers
and nonredeemable test credits are the next external-feature experiment baseline.

## Validation and next work

The report records a 16-oracle developer-host Rust/Lua probe with allocator/hook/
protocol bounds and child crash/deadline cleanup. Production OS sandbox, current
real member authorization, NetBSD Lua execution and whole-host costs remain open.
Build #35's local identity/room/session slice next, then evolve #10's Lua contract.
Retain VM load/recovery/security tests. Reopen the host design if it cannot keep
the core safe and useful when optional workers crash, stall or are disabled.

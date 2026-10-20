# A terminal BBS with room to experiment

Status: owner research direction, 2026-10-19; no extension, wallet, chain or network
integration is released. Core local BBS software comes first. Physical SBC hardware
is unavailable; the owner will test a complete product later. Software/VM evidence
drives development now; hardware-specific support claims wait for actual tests.

The first extension priority after the local core is **Lua menus and isolated game
doors**. Preserve the ANSI/TTY experience while exploring newer storage, networks,
payments, stores and games through optional modules. A sysop should be able to run
a private local board with every external module disabled.

## Ideas to investigate

| Idea | Member experience | Research boundary |
| --- | --- | --- |
| Lua menus and doors | A board with its own commands, screens, stories and games | Explicit scoped APIs, resource budgets, lifecycle and failure isolation; no implicit host shell |
| Permanent-message room | A separately labeled public publishing room for messages or ANSI art | Compare chain data, storage networks, pinned content and proof-only anchoring; never promise literal forever |
| ANSI store | Browse art, games, software or services and request payment from a terminal | File rights/provenance, local delivery/entitlements and verifiable settlement; no browser wallet dependency |
| Game economies and wagers | Text adventures, tournaments, economies, betting/gambling modules where appropriate | Compare local credits, signed receipts and chain settlement; simulations first, no real-value deployment selected |
| Other networks | Reach the board or selected public services through optional adapters | Research Tor/I2P, Reticulum/mesh and signed relay protocols without imposing their daemons on every board |

Both member-controlled external wallets and a separate sysop-run payment service
are research candidates. Neither custody nor a wallet provider/chain is selected.
Terminal use must support readable invoice/transaction summaries and confirmations;
do not require a JavaScript browser, extension or SDK. No real purchase, publication,
transaction signing or spending is part of the current experiments.

## Local privacy and deliberate publication

Local rooms/mail/files/accounts remain local by default; automatic BBS federation
continues to apply only to subscribed shared chat. The proposed public publishing
module is a separate, explicit outbound capability under research, not permission
to mirror existing private rooms. It needs a distinct room type, public/permanent
label, exact preview and deliberate author confirmation before any external send.
Provider failure must leave the useful local core working. No deletion of a local
copy can promise deletion from another network or independent retained copies.

Extensions propose actions; the Rust host rechecks the current member, local role,
room scope and extension permissions. Network access, local private reads, external
publication, payment requests and spending are separate privileges to design.
Remote keys, payments and game outcomes never grant local BBS membership approval.
Do not turn a scripting API into direct database, shell or spending-key access.

The [research and software build plan](../dev-docs/research/0010-extensible-ansi-platform.md)
records sources, alternatives, actual probes and scoped follow-ups. GitHub Issues
hold candidate status; these ideas do not all belong in the first release.

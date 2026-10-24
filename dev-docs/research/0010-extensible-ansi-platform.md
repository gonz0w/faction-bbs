# 0010: A small local core with adventurous ANSI extensions

Status: bounded research completed; production extension contract proposed.
Date/retrieval: 2026-10-19. Issues: #5/#9/#10/#11/#18/#19/#20/#31/#35,
plus #42/#43/#44/#45.
Owner: project founder; [ADR 0007](../decisions/0007-software-first-and-extension-direction.md)
records accepted sequencing, not a production Lua/provider/currency selection.
Stopping condition: source comparison, one executable boundary experiment, scoped
backlog and a concrete software-first core plan. No live external actions.

## Question and direction

How can a Rust/NetBSD BBS remain small, private and terminal-first while supporting
Lua menus/games, permanent publication, payments/shops and modern network adapters?
The owner explicitly wants more building/research, a local core first, Lua menus
and isolated doors next, and both wallet models compared. Physical SBC hardware
is unavailable and is **not a software build gate**. VMs/developer-host tests now;
physical checks belong after a usable product, before hardware-specific claims.

An ANSI client can present modern services as rooms, commands and doors. The
service protocol need not dictate the member UI. Our design inference is to keep
local state authoritative and place optional external service/crypto work behind
bounded adapters or companion processes. No full blockchain node is a core
prerequisite; helper trust/availability/cost must still be explicit and measured.

## Storage: bytes, proofs and availability are different

| Candidate | Possible ANSI experience | Documented distinction / open cost |
| --- | --- | --- |
| Local SQLite + export/backup | Ordinary private rooms; export selected records | Useful independent baseline; backups/retention remain the sysop's responsibility |
| Hash/Merkle-root anchoring | A room shows a publication proof and compares a local record | A digest does not store/recover the message bytes or hide a predictable low-entropy message; separate archive required |
| On-chain payload | Public text messages/receipts | Fees, chain retention/indexer access, finality/reorgs and metadata disclosure; no full-chain-per-BBS assumption |
| Arweave-style storage | Explicit public archival room and durable ANSI zines | Endowment-based persistence design; ecosystem/economic/retrieval assumptions, not a literal forever guarantee |
| IPFS/content-addressed files | Fetch verified ANSI art, zines or game packs by content ID | Addressing/integrity is distinct from persistence; pinning/providers must retain bytes |
| Filecoin/contracts or other storage helpers | Optional replicated public archives | Renewal/provider/retrieval economics and companion footprint require separate evidence; not measured here |

Primary evidence: [IPFS persistence](https://docs.ipfs.tech/concepts/persistence/),
[Ethereum storage comparison](https://ethereum.org/developers/docs/storage/),
[Arweave lightpaper](https://arweave.org/files/arweave-lightpaper.pdf) and
[Arweave HTTP API](https://docs.arweave.org/developers/arweave-node-server/http-api.md).
An HTTP API can be called by a native adapter without a JS client or a public HTTP
portal. This is protocol feasibility, not a selected Rust SDK or tested integration.
No live provider, cost estimate, availability guarantee or funding was tested.

## Public publication is a new explicit boundary

Chat-only **BBS federation** remains the baseline. The owner's proposed chain room
adds a separately researched public publishing category; it does not authorize
automatic export of private rooms/mail/files/accounts. A proposed flow is:

```text
new PUBLIC-PUBLISH room -> draft -> exact safe preview + destination/cost
  -> deliberate author confirmation -> current approval + room/publish capability
  -> bounded outbox -> optional provider -> pending / confirmed / failed / unknown
```

This is a proposed interface, not implemented confirmation UX. Never convert a
private room to public history silently. Store exactly what the author confirms;
minimize identifiers/metadata. Reconcile unknown/retried sends against immutable
payload IDs rather than automatically paying/publishing twice. A correction can
add a new record/local moderation view; it cannot promise erasure of remote copies.
Even hash-only exports can reveal equality or permit guessing; encryption does not
make irreversible publication risk disappear. Public input remains terminal-untrusted.
The first provider experiment uses local fakes and demonstrates **zero outbound
publication when disabled**. Actual remote publication is outside this task.

## Payment, shops and games: three separate responsibilities

The owner chose **compare both** external-member-wallet and sysop-service paths.
Keep UI/order intent, signing/settlement and entitlements/game accounting separate.
An invoice service can be watch-only, noncustodial or custody-bearing; calling it
a helper does not settle that trust question. A member never pastes seed phrases
into a door; no Lua callback gets a generic spending key or unrestricted signer.

| Model | Candidate terminal workflow | Unknowns before adoption |
| --- | --- | --- |
| Member's external wallet | Display a readable intent/invoice or export an unsigned request; member signs externally | Headless wallet compatibility, fees/expiry, return proof and substitution prevention |
| Separate sysop service | Door requests a scoped invoice; helper reports independently checked settlement | Key custody, daemon/package cost, outage/replay/recovery, private API authentication |
| Fully local test credits | Text games/stores exercise offers, receipts and idempotent delivery without money | Not redeemable currency, real settlement or a security/economic guarantee |

[BIP-174 PSBT](https://github.com/bitcoin/bips/blob/master/bip-0174.mediawiki)
documents a Bitcoin transaction exchange/signing format; it does not pick a Faction
wallet. [BOLT 11](https://github.com/lightning/bolts/blob/master/11-payment-encoding.md)
documents Lightning payment requests; parsing/displaying one is not settlement.
No payment library, network, wallet, custody model or funded transaction is adopted.
Price/fee/recipient/network/asset/expiry/order confirmation and settlement state
must survive replay, helper outage and wrong-network/asset claims. Compare exact
amounts in integer base units; do not trust a client or game saying "paid".

An ANSI shop could sell/distribute approved software, art, game packs or services;
order/delivery/license rights belong in #44, payment transport in
#43. Preserve local member/file ACLs after settlement, and test duplicate
payment callbacks producing one entitlement. No third-party wares/assets are bundled.

Game possibilities include persistent space economies, competitive doors, signed
tournament results, collectibles and optional betting/gambling modules. Begin with
nonredeemable test credits. Hash receipts/commit-reveal can prove particular steps,
not automatically fair randomness, honest game logic or equitable settlement.
Real-value modules need separate contract/custody/loss/dispute and jurisdiction/
operator/age policy decisions; no live gambling is deployed by this research.
#45 is scoped to simulations and comparison first.

## Modern network and service map

These are research leads on existing #5/#7/#15/#16/#17, not supported adapters.

| Candidate | Type of boundary | Existing issue and experiment |
| --- | --- | --- |
| Tor onion service | Reachability over an optional companion daemon | #5/#18: private bind/daemon failure, session metadata and latency; no anonymity claim from UI |
| I2P SAM v3 | Native client to optional router bridge | #5: stream/datagram/session distinctions, reconnect/quotas and non-JS implementation feasibility |
| Reticulum / delayed mesh | Different delivery/identity constraints | #15/#16: bounded envelopes, explicit delay/retry/expiry; do not pretend every link is interactive TCP |
| Nostr or signed public relays | External event publication/subscription | #5/#17 plus public-publication issue: authenticity/retention/relay trust; signatures never grant BBS approval |
| IPFS/chain/payment services | Data/payment adapters, not terminal transports | Their new issues: content/settlement verification, optional helper cost and failure independence |

Sources: [Tor rendezvous specification](https://spec.torproject.org/rend-spec/index.html),
[I2P SAM v3](https://i2p.net/en/docs/api/samv3/),
[Reticulum understanding](https://reticulum.network/manual/understanding.html),
[Nostr NIP-01](https://github.com/nostr-protocol/nips/blob/master/01.md).
NIP-01 describes signed events and relay exchanges; an event signature is not a
durability or deletion guarantee. None of these sources is a Faction runtime test.
Do not duplicate the existing network/federation backlog for a new acronym.

## Lua alternatives and a boundary we actually built

| Approach | Why consider it | Current limitation |
| --- | --- | --- |
| Declarative menus | Small, simple initial core styling and dispatch | Less expressive; keep as baseline rather than requiring a VM for each screen |
| In-process Lua/mlua | Straightforward Rust callbacks, memory-limit/hook APIs | C/FFI shares core process failure; full dependency/NetBSD audit unperformed |
| Separate Lua worker + scoped host requests | Script failures can be terminated/reaped outside the host | Process/IPC cost and OS confinement still need work; selected for this disposable boundary probe only |
| WASM/component or native process plugins | Other portable/legacy extension options | Runtime footprint/NetBSD availability/security unmeasured; no runtime selected |

Lua 5.5's primary manual documents an embeddable C API, allocator control and
instruction hooks. Lua is MIT licensed. mlua 0.12.2 exposes safe Rust-facing Lua
and memory/hook APIs; that alone does not sandbox native faults or choose a
production binding. Exact sources: [Lua manual](https://www.lua.org/manual/5.5/manual.html),
[Lua license](https://www.lua.org/license.html),
[mlua Lua API](https://docs.rs/mlua/0.12.2/mlua/struct.Lua.html).
No mlua crate/transitive graph was added or claimed audited.

The [new probe](../../experiments/010-lua-boundary/README.md) uses a dependency-free
Rust synthetic authorization dispatcher and a separate small C/Lua worker with
two proposal APIs, `menu` and `room_post`. It deliberately loads no standard
libraries: no OS/I/O/module/loader/debug/coroutine/pcall access. Worker messages
carry no trusted member identity. Rust checks current synthetic status and grants,
validates the complete bounded batch before accepting any simulated mutation,
and discards proposals if the worker failed. Fixed fixtures include a hostile API
attempt, controls, oversize messages, memory exhaustion, infinite Lua loop and
independently injected native child crash/stall. The parent has a two-second deadline.

**Observed:** 16 oracles passed on Darwin ARM64 using Rust 1.98.0, Apple Clang
21 and installed Lua 5.5.1. The successful small menu/post fixtures requested about
6–8 KiB peak live memory **through the Lua allocator only**; the growth fixture
failed below the configured 1-MiB Lua heap cap, and allocator live bytes returned
to zero after close. The hook interrupted a runaway script; a native stalled child
was killed/reaped; a healthy child still ran afterward. Native binaries, C runtime,
allocator overhead, stacks, parent memory and process costs are excluded; this is
not a tiny total-worker/RSS or scalable 250-door result. See
[saved exact observations/hashes](results/010-lua-boundary.json).

Limits: synthetic membership/grants, no persistent room/chat/auth/ANSI engine,
fixed script fixtures, no live revocation race/transaction proof in this probe,
no standard game libraries, OS filesystem/network sandbox, NetBSD Lua execution,
process-wide memory cap, real game, wallet or external publication. Separate
processes/API restrictions are not an OS sandbox. A compromised native Lua/C
process retains its OS account privileges; production helpers need a separate
restricted identity, descriptor/filesystem/network policy and global worker limits.

## Gates, confidence, reversal and next work

No-JS/local-independence gates pass for this **probe**: Rust std, C and installed
Lua only, synthetic local data, no network or publication. Current-approval checks
and failure handling pass narrow fixtures. Production sandbox, NetBSD/runtime
dependencies, all-host memory, real identity and ledger/privacy gates remain
unknown and block production adoption. Do not score unsupported alternatives as
winners. Software-first sequencing is accepted; the Lua host shape is still proposed.

High confidence in owner priorities and measured host oracles; moderate confidence
in the adapter decomposition; low confidence in unbuilt external integrations.
Keeping extensions optional and the boundary disposable makes reversal inexpensive.
External immutable publication or custody would raise reversal cost sharply, which
is why fake providers/no-value tests precede live features.

Next implement the [GitHub issue](https://github.com/gonz0w/faction-bbs/issues/35), starting
with real identity/approval/rooms and terminal safety in #6/#12/#21/#29/#30/#35.
Then #10 advances Lua with a versioned capability/lifecycle contract, actual
NetBSD software tests and OS-level worker/resource enforcement. Parallel bounded
research can refine the new future candidates without expanding first-core scope.

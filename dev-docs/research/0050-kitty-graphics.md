# 0050: A bounded inline Kitty graphics path

2026-10-22, #117 after merged #119 (7df112f). Question: can the prototype
show one modern bitmap without widening generic artwork/control trust or holding
52 KiB of encoded pixels in its 32-KiB shared queue?

## Comparison and decision gates

| Option | Fit and cost | Decision |
| --- | --- | --- |
| Classic ANSI and native block/braille | Small, existing fallback and themes; no bitmap fidelity | Retain default |
| Inline raw RGB Kitty | Named positive query; bounded base64 chunks; no local filesystem assumptions over SSH | Adopt narrow optional preview |
| General Kitty APC passthrough | File/shared-memory, PNG/compression, animation and unrelated remote control surface | Reject |
| Sixel | Separate negotiation/decoder and emulator coverage | Defer independently |
| Image toolkit/theme importer | Additional graph, asset parsing and provenance work | Defer; fixed original code-owned asset first |

The [primary Kitty graphics specification](https://sw.kovidgoyal.net/kitty/graphics-protocol/),
retrieved 2026-10-22, defines APC inline `t=d`, raw 24-bit `f=24`,
query action `a=q` without retention, final/nonfinal `m` chunks, reply suppression
`q=2`, and image-specific uppercase deletion. Chunks after the first carry only
`m` and `q`; base64 is divisible by four. A delete aborts unfinished uploads.
This implementation uses at most 4,000 payload bytes / 4,054 total command bytes,
below the protocol's 4,096-byte payload maximum. No device-attributes query is
needed: a short no-reply timeout conservatively refuses support.

The optional [Ghostty feature documentation](https://ghostty.org/docs/features)
was reviewed for parent #96; advertised support is not conformance evidence.
No GUI/font certification or tmux wrapper is inferred. Actual installed GNU screen
returns no reply in the private detached query fixture.

## Implementation and costs

Use one original compile-time 160x80 frequency sigil, 38,400 read-only bytes,
not a new identity-art style or third-party asset. Its inline transfer is 51,410
bytes: thirteen commands and 51,200 base64 bytes. The common encoder allocates
exactly 4,096 bytes per live chunk, using no decoder, encoder dependency or whole
frame cache. A shared member retains only image-active/rate/cursor/probe state.
The owner rechecks approval on each chunk and queues the next only after output
drains, under rotating work and existing 32-KiB/five-second constraints.
A five-second total upload deadline prevents a slow receiver holding a partial
frame across unlimited chunks. A helper-free native gate checks exact code-owned
bytes, order and rate, then emits regenerated chunks. Its only initial permission
is explicit local `--kitty` plus a successful local probe before SSH.

Native-filter working storage is a 4,096-byte APC accumulation plus a 4,096-byte
expected chunk. Verified chunks become separate bounded Text events, avoiding
another copy in the filter text accumulator. The input pump reads at most 4,096
bytes; at most one preceding partial command can complete with that read, so
verified graphics events total less than 8,192 bytes. The existing local output
queue remains capped at 32,768 bytes. These are bounded buffer costs, not total
RSS or a new 250-member measurement; allocator/Vec/event/TTY/kernel overhead
remains separate. Measured struct/capacity values are in the evidence record.

## Validation and limits

Falsifiable gates: host/native workspace tests, independent Python base64 decode
of real PTY output, exact command/byte/capacity assertions, stopped output deadline
and flag restoration, positive/refused/unknown/malformed/late probes, native local
opt-in PTYs and native shared UID/SSH-orchestrated frontend streaming/interruption.
The optional installed-screen fixture must return bounded refusal or a recorded
response, never imply broader emulator support. All previous classic, Unicode,
reader/unread/revocation checks remain required.

Confidence is high for bounded byte/state behavior on software fixtures; actual
GUI display/font/multiplexer compatibility remains #118. A terminal can lie about
support; the effect is limited to this one trusted presentation. Fixed IDs assume
Faction owns its terminal screen; unrelated simultaneous applications sharing it
are outside this preview. Failed transport cleanup is best effort. This is not a
PNG/art importer, animation engine or hostile-terminal sandbox.

## Reversal

Choose C/D, reconnect, or omit the native flag. Removing the preview changes no
DB format/schema, identity, approval, file transfer, helper, listener or host policy.
Keep all thirteen frozen snapshots and twenty-five historical results unchanged.

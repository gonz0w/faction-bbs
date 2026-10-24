# 0048: Bounded native operator host counters

Date: 2026-10-22. Status: selected and software-validated, #95/#18/#98.
Builds on [ADR 0047](0047-private-operator-snapshots.md).

## Decision and alternatives

Use the existing `faction-unix` C/Rust boundary for fixed read-only host counters.
The dashboard gets 64 integer slots, at most 16 non-loopback interfaces, and one
64-KiB stack routing-query buffer. No refresh subprocess, registry dependency,
owner thread, listener, privilege change or general-purpose query API is added.
Kernel calls do not gain a hard deadline guarantee; the API has fixed data bounds.

A broad metrics crate would introduce dependencies and platform coverage needing
review. Parsing per-refresh command output adds helper lifetime/memory and text
format assumptions. The narrow native API covers the selected NetBSD target and
macOS development host; unsupported platforms/failed individual metrics return
unavailable. Application delta/graph logic stays safe Rust.

CPU busy uses elapsed tick deltas; interface bytes use monotonic elapsed time.
Resets/scope changes/first or resumed samples warm up instead of spiking. RAM used
includes cache/reservations, and network interface sums can double-count tunnel/
bridge traffic. Load, utilization, capacity and working set remain distinct.
Histories cap at 60 entries; only the newest fitting samples render. Plain stays
ASCII; UTF-8 block graphs require explicit `--unicode` and valid ANSI geometry.

## Evidence, confidence and reversal

See [GitHub issue](https://github.com/gonz0w/faction-bbs/issues/95) and
[native observations](../evidence/operator-host-netbsd/README.md). Actual installed
NetBSD 11 headers/SDK and native builds/runtime checks complement deterministic
reset/failure/delta tests. This is software evidence, not a supported capacity,
all-NetBSD-architecture, physical-terminal or sandbox claim. The original result
is frozen in the twentieth source snapshot; never refresh its historical hashes.

Reversal is a dashboard binary rollback with no state migration. Leave private
publication disabled if undesired. Existing member/service authority and defaults
stay unchanged. Peer/backup adapters and host-action/hardening boundaries remain
separate; no synthetic success indicators or shell-command field.

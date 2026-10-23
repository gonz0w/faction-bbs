# Opt-in local chat lounge software evidence

[Results](results.json) pin host and a separate marked NetBSD 11.0 ARM64 software
VM (two cores/1 GiB), using normal eight sessions and 5000-ms admission.

Coverage includes default-hidden/explicit availability across chat channels,
bodyless requests and explicit accept/decline, channel/draft preservation,
cooldown, blocking/mute/UTC quiet hours, opted-in sysop paging, lease expiry,
disconnect/reconnect, revocation and escape-free plain commands. Signals are
explicitly polled; presence outside chat and automatic page notifications remain
open under #137. Existing channel /names stays separate from lounge hiding.

Local/worker and real restricted SSH/native journeys use actual authenticated
actions. Some expiry/cooldown controls age timestamps in the owned fixture;
they are synthetic clock controls, not measured timing/capacity evidence.
Native complete recovery runs the actual CLI under distinct owner/file UIDs;
synthetic backup rows verify durable privacy intent and removal of live authority
and rate timestamps. Existing room/writing/editorial/art/library/vault/travel
regressions also pass.

Selected native input bytes match the clean final build/runtime, and installed
binaries match tested binaries. The previous editorial record and 310 union
inputs remain unchanged through snapshot 41 and
[its manifest](../editorial-baseline-manifest.json). No new registry dependency or
normal-budget change. The owned VM, test credentials and temporary vendor are
removed after validation; the saved owner playground remains stopped/intact.
No physical hardware, new capacity, encrypted-mail transport, public service or
hardening claim. See [plan 0074](../../plans/0074-social-lounge.md) and
[ADR 0069](../../decisions/0069-opt-in-chat-lounge.md).

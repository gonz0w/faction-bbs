# 0043: Native Unicode and terminal capabilities

Issue #96. Status: implemented and validated, ready for review, 2026-10-22.

- [x] Merge green #116 (a611cd5), freeze 101 exact original inputs and result.
- [x] Review primary terminal/Unicode sources and a no-default-features width table.
- [x] Implement session-only terminal settings, native Unicode room/subject rendering,
  cell-aware clipping/wrapping, color choices and bounded synchronized redraw.
- [x] Isolate a single explicit DECRQM sync query from all member command input.
- [x] Validate known/unknown/lying/malformed/late probes, fallback, fonts/widths,
  native-client filtering, classic regressions and native NetBSD software PTYs.
- [x] Record budgets, dependency review, comparison/ADR, exact evidence and PR.

Default classic mode, current schema 7, ordinary eight sessions and unchanged
credential/owner/helper limits stay. Graphics/mouse/extended keys remain disabled;
capability signals/preferences grant no member, host, file or network authority.
No raw image/art/control passthrough, public endpoint or host/hardening changes.
Software fixtures suffice; keep the original saved playground stopped and private.
Preserve twelve source snapshots and twenty-four prior result identities.

## Actual outcome

145 workspace Rust tests pass on macOS and native NetBSD, with five modern and
five classic composer PTY controls per host, fifteen shared native controls,
104 Python tests and the unchanged dependency/source guards. The expanded shared
fixture respects normal admission pacing after initially exhausting its burst;
Done returns directly to the shared menu. Local probe deadlines are rechecked
after poll wakes. Native body rendering avoids per-scalar temporary allocations.
Current 106 application input hashes match the native checkout; the clone is
removed and the original saved playground is unchanged/off. #117 and #118 capture
bitmap/client-gate and real emulator/font/multiplexer follow-ups. No full GUI,
bitmap, hardware, capacity or hardening adoption is claimed.

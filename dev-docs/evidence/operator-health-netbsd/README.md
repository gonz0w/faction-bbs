# Publisher lifetime: NetBSD software evidence

[results.json](results.json) records the 2026-10-24 software checks for
[issue #95](https://github.com/gonz0w/faction-bbs/issues/95). It pins the feature
source revision and 230 native input hashes. Those exact inputs matched the
owned NetBSD 11 ARM64 build before teardown; source remains resolvable through
that Git revision. This report is a historical observation, not live task state.

The two-core/1-GiB VM used the locked offline dependency set. Validation passed:

- 378 workspace Rust tests on macOS and NetBSD, plus strict host workspace Clippy.
- Eight existing operator UI controls and nine lifetime controls per OS, including
  legacy/unavailable/stale/future observations, released/replaced locks, safe input,
  plain/ANSI output, 40x18 resize and restored terminal settings.
- Eight actual shared-owner/SSH controls: explicit opt-in, denied member UID,
  normal approved login/counts, operator resize/exit, graceful stop, abrupt exit,
  restart and a deliberately substituted old instance with the current PID.
- Native complete-bundle recovery (21 controls), local console, two-slot credential
  and broker lifecycle (seven controls) regressions.

The substituted snapshot tests PID reuse semantics; it is not an observation of
actual kernel PID reuse. A held publisher lease is not proof of responsiveness,
peer reachability or end-to-end delivery. Federation stays unconfigured. Schema,
normal admission/credential/helper/output limits and dependencies are unchanged;
no physical hardware, new capacity, artwork approval or hardening claim is made.

## Preserved failed candidate and repeat

[The failed observation](rejected-header/observation.json) and its two actual
[dashboard](rejected-header/crates/faction-cli/src/operator.rs) /
[PTY harness](rejected-header/tools/smoke_operator_health.py) inputs preserve the
first NetBSD compact/unavailable-header timeout. The status text clipped the
40x18 dimension suffix. Current code reserves room for that suffix; the original
assertion remains. These archived inputs are evidence, not current build sources
or agent instructions. Earlier accepted native records and frozen snapshots are
unchanged; none is relabeled as testing this feature.

A later green attempt lost its private comparison inputs when cleanup ran after
a failed export. No matched-input claim is made for that attempt. The final report
comes from a fresh complete repeat with export-before-cleanup checks. It records
verified removal of the owned VM/cache/private inputs and confirms the saved
owner playground stayed stopped with its disk metadata and login/key bytes intact.

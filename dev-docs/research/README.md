# Research and experiments

Use the [template](template.md) for a bounded investigation, linked to an issue and the affected decision. Prefer primary sources with retrieval dates and relevant versions. Distinguish documented behavior from measured results and inference.

An experiment answers one named uncertainty with stated inputs, resource/link conditions, pass/fail observations, and cleanup boundaries. Keep disposable prototype code under an issue-named `experiments/` directory only when an issue calls for it; do not create a production stack by accident. No JS/TS dependencies are permitted.

The [decision framework](decision-framework.md) defines gate screening, comparison criteria, evidence/confidence, reversal cost, and prototype requirements. The [approval-boundary worked example](0001-approval-enforcement-example.md) exercises the method; it contains no measured product behavior or adopted runtime architecture.

The [private/poor/interrupted-link journey report](0003-constrained-user-journeys.md) and [synthetic payload results](0003-link-budget-results.md) provide proposed test inputs for #8/#28. Reproduce them with `python3 tools/model_link_budget.py --check`; regenerate deliberately with `--write` when fixtures or arithmetic change. This model is Python repository tooling, not a product stack or runtime/network experiment.

The [interaction budget research](0008-interaction-budget.md) includes [measured application-paced ASCII/ANSI replay](0008-replay-results.md). The [small-host research](0028-small-host-budgets.md) defines proposed memory/CPU/storage/load targets and records a C allocation-ownership probe. These are bounded research experiments, not an implemented or measured BBS.

```sh
python3 tools/measure_text_replay.py --check
python3 tools/measure_buffer_probe.py --check
```

`--check` verifies saved source/result provenance without rerunning timing or requiring a compiler. Deliberate `--run` refreshes observations on the current host; it is not a CI performance gate. Buffer `--run` additionally requires `cc` and uses disposable binaries outside the checkout. Python remains research tooling, C remains an isolated probe; neither selects the product stack.

The [native runtime comparison](0004-runtime-comparison.md), [local storage/recovery comparison](0009-local-storage-comparison.md), and [hobby appliance/access direction](0018-appliance-and-access-direction.md) narrow the next slice to a proposed Rust/readiness + SQLite WAL experiment. SSH/private-network product intent is accepted; Rust and primary NetBSD are now accepted direction ([ADR 0006](../decisions/0006-rust-and-primary-netbsd.md)); complete production stack and image/backend selection remain unvalidated. See the [native NetBSD dependency/gateway report](0023-netbsd-dependencies.md) and `python3 experiments/023-netbsd-dependencies/verify_evidence.py` for saved provenance checks.

```sh
python3 tools/measure_native_sessions.py --check
python3 experiments/009-local-storage/probe.py --check
```

Fresh `--run` native probes require C/Rust/Cargo/Go and initial crate access; saved-evidence checks require only Python. SQLite fresh runs require stdlib SQLite with FTS5. Both use synthetic local disposable data; there is no deployable BBS in these experiment folders.

The [extensible ANSI research](0010-extensible-ansi-platform.md) compares optional storage, wallets, shops, games and modern networks, with a separate Rust/Lua boundary experiment. `python3 tools/probe_lua_boundary.py --check` verifies saved provenance; native reruns require Rust/C/Lua as documented. Physical SBC tests are deferred, with software/VM core work first.

The [menu/theme and art reference study](0025-menu-and-art-direction.md) captures Oblivion/2-inspired dual navigation, sysop/seasonal themes and attributed ACiD/iCE/Razor-related references. It proposes software menu/parser/theme tests; none is claimed executed.

The [restricted SSH/native application report](0024-restricted-ssh-netbsd.md) records actual member flows, complete Rust/PTY/recovery tests and the native CMSG ABI repair; `python3 tools/check_netbsd_application_evidence.py` checks saved source provenance without rerunning a VM.

The [managed lifecycle/load report](0026-service-lifecycle-load.md) extends this with service, overload and failure oracles.

The [architecture map](../architecture.md) and [GitHub issue](https://github.com/gonz0w/faction-bbs/issues/133) identify the next questions. Full secure-server, target-hardware, real-network and terminal/failure experiments remain to be built.

- [0027: Native memory profiling and distinct-account SSH scaling](0027-memory-scaling.md)

The [shared-board and persistent credential-peak comparison](0029-shared-board-and-credential-peaks.md)
extends #55/#56 with an isolated readiness terminal owner, matched full SSH loads,
native authorization/failure oracles and calibrated persistent sampling. It does
not change the normal eight-worker gateway or adopt a production replacement.

- [0030: Native allocator and remaining SSH transport memory](0030-transport-memory.md)

- [0031: Repeatable 250-member headroom](0031-repeatable-headroom.md)

- [0032: Full shared UI and ANSI themes](0032-shared-ui-themes.md)

- [0033: Maintained shared service on NetBSD](0033-shared-service-integration.md)

The [native confinement report](0034-process-confinement.md) records kernel peer
authentication, inherited-authority denials and irreversible process ceilings on
the maintained shared service, with new complete 250-member VM measurements.

- [0036 SSH admission and abuse costs](0036-ssh-admission.md) — bounded native listener, base SSH inetd mode and complete accounting.

- [0037 Sustained activity and abuse](0037-sustained-abuse.md) — longer active intervals and admission-work/fairness gates.

- [0038: Bounded SSH key-lookup work](0038-bounded-key-lookups.md) — four charged lookups, native adversarial accounting and whole-host controls.

- [0039: Global SSH admission and private peers](0039-global-ssh-admission.md) — native pre-fork gate, failure and headroom experiments.

- [0041: Reuse-first classic BBS doors](0041-classic-door-reuse.md) — original server/DOS inventory, acquisition unknowns and comparison.

- [0042: Running original BBS games](0042-original-classic-games.md) — native LORD/DOSBox reuse and deferred TWGS activation.

- [0043: File areas and SFTP](0043-file-areas-and-sftp.md); [0044: Home identities and encrypted mail](0044-home-identities-and-encrypted-mail.md) — owner choices, primary standards and bounded follow-ups.

- [Native file-area implementation evidence](../evidence/file-areas-netbsd/README.md) — #83 scoped SFTP, quarantine and complete file-data recovery.

- [0045: Two-home encrypted mail and scoped roaming proof](0045-encrypted-mail-and-roaming-proof.md) — actual reference mail/native verifier controls, dependency gates and native signing failure; #84 remains open.


- [0047: Modern terminal, operator and protocol inputs](0047-modern-terminal-and-federation-options.md) — primary-source inputs for #95–#101 plus the #103/#104 Netnews article/NNTP-first direction; no production wire adoption or runtime evidence.

- [0048: Original INN offline article validation](0048-offline-inn-article-validation.md)

- [0050: Bounded inline Kitty graphics](0050-kitty-graphics.md)

- [0051: Real terminal compatibility](0051-real-terminal-compatibility.md)

- [0046: Native local mail vault](0046-native-local-mail-vault.md).

- [0052: NetBSD certificate-signing exit](0052-netbsd-signing-exit.md) — reproduced stale noreturn annotation, controlled source correction and unchanged installed-issuer boundary.

- [0053: Native OMEMO-2 reuse qualification](0053-native-omemo-reuse.md) — pinned candidate comparison and host build/tests; no product adoption or NetBSD/complete-interop claim.

- [0054: Two-way native OMEMO-2 and NetBSD qualification](0054-omemo-interop-netbsd.md) — rejected original candidate, explicit corrected-source independent interoperability and typed-device/state controls; no product mail adoption.

- [0055: Native XMPP transport](0055-native-xmpp-transport.md) — pinned reuse comparison and actual NetBSD software qualification.

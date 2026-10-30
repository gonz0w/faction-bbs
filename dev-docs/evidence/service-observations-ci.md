# Service-observation CI scheduling correction

This 2026-10-30 record supplements the [service-observation evidence](service-observations/README.md)
for issues #308/#309. It preserves the first hosted failure and qualifies a
scheduling correction. It does not rewrite the original source-bound observations.

## Observed failure

[PR #310](https://github.com/gonz0w/faction-bbs/pull/310) first ran at
`8d813b1204f386bc6871e326ffc7731d9d867150` against
`520f9a45e2118c3ba812b28e6e35573aa41f0c28` in
[run 41101498724](https://github.com/gonz0w/faction-bbs/actions/runs/41101498724).
The provider job completed every selected provider profile and native-client
all-target tests, then was canceled at its 15-minute limit during the final
client-console probe. That probe did not complete. The documentation job correctly
failed its mandatory aggregation because the provider result was canceled.
Native-mail, operator and issue/TDD jobs passed; this was not a successful full CI run.

The three new service-profile Clippy/test pairs occupied 153.944 seconds in the
crowded job. The existing operator job passed in 4 minutes 40 seconds. These
observations motivated redistributing the same work; they are not a guarantee
about future runner timing.

## Correction and checks

Source checkpoint [41e19e3](https://github.com/gonz0w/faction-bbs/commit/41e19e3cb86846739c7cd14f9323e5dc3e2c3fc3)
moves exactly six unchanged commands to the existing operator job after source
qualification and before explicit builds. Both jobs keep complete source warmups.
The same three optional jobs, 15-minute budgets, two compiler jobs, toolchain,
serial tests, native/client/console controls and mandatory aggregator remain.
The independent codec step is byte-identical. No runtime assertion was changed
or weakened; dependency, lockfile and ordinary feature selections remain
unchanged. The CI placement assertions were deliberately updated and extended.

The checker enforces exact placement and one execution of each service command,
including refusal of duplicates in extra steps or jobs. Actual test-first traces:

| Phase | Command selection | Observed result | Output SHA-256 |
| --- | --- | --- | --- |
| Placement RED | `python3 -m unittest discover -s tools -p test_provider_dependencies.py -k ServiceProfileSchedulingCI` | Exit 1: six placement failures and old guard refusal before producer changes | `4688ba5512bdf2711b56a93eb64b3a1bb2c29f5a55d5d20815780684b31c9fa5` |
| Duplicate RED | Same discovery, `-k test_service_pairs_cannot_be_repeated_in_an_extra_step_or_job` | Exit 1: twelve extra-step/job mutations exposed before guard refinement | `629571029e9c66a887d5aff7c362cbe8698b915f93c2c02220205d4cbe697fa3` |
| Provider GREEN | Same discovery without `-k` | 51 tests pass | `82634238e9f1f0df6bffac15490be74363cc2a8f021c32926e1cafbd1603fd9d` |

All 11 codec qualification tests, both actual selected-source guards, formatting,
repository and whitespace checks pass. Full `python3 -m unittest discover -s tools`
passes 635 tests in 84.370 seconds. No Rust/runtime tests were rerun for this
scheduling-only correction: all 31 provider inputs and 376 native execution inputs
remain byte-exact to the original record. Fresh hosted CI remains a separate
integration gate, reported in the PR; these local checks do not establish its success.

## Executed source identity

| Maintained file | SHA-256 at the source checkpoint |
| --- | --- |
| `.github/workflows/documentation.yml` | `02a511a483baafc84f6cb9f46baad8a9486584a0770147d1f1c019d0b496b96a` |
| `tools/check_provider_dependencies.py` | `239faeb3951735e25a04b227761c595e9cf80f7fab87dd8015f1ef87b99e45d1` |
| `tools/test_provider_dependencies.py` | `4d252ffb8c89b55c85430b6ed9bff0727b027c9cec4943bd08151fb87ad1b7a6` |

The current inventory stays byte-exact. Historical producer bytes resolve through
immutable Git objects and retained original captures; no old hashes are refreshed.

## Retention and limits

A separate private archive retains all 134 regular capture files, including original
sources, both REDs, intermediate GREEN, exact commands/logs/exits, scope proof and
hosted diagnostics. Every member was verified before the marked correction scratch
root was removed. Archive: 1,866,128 bytes, SHA-256
`adf2baabcb086c80cb677f4fb1d1074a00331216bbda33c54ec6a8db663d4170`.
Its private file index has SHA-256
`58118fde7a9dd52871419516abc823f4748ed1977bd479edc24ab930febe2c51`.
The earlier private archive and public nine-record manifest remain unchanged.

Full-suite output is retained as tool-poll observations; the initial two progress
dots appeared with the preceding summary in the tool transcript. The final test
count, exit and output are retained. An initial lead-only identity inspection
compared nested records with hash strings; the schema-aware correction verifies
all 376 native inputs, with no product edit or failed product assertion.

Temper used the configured implementer and separate reviewer mappings. Native role
loading and backend model telemetry were unavailable; requested settings are not
claimed as observed. No VM or playground was started, and no host setting changed.
This correction establishes no new NetBSD runtime, hardware, capacity, deployment
or security-audit claim.

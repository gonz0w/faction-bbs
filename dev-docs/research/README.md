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

The [architecture map](../architecture.md) and [roadmap](../roadmap.md) identify the next questions. Full secure-server, target-hardware, real-network and terminal/failure experiments remain to be built.

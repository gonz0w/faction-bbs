# Verification follow-up

The first exact-head CI attempt reached the unchanged 15-minute documentation
job deadline during the existing terminal suite. Native mail, the issue gate,
source guards and tool tests passed, but that attempt did not finish integration.
The [separate verification record](verification-ci-followup.json) preserves this
cancelled attempt and the exact source, actual red/green tests and controls for a
bounded manifest decode cache. Every lookup still reads and verifies all pinned
manifests and the chosen source. Public manifest helpers still return fresh data.

The cache change touches verification tooling only. Its 8 tests first produced
4 intended failures and 4 passing controls; all 32 focused/regression tests and
the full 330-test tool suite then passed. Host before/after guard timings are
observations with overlapping candidate validation, not isolated benchmarks or a
CI pass claim. The original software result and its publication predecessor keep
their exact digests. The owned cache fixture was removed after source preservation.
The PR's current exact-head checks remain the separate integration authority.

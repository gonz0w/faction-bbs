# Testing Map

## Current state

No automated or manual test assets exist in this documentation-only seed. There is no product code, test runner, fixture, CI configuration, or VCS repository. No test command is available or was run.

## Declared future testing constraints

The transport design explicitly calls for unit tests per transport adapter and integration tests covering hybrid scenarios. It also requires graceful degradation on constrained networks, which suggests later tests for asynchronous/store-and-forward behavior, partial connectivity, and transport fallback.

The PRD implies acceptance areas that later testing must cover: persistent chat history and multi-device access, threaded posts/search, role-based moderation, data export/backup/restore, self-host deployment, terminal compatibility, privacy/security, and operation at dozens to hundreds of concurrent users depending on hardware.

## Research/testing gaps

- Define domain and adapter contracts before selecting test frameworks.
- Establish integration environments for at least the initial IP and Reticulum priorities.
- Specify security, backup/restore, import/export, compatibility, and performance validation criteria.
- Separate product acceptance criteria from unverified draft aspirations.

## Evidence

- `docs/initial-seed-prd.md` lines 44-74, 83-101
- `docs/transport-erd.md` lines 44-81, 100-107, 121-132

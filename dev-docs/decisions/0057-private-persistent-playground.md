# 0057: Private persistent terminal playground

Status: accepted for development preview only
Date: 2026-10-22
Owner: owner request after #151; implementation in #152
Issue: [#152](https://github.com/gonz0w/faction-bbs/issues/152), related #21/#71
Supersedes: earlier playground-off request only for this explicitly requested preview
Research/evidence: [GitHub issue](https://github.com/gonz0w/faction-bbs/issues/152)

## Decision and alternatives

Use a distinct private copy-on-write NetBSD software fixture, existing restricted
SSH and current native binaries to give the owner a populated BBS to try. Preserve
the saved older board, local approval, classic fallback and normal resource limits.
Provide repeatable status/start/stop/connect controls and a new-only sample seed.
A public demo fails the requested local/private exposure boundary. An empty local
console would not exercise the requested real SSH login. A general image installer
is larger than this reviewable development deliverable and remains separate.

## Gates and evidence

The applicable decision-framework gates are existing no-JS/local authority,
terminal safety, bounded runtime dependencies, privacy and software-first NetBSD
validation. The seed reuses core APIs, adds no runtime package/schema/service and
never runs in the member entry point. The controller uses Python standard library,
base SSH and operator-installed QEMU; it grants no BBS user host authority.
Private SSH keys and known-host records are prepared independently, not trusted
from a public key scan. Administrative actions target only the registered owned
process identity. Lifecycle/refusal tests and actual SSH/restart observations are
recorded in the plan; unknown production/physical/capacity gates stay open.

## Tradeoffs and reversal

The current prepared image/serial workaround is macOS ARM64/HVF specific, not a
portable appliance. Its overlay needs the saved backing image and kernel. Shared
synthetic demo passwords are confined to this local fixture. Reversal is low for
source changes: remove the optional tools/example. Preserve or export explicitly
wanted test data before deleting the stopped private VM. No historical runtime
measurement, public endpoint, host policy or cryptographic mail decision changes.

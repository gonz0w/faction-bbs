# Read-only board guide: software evidence

[results.json](results.json) records the guide batch under #21/#133. The native
fixture ran NetBSD 11 aarch64 with 2 cores and 1 GiB RAM, using a new isolated
project and the locked offline vendor set. Sources, resources, executed harnesses
and binaries were compared before owned-fixture teardown.

- 369 workspace Rust tests on macOS and NetBSD, strict host Clippy and 130 Python tests.
- 17 local/inherited-worker guide controls per OS, including ANSI and plain 40x18.
- 10 actual shared SSH/host-native/guest-native controls: current keys/roles/gates,
  inert notes, paging, resize, live disable, plain access and revoked navigation.
- Maintained host terminal workflow and native social/Who/idle notification,
  writing/community, client travel/passport/vault and export/restore regressions;
  18 deterministic artwork previews. These are software cells, not GUI approval.

Guard 62 (`python3 tools/check_board_guide_evidence.py`) checks 340 exact native
inputs, 14 harness identities and all assertions above offline. Snapshot 48 retains
335 prior input bytes and the complete immutable passport result. Normal eight
sessions, 5000-ms admission, schema 18 and existing helper/output/crypto limits stay.
Guide access uses the existing visit/heartbeat and broad Menu activity; it performs
no content action. See [the guide](../../../docs/board-guide.md) and
[plan 0081](../../plans/0081-board-guide.md) for scope and validation corrections.

The record covers owned software fixtures only. It makes no new capacity,
hardware, hardening, federation, private-mail or graphical-terminal acceptance
claim. Teardown verifies that the saved owner playground remains off and intact.

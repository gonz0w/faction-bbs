# Full-screen room editing and saved checkpoints: software evidence

[The result](results.json) pins the final actual copied native sources, host
harnesses and installed executable hashes. macOS ARM64 and an owned NetBSD 11
ARM64 VM (two cores / 1 GiB) validate software, not physical SBCs or new capacity.
Read [the product contract](../../../docs/writing-desk.md),
[ADR 0054](../../decisions/0054-room-editor-and-checkpoints.md) and
[plan 0059](../../plans/0059-full-screen-and-room-checkpoints.md).

## Results

- **213 Rust tests per OS**, host Clippy with warnings denied and formatting;
  **118 host Python tests**. Schema-8 tests cover ownership, quotas, expiry,
  revision conflicts, atomic send/retry cleanup, old schema-7 exports/upgrade,
  malformed snapshots, current approval and restore/session boundaries.
- **Seven full-screen/checkpoint PTYs per OS**: caret/paragraph editing, preview,
  explicit save, disconnect/resume, subject retention, confirmed send, plain/worker
  routes, resize reflow/fallback and unsupported-input recovery. Existing thirteen
  unsaved line-writer controls still pass per OS; their legacy result fields
  describe those deliberately unsaved test journeys, not absence of checkpoints.
- **Eleven actual restricted-SSH/native/plain controls**: full writer/checkpoint,
  reconnect/confirmation, host/native NetBSD Unicode, plain save/resume/cancel,
  resize, concurrent save conflicts, exactly-one-post busy retry, pending/revocation
  and stale reply refusal without redirect/body disclosure plus explicit deletion.
- **Eleven backup/check/restore controls per OS**; native proof runs the actual
  CLI with distinct non-root owner/file UIDs under a marked root-owned fixture.
  Host uses the developer API fixture. Export/verify/restore CLI passes per OS;
  core tests verify the checkpoints themselves travel in owner snapshots.
- Ordinary browser ten per OS, local account/menu/reader regressions per OS,
  shared UID/PTY reader 22; native directory eleven, launcher seven, operator
  eight, artwork previews eighteen, local/worker art seven, and actual SSH/native
  artwork seven all pass.

**184 selected native input hashes** match the actual copied bytes, including
schema, tests, editor, build inputs and backup/export harness dependencies.
**Seventeen host harness inputs** are pinned. **Twenty-five dependency/access/
credential inputs** remain unchanged. All four installed native executables match
the clean native build; a checkpoint literal and the real UI routes verify that
implementation was exercised. Snapshot 26 freezes 180 previous actual #148 inputs
and its entire unchanged result. All 26 snapshots/39 historical identities remain
fixed; no earlier measurements were refreshed.

## Failure found and corrected

Initial resize tests demonstrated that resetting the editor decoder between the
ESC and `[D` bytes of an arrow inserted `[D` into the body. The final implementation
preserves partial-key/scalar decoding across fitting resizes. Too-small/unsupported
input returns to classic behind an Enter barrier, discarding delayed fragments.
The exact failing synthetic body was retained only in a private diagnostic; no
failed candidate is counted as passing. Both native and host final PTYs exercise
the corrected resize path, with an additional fragmented-key unit regression.
A test also initially expected an ANSI main menu after a 30x12 resize; it now
checks the actual required plain menu. No output/admission/credential limit was
relaxed to pass. Final native workspace artifacts were cleaned before rebuilding.

The stale-parent test deliberately removes only a synthetic post/metadata pair
through the marked fixture's admin connection, then confirms resume refuses the
missing target without displaying its draft and permits deliberate checkpoint
deletion. It is not a member deletion API or a new host capability.

## Limits and cleanup

Checkpoints are explicit, sysop-readable local-room plaintext. No autosave,
member-side mail plaintext/key store, secure erasure, remote draft synchronization,
member draft export or external editor is claimed. Schema 8 needs the documented
stopped complete backup and fresh-path rollback. Logical 30-day expiry is lazy;
inactive rows/WAL/older backups may retain data.

Normal eight sessions, 5000-ms admission, two credential permits, three helpers,
32-KiB shared output and five-second drain remain. Editor frames cap at 8 KiB;
bodies at 1024 UTF-8 bytes / 32 lines; four saved rooms/member. No new registry
package, listener, measured RSS delta, fresh 250-member capacity, physical device
or hardening claim. The owned VM/cache are stopped/removed; saved playground
remains off. Private credentials, profiles, raw transcripts, VM images and temporary
vendor artifacts stay outside Git. Offline guards validate saved evidence only.

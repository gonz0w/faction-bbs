# 0054: Verified backup checks and the local Recovery panel

Issues #95/#18/#98. Status: implemented and software-validated; PR handoff pending.

## Goal and boundaries

After checked #129 (head 2fc49ec, merge 1698b38), make recovery visibility useful
without fabricating backup health or adding privileged dashboard actions. Add a
verify-only operation for existing complete file-area bundles, an explicit
`--record-status` option for backup/check operations, and a D:Data/Recovery panel.
Keep the existing stopped/drained backup and fresh-path restore contracts. The
receipt describes an operator-selected artifact check, not provenance, current
board identity, live artifact existence, successful restore, off-host replication
or inclusion of themes/host/device keys. No dashboard DB/blob reads, new listener,
owner thread/session/credential limit or host policy change. Hardening stays later.

## Plan

- [x] Merge #129 only after exact-head required check SUCCESS.
- [x] Before edits preserve 162 distinct current source/harness inputs and the
  whole host-stat result in twenty-first snapshot `sources/pre95-recovery`.
- [x] Share complete bundle verification between verify-only and fresh restore;
  retain all catalog/blob/manifest validation and stopped backup authority.
- [x] Publish bounded owner-private numeric receipts only when explicitly asked;
  failure must not erase last successful check and unsafe/unknown data fails.
- [x] Add read-only Data panel, age/clock/failure/unrecorded labels and help,
  retaining compact/plain and current host graphs.
- [x] Exercise actual CLI bundle/check/restore and UI on host and owned NetBSD
  software clone; record measured sources/resources, preserve old identities.
- [ ] Update docs/README/issues, clean owned clone, focused PR; no future PR merge.

## Validation and decisions

Use existing Python standard-library backup tool plus safe Rust receipt reader.
Root backup helper drops to the owner for existing Rust catalog verification;
ordinary dashboard remains an explicitly launched owner/root command. Verify-only
needs no destination reservation, member/file service start or lock mutation.
Receipt has fixed version/order/numeric fields, maximum 512 bytes; no filenames,
paths, identities, private data, remote addresses, commands or secrets. Display the
last attempt and last successful check separately. Future clock/read failure is
not fresh success. A missing/failed receipt never blocks BBS operation.

Native SDK/runtime tests use a fresh owned two-core/1-GiB NetBSD 11 ARM64 VM.
Never boot/migrate the saved original, enable public listeners or edit user login
profiles. Keep private fixture credentials/admin argv/logs outside Git. No physical
SBC, production-hardening or new 250-user claim. Peer health waits for real adapters.

## Outcomes

- 184 Rust tests per OS, 118 host Python tests, host Clippy/format pass.
- Eleven backup/check/restore controls per OS. Developer APIs use one fixture UID;
  native production CLI runs as root with distinct owner/file UIDs and Rust
  verifier dropped to the owner. Verify-only omits the file UID and leaves bundle
  bytes/destinations unchanged. A connected local session survives the check.
- Corrupt bytes record FAILED while retaining prior verified metadata; restore
  refuses before reserving paths. Valid restore returns exact bytes and working
  login, excluding telemetry. Unsafe/future/missing receipt states remain explicit.
- Eight themed UI controls per OS, ten shared operator/SSH/lifecycle/resource,
  four serial/getty controls and seven actual shared art routes pass; native
  previews eighteen/local artwork routes seven. Data panel is included in arrows
  and serial controls; classic/40x18/plain fallback is retained.
- Twelve half-second default-palette samples with one member: dashboard 2,436 KiB
  RSS / one LWP; shared owner 5,156 KiB / four LWPs; minimum whole-VM free 333.37 MiB.
  This is a short software observation, not supported 250-member capacity.
- 159 selected native inputs match execution tree; fourteen relevant host harness/
  test inputs pinned. Existing 162-input baseline and all old result hashes stay
  unchanged. An initial fixture invocation used the private builder output path;
  switching to the installed root-owned binaries restored normal owner traversal
  without loosening any permissions. No product-policy exception.

Docs are updated; owned VM shut down and clone/vendor cache removed after PID/
disk/closed-handle checks. Saved original has no open handle. All fifty offline
policy/evidence/dependency checks pass, including thirty-five saved-evidence guards
and twenty-one snapshots/thirty-four unchanged historical identities. Formatting
and clean diff checks pass. PR/issue handoff remains the final step.

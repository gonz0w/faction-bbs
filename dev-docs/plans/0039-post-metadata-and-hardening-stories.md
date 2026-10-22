# 0039: Persistent post metadata and later hardening stories

Issues #103, #106–#111. Status: implemented and validated, 2026-10-21.

## Outcome and constraints

Capture later-phase signing/VM/process/storage/verification stories without
activating hardening or changing current authority. Continue development with
schema-6 post metadata: public stable article identifiers, generated subject labels,
actual timestamps for new posts and explicit unknown legacy dates. Preserve
bodies/IDs/replies/unread, file namespaces and private restore identity rotation.
Show metadata in current room screens and validate old/new exports and rollback.
No network publication, custom crypto, extra owner helpers or weakened access.

## Progress

- [x] Log six bounded later-phase hardening stories with source/parent references.
- [x] Freeze all 87 #105 executed inputs and its original result before edits.
- [x] Implement transactional metadata migration, atomic new-post metadata and UI.
- [x] Verify migration failure/retry, approval, stable IDs and old/new snapshots.
- [x] Run Rust/tooling/real terminal and native software checks; update docs/PR.

## Decisions

Keep the existing local owner-open migration model. Operators must stop/drain,
make a verified complete backup and upgrade binaries together before first open;
this is not an online/mixed-version upgrade promise. Migration labels old records
without invented dates. Local IDs are public references, never credentials or
home identity proofs. Wire injection/From/Path/group consent and author-entered
subjects remain follow-ups under #103/#101/#104.

No PGP keys, NPF/VM changes or jail activation in this story capture. NetBSD chroot,
FreeBSD jails and VM boundaries have distinct capabilities and residual risks.
The database owner can access its authorized plaintext; offline encryption is
not live-process confidentiality. Member mail keys stay on member devices.

## Actual outcomes

121 workspace Rust tests pass on macOS and native NetBSD 11 ARM64, including six
metadata cases and bounded inert header coverage. Build/format/host clippy pass.
Real local reader and full console checks pass on both systems; host worker and
export/restore flows pass. Five native shared controls include subject/UTC display,
partial/full unread behavior, ordered output and current revocation. 104 Python
tooling tests pass. No new dependencies, owner helpers, credential/default changes
or supported-capacity claim. Old measured source/result identities stay fixed.

The current record pins 90 real executed source inputs. The disposable VM clone
was shut down and removed; the owner's stopped playground/checkpoint remain intact.
Later hardening stories #106–#111 are logged and linked, with no host/key/firewall
actions. Current guidance documents stopped/drained backup/upgrade and fresh-path
rollback; old exports stay read-only.

#103 remains open for authored subjects, full From/Path/group projection, broader
MIME and independent peer/provider interop. No automatic publication or live
network adapter is enabled. This branch stacks on #105; #94 stays independent.

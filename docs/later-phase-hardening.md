# Later-phase hardening
Status: requested backlog, 2026-10-21. These are later design/build/validation
stories, not currently adopted hardening or a guarantee against compromise.

| Story | Issue |
| --- | --- |
| Design OpenPGP signing and offline verification for releases and appliance images | [#106](https://github.com/gonz0w/faction-bbs/issues/106) |
| Research optional member-held OpenPGP signatures for discussion articles and files | [#107](https://github.com/gonz0w/faction-bbs/issues/107) |
| Design and validate a hardened NetBSD VM appliance profile | [#108](https://github.com/gonz0w/faction-bbs/issues/108) |
| Prove post-compromise process containment and shell-escape resistance | [#109](https://github.com/gonz0w/faction-bbs/issues/109) |
| Protect databases, file storage, backups and secrets across component compromises | [#110](https://github.com/gonz0w/faction-bbs/issues/110) |
| Run independent hardening validation, fuzzing and compromise-recovery exercises | [#111](https://github.com/gonz0w/faction-bbs/issues/111) |

## What the phase must prove

Keep a useful terminal BBS while adding layered, measured defenses. Begin with
component threat models and explicit filesystem/network/exec/IPC/data authority.
Assume compromise separately under frontend, file worker, game/adapter, board
owner, guest root and hypervisor identities. Confinement must have positive
controls, real denied escape/data probes and independent review before an adopted
hardened profile is claimed. Reuse #55/#23/#29/#30 and existing experimental evidence.

Keep OS operator console/management credentials separate from members and BBS
roles. Preserve intentional emergency console/recovery access while denying member
shell/exec/forwarding and unintended host consoles. Narrow privileged setup, no
ambient root API, no shared host mounts/sockets or game access to board state.

NetBSD remains primary. Its [chroot syscall](https://man.netbsd.org/chroot.2)
changes the filesystem root for path lookup; that alone establishes no complete
process/network/kernel boundary. [FreeBSD jails](https://docs.freebsd.org/en/books/handbook/jails/)
are a separate operating-system facility to compare, including optional isolated
game hosts; they are not a NetBSD API or a promise to port the whole BBS. VM,
guest, hypervisor and shared-kernel risks need separate validation.

## Signing, keys and storage

Capture both requested OpenPGP directions separately: release/image provenance
and optional member-held content signatures. [RFC 9580](https://www.rfc-editor.org/rfc/rfc9580)
is the current format reference; maintained interoperable implementations and a
complete no-JS dependency review are required. No custom crypto or automatically
trusted key-server identity. Define canonical signed bytes/context, fingerprint
bootstrap, expiry/revocation/rotation, replay/rollback and failure UI. Signatures
do not grant local approval or prove software is vulnerability-free. Release keys
stay outside the live BBS; member signing/decryption keys stay on member devices.
Signing is not a replacement for the encrypted XMPP private-mail boundary.

Inventory DB/WAL/SHM/journal, file blobs/partials, exports/backups, VM snapshots,
logs/temp/swap/core files and credentials. Separate UIDs, restricted IPC, mounts,
quotas and link/race-safe operations should contain less-privileged compromises.
[NetBSD CGD](https://www.netbsd.org/docs/guide/en/chap-cgd.html) is an offline-storage
candidate with explicit unlock/reboot/recovery policy. An unlocked process that
legitimately reads local plaintext can expose it if compromised; disk encryption
and chroot do not make the shared database owner blind. Member-device E2EE remains
separate. Validate confidentiality, integrity and consistent backup/restore under
corruption, disk-full, malicious replacement and stale-policy recovery.

## Sequencing and limits

Keep core product work moving. This later phase must preserve current approval,
SSH, credential/session/helper limits and local-by-default data. It does not enable
NPF, change a running VM/host, generate/publish keys or introduce a release/update
service. #97/#98 own reviewed firewall/setup plans and recovery. Software/VM
fixtures with synthetic data come first; physical hardware is not a software gate.
No public/third-party probing or absolute “unbreakable”/“fully locked” claims.

Primary references retrieved 2026-10-21; linked upstream documentation is a research
input, not a tested Faction deployment. GitHub issue state is authoritative.

# 0033: Native terminal transfers and local file references

Status: accepted bounded implementation choice, 2026-10-21. Issues #84/#87;
parent mail/identity proof #84 remains open. Owner asked for transfers to feel part
of the BBS, room/chat links, and encrypted attachments retained in the plan.

## Decision and alternatives

Build a separate local Rust terminal client around base OpenSSH, a client-only
PTY bridge and a bounded SFTP v3 stream. Reuse the existing independently scoped
file service. Negotiate file controls only in the approved Files screen; use the
current connection's member/epoch, a random local nonce and monotonically increasing
request sequence. The public key is key-binding input, never an identity assertion.
No remote endpoint/path/shell execution or automatic transfer comes from frames.
The local user selects a path and confirms before streaming.

Manual SFTP remains a useful ordinary-SSH fallback but interrupts the BBS experience.
Embedding another SSH implementation would increase crypto/dependency review for
no demonstrated benefit in this increment. Running SFTP or native client processes
inside the shared owner would violate its confinement/resource boundary. A browser
terminal conflicts with the current no-JS/TS constraint. We choose the base SSH
process on the **member device**, plus one temporary transfer child there.

Schema 5 adds a stable public random file namespace independent of rotating
session identity. `faction-file:NAMESPACE/OBJECT` is a local semantic reference,
not a URL, capability or network locator. Up to four per room/chat message;
publishing validates the sender's current access. Read rendering rechecks the
viewer and redacts inaccessible metadata; download rechecks again. Restore keeps
public links but rotates private authority and discards transfers. No federation,
public blob URL, file copy or remote fetch follows from sharing a reference.

## Validation and limits

See [current evidence](../evidence/native-client-netbsd/README.md) and
[GitHub issue](https://github.com/gonz0w/faction-bbs/issues/87). Real host and native NetBSD
clients exercise login/approval, menus, upload/review/download, no-overwrite/failure
cleanup, per-reader room/chat redaction, recent links and revocation. Rust tests
cover versioned framing, terminal controls, profile permission/options policy,
public namespace restore and scoped cancellation. The full catalog/blob backup
is also rehearsed with schema 5.

Confidence is integration-level, not an independent security review. No hardware,
all-Unix support, release/capacity claim, E2EE or roaming auth. Client-only C/PTY/base
OpenSSH remain trusted; ordinary BBS owner limits/default eight/credential controls
are unchanged. Client forms currently restrict paths to ASCII, the local Files
palette is fixed, and transfer resume remains deferred. A malicious same-user
local process is outside the local filesystem trust boundary.

Primary sources retrieved 2026-10-21: NetBSD [openpty(3)](https://man.netbsd.org/openpty.3)
for client libutil PTY creation and [ssh(1)](https://man.netbsd.org/ssh.1) for fixed
base SSH transport/subsystem/TTY behavior. Existing SFTP profile is ADR 0032.

Reversal cost is moderate: ordinary SSH/manual Files remains, and disabling the
client changes no server deployment. Older binaries cannot open schema 5; preserve
pre-upgrade backups. A future SSH adapter can replace the local transport while
retaining semantic file operations. Reopen for non-Unix clients, resume, remote
file federation, client themes or a proven need for embedded SSH. E2EE attachments
must use separate ciphertext delivery/key interfaces under #88, not current areas.

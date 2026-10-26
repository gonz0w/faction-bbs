# Encrypted local unread state and parked Files mail

Issues [#216](https://github.com/gonz0w/faction-bbs/issues/216) and
[#217](https://github.com/gonz0w/faction-bbs/issues/217) are finite children of
[#99](https://github.com/gonz0w/faction-bbs/issues/99). The
[member guide](../../../docs/local-unread-mail.md) and
[technical contract](../../local-mail-read-state.md) describe the implemented
behavior. [The pinned result](results.json) contains software observations.

## What was observed

Host and native NetBSD each exercise 13 local unread terminal controls. Explicit
FACTCP04 opt-in and received-only read decisions remain encrypted and atomic.
Counts and body inspection do not silently acknowledge a message; a fresh typed
`read` decision applies only to its authenticated received record. Duplicate,
pending, cursor, recovery, failure and process-death behavior are tested separately.

The combined journey has 14 controls in both classic and Unicode modes on each
OS. One client and one SSH child enter an independently approved BBS, park at the
native Files desk, open local mail after confirmation, encounter real home TCP
refusal, read local history/outbox, explicitly upgrade and acknowledge received
mail, lock and resume ordinary Files RPC. The same visit uploads a deliberately
public payload, receives sysop review/publication and downloads it through
restricted SFTP with an exact checksum. Local unread moves from one to zero while
history, every pending ID and the full nonempty archive cursor remain exact.

Notices start off. Enabling starts unknown; only a subsequent graceful mail lock
supplies a stale device-local snapshot. Opt-out clears it. Notices stay out of
transfer forms/progress. Three additional real Files controls per OS cover
unconfigured default, native-only and network builds; configured unsupported
builds retain their earlier early-refusal checks.

An owned, verified guest TTY writer injects at most 1 MiB in at most 32 KiB chunks
under its own short deadline. Board output remains absent from hidden local
prompts and the private view. A deliberate recorded SSH-child termination during
mail then fails closed without redial after the private handle locks. The kernel
PTY supplies bounded backpressure; indefinite suspension of an arbitrary remote
stream is not promised.

Actual remote-only transcript intervals and board database/WAL, file-store and
service-log files are scanned for synthetic private mail markers. No matches were
found. Private console intervals are deliberately separate from board output.
This is an executed marker test, not a general proof against every possible leak
or a compromised member device. Public SFTP content is not an encrypted attachment.

## Source, budgets and preserved corrections

[The prior picker baseline](../picker-mail-unread-baseline-manifest.json) binds all
684 #214 source identities to existing immutable objects and pins its unchanged
whole result. No prior measurements or hashes were refreshed.

Native server code is built from committed #215; client and runtime maps retain
separate identities and one explicit Python-only fixture override. Rust/C/Cargo
and executable identities remain exact across that override. Compiler work uses
two cores/2 GiB; runtime qualification reboots the same owned overlay at 1 GiB
(1,054,990,336 physical bytes). This is not a fresh 1 GiB compilation, physical SBC,
new RSS, throughput, capacity or production-support claim. Normal session,
credential, helper, Argon2, file, output and drain limits remain unchanged.

A preliminary host UI pass lacked a composed main/module source map and is not
used as final acceptance. A copied host inventory also contained eight ignored
Finder metadata files; their contents were never published. The original inventory
was not rewritten. Final host acceptance is repeated from a new clean frozen
source tree. Qualified NetBSD source maps contain no Finder or Python cache files.

A first live probe failed TTY discovery because NetBSD `ps` treated comma-separated
empty-header formats differently. The executed failed source is retained; portable
named columns fix the fixture. Notice timing was corrected during preflight to
respect opt-in-before-snapshot semantics. A final legacy compatibility attempt
observed normal terminal admission denial after a burst; its failure remains
preserved. A separate successful retry waited the normal refill window, retained
the old pending account and used a new synthetic member. No limit or board-state
reset was used, and token-level exhaustion was not directly measured.

Only public source bytes, fixed-schema control results, counts and digests are
published. Raw terminal logs, runtime settings, generated credentials, private
keys, message bodies and opaque cursor values are excluded. Verified teardown
removes only the marked owned fixtures, VM/controller and test services; the saved
owner playground remains stopped and unchanged.

Run `python3 tools/check_unread_mail_evidence.py` to validate saved record identity,
source bindings, both rendering modes, unconfigured feature graphs, exact private
state, bounded remote handling and cleanup. This checks saved evidence; it does
not rerun NetBSD. Required PR CI is separate.

#99 retains background/unread push, broader active-session continuity and private
live chat. Roaming #91 and encrypted attachments #88 remain separate. No server
read receipt, mailbox-completeness, security-audit or deployment claim follows.

## Verification follow-up

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

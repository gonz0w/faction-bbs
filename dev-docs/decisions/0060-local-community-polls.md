# 0060: Local community polls and explicit ballots

Status: accepted for the hobby preview, 2026-10-23. Issue: #139.

## Context

Small boards need lightweight feedback and weekly questions without a web survey
service. Keep ordinary SSH, approved membership, local data and the ANSI vibe.
Voting must not imply private mail, secret ballots or election-grade integrity.

## Decision

Schema 12 adds optional local poll policy, polls, numbered choices and one ballot
per member/poll. Default disabled, sysop-only creation; optional approved-member
creators. Use one shared `faction-polls` workflow with core/terminal dependencies
and local/worker/shared adapters, following wall/pages/gatherings. No host/network
I/O authority, new registry package, owner helper, listener or resource-default change.

One question, 2–8 distinct choices, single choice, open vote changes/withdrawal.
First vote permanently locks question/choices/result policy, including after all
withdrawals. Only the creator edits/closes/cancels; close/cancel are irreversible.
Sysops independently hide/show/remove without rewriting authorship. Revisions
protect editorial/moderation/state changes and reviewed ballots. Vote transactions
serialize changes; vote updates do not rotate the wording revision. Result display
is always/after own vote/after close, with no creator/sysop bypass and no voter list.
All ballots remain owner-readable plaintext included in backups. Current approval,
creator status and visibility apply to every screen/action; revoked-voter rows are
excluded from counts but retained for quota until withdrawal/removal.

Limits: 32 retained polls, four open per ordinary creator, 250 retained ballot rows
per poll; question 96 ASCII bytes, choices 48 each, notes 240/four <=72-column lines.
Manual retention/closing avoids a scheduler or time-policy dependency. Read-only
transactions keep previews/results usable during Busy writes; an explicit Save
retains its form/request key for retry. No autosave/checkpoint or unread-ledger writes.

The host inserts optional V, while a trusted theme may place `polls`. A disabled
capability stays unavailable even if a theme declares it. Bounded choice/note pages
and `#`/numeric result bars work in plain and ANSI terminals, including 40x18.
Stale reviewed choices are rejected before indexing current choices during render.

## Alternatives and consequences

Multiple selection, grouped questionnaires, deadlines, secret ballots and federation
add policy/state not needed for the first community question. They remain separate
increments. Server-blind mail cannot be used as a server-side ballot-decryption
shortcut. Aggregate display rules do not hide owner data or prevent duplicate humans.

A schema migration is justified by persistent ballots and irreversible wording locks.
Use stopped/drained complete pre-upgrade backup and fresh-path rollback. Old schema-
2..11 snapshots remain exact/read-only valid. Deletion removes local rows, not media
or old backup copies. Votes on a hidden or revoked creator's poll cannot continue.

## Validation and reversal

See [plan 0065](../plans/0065-local-community-polls.md) and
[NetBSD software evidence](../evidence/polls-netbsd/README.md). Snapshot 32 preserves
the preceding 228 measured/harness inputs and whole gathering result before edits;
all older records remain fixed. Remove the optional UI or restore a complete older
bundle with matching binaries in fresh paths. No in-place downgrade, supported
capacity, physical SBC, federation or hardening claim.

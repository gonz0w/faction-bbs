# 0058: Deliberate local vault entry between board visits

Status: accepted development slice
Date: 2026-10-22
Owner: owner requested remaining PR merges and continued useful development
Issue: [#90](https://github.com/gonz0w/faction-bbs/issues/90), [PR #94](https://github.com/gonz0w/faction-bbs/pull/94)
Supersedes: none; extends local entry under ADR 0035
Evidence: [GitHub issue](https://github.com/gonz0w/faction-bbs/issues/90)

## Decision, gates and alternatives

An optional `mail_store` in the private member-owned board catalog adds M to the
local picker. It opens an existing encrypted offline vault before any SSH child
starts, or after the previous child has closed/reaped. Its own hidden passphrase,
current core-dump refusal, vault locking/zeroization and explicit backup/recovery
remain unchanged. Locking returns to the same board selection. Missing/busy/bad-
secret stores return to the picker; they do not auto-initialize or recover.

The mandatory gates are member-held plaintext/keys, no remote unlock/config/trust
authority, no-JS, classic terminal utility and bounded resource/dependency work.
Catalog validation reuses its private bounded local-file boundary. The local
screen is never an SSH/RPC action; no board output chooses its path. Current
network-mail, OMEMO, home-proof/roaming and device-encrypted attachment gates stay
open. Existing approved local community membership remains independent.

Standalone commands remain useful for initialization/recovery. Opening private
mail inside an ordinary remote SSH BBS would violate member-side decryption.
Suspending a live SSH stream under an unlocked local screen adds concurrent
output/typeahead/lifetime complexity; deliberately leave the board first using
the existing confirmation. This keeps a simple local lifetime without new servers,
threads, protocol messages or registry dependencies beyond reviewed #94 packages.

## Validation, limits and reversal

Use real PTYs on macOS and native NetBSD for initialization, explicit picker
unlock, encrypted unsent drafts/reopen, wrong-secret/cancel/missing-store return,
40x18 sixteen-board menus, no child during unlock and unchanged graphics/launcher
regressions. Existing thirteen vault checks retain backup/recovery/pin/tamper/
concurrent-lock coverage. Actual results belong in the plan/new evidence record;
old measurement records and both source sets stay immutable.

This is an offline vault convenience, not integrated network private mail or an
independent crypto audit. The passphrase lives only on the member device; terminal
scrollback/OS compromise and old backups retain their existing limits. Reversal
cost is low: remove the optional catalog setting; keep standalone vault commands
and the unchanged encrypted data format. No server upgrade or data migration.

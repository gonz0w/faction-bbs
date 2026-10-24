# Room writing desk software evidence

The machine-readable [result](results.json) pins the actual selected copied
native sources, host harnesses and installed final binaries. This is software
validation on macOS ARM64 and an owned NetBSD 11 ARM64 VM (two cores / 1 GiB),
not physical SBC or new 250-member/capacity evidence. Normal eight-session,
5000-ms admission, two credential permits and output/drain limits stay unchanged.

The implementation adds only an opt-in room writer: paragraphs, undo/clear,
fresh authorized parent quotes, paged inert preview and explicit send/cancel.
There is no saved draft, new schema, runtime package, listener or private-mail
plaintext. See [the writing guide](../../../docs/writing-desk.md),
[GitHub issue](https://github.com/gonz0w/faction-bbs/issues/141) and
[ADR 0053](../../decisions/0053-explicit-room-writing-desk.md).

## Validation

- 199 Rust tests per OS; host Clippy with warnings denied and formatting pass.
- 118 host Python tests and all repository/evidence policy checks.
- Thirteen local/worker writing controls per OS: paragraphs, preview, explicit
  send, quotes, empty/byte/line caps, cancel, worker TTY, disconnect and Unicode.
- Eighteen real shared restricted-SSH/native/plain controls: selected-room entry,
  the writing journey, both host/native NetBSD clients, resize, reconnect,
  same-key busy retry, pending/revocation and five plain-terminal controls.
- Ten local/worker browser controls per OS and existing full local account/menu
  and room-reader regressions; 22 shared UID/PTY reader controls.
- Native directory eleven, launcher seven and operator eight controls;
  eighteen artwork preview profiles, seven local/worker artwork routes and
  seven real restricted-SSH/native artwork routes.

174 selected native inputs match their actual copied bytes; eleven relevant host
harnesses are pinned. Thirty-two core/build/frontend inputs remain unchanged.
No measured RSS delta or fresh resource-capacity claim is made by this batch.

## Provenance and interpretation

Snapshot 25 (`sources/pre141-writing-desk`) preserves 174 original #147 inputs
and its whole room-browser record before edits. All 25 source snapshots and
38 historical result identities stay unchanged. The new record must pin its
own actually executed bytes; never substitute current files into older records.

Native workspace artifacts are cleaned before the final build. Source hashes,
final installed executable hashes and an actual writing-desk literal check bind
the tests to that build. Private synthetic state/credentials/SSH profiles, serial
logs, failed-attempt fixtures and raw transcripts remain outside Git. A stopped
failed fixture is moved aside rather than unlinking live locks. The earlier
harness expected a local registration prompt on shared SSH; it was corrected to
the actual existing shared flow before accepting the final result. Long preview
checks answer paging prompts; this was a test-oracle correction, not a runtime
budget change. Copying a formatted test requires its final native rerun too.

Preview/quote checks verify no durable unread rows and no submitted post until
explicit send. The busy test holds a transaction on this private synthetic board,
then releases it and accepts the existing same-key retry; exactly one new row
must result. Pending/revoked members cannot enter/send. Real shared plain sessions
use a stopped temporary configuration and restore the original before cleanup.

The saved playground is never started. An owned disposable clone and reviewed
private vendor cache are stopped/removed after evidence collection. This record
is not a claim of full-screen caret editing, saved-draft recovery, encryption of
local room drafts, new news/mail federation, arbitrary-terminal certification or
new hardening. #141 remains open for its broader acceptance criteria.

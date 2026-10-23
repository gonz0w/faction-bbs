# 0055: Bounded local community wall

Status: accepted for the hobby preview. Issue #134; owner selected local community
features and classic fallback, with hardening deferred. 2026-10-22.

## Decision

Implement a separate local wall rather than projecting room articles, live chat or
private mail into a second view. The wall has ephemeral notes, own edits/deletion,
sysop removal, configurable quotas/cooldown/retention and optional weekly editions.
No federation, feed/provider, notifications, paid extension or cryptographic
private-mail behavior follows. Member input is inert printable ASCII/newlines;
ANSI colors/clears come exclusively from the trusted theme renderer.

SQLite schema 9 stores one policy, at most 128 notes per edition and at most twelve
retained old editions plus current; each note is <=240 bytes/four <=72-column
lines. Payload upper bound is 399,360 bytes, excluding SQLite/catalog/index/author
and receipt overhead. Retain one fixed-field cooldown/recent-request row per
member. This does not promise a total database or process memory bound.

UTC-Monday rotation and expiry are lazy transactions, without resident jobs.
Persist the current edition so clock rollback cannot reopen a past week.
Changing weekly mode clears wall content after explicit UI confirmation; archives
are opt-in and honor the independent day limit. Lowered quotas prune content.
Deleting does not reset cooldown or preserve a second plaintext message history.
Retained unchanged same-key sends are idempotent; newest deleted requests are
refused. Older pruned requests have no permanent receipt guarantee.

Core APIs recheck board/member/epoch/approval and current roles inside mutations;
own edit uses expected-body compare-and-swap, removal targets the shown stable ID.
Only explicit Send creates a request key, retained on Busy. A stale edition cannot
receive a reviewed composition silently. Ordinary plain/ANSI/worker and shared
terminals use one bounded workflow in the new faction-wall workspace crate,
depending only on existing core/terminal crates, with no I/O/network/host authority.
The shared service allocates that workflow only while the wall/settings is open.
Theme copies for the optional menu are temporary and absent on the disabled path.

## Alternatives and consequences

Reusing room storage would entangle unread/article/reply/export semantics and
suggest future NNTP publication. Reusing chat would mix ephemeral live delivery
and join-scoped history with an intentionally persistent local hangout. Native-only
wall drafts would exclude ordinary SSH; instead wall drafts remain unsaved and
short. Room checkpoints and encrypted native mail keep their own contracts.
A declarative optional main-menu action allows a sysop's own key/name/palette;
menus beyond eleven entries use a bounded nine-row scrolling lightbar window.

New schema data participates in validated owner export/restore and complete
bundles. Older exports remain verifiable/read-only; opening restored old boards
upgrades them. Stop/drain, verified complete pre-upgrade backup and fresh-path
rollback remain required. Logical retention/deletion is not secure erasure.
No independent sandbox, supported host capacity or protocol adoption is claimed.

## Evidence and reversal

See [plan 0060](../plans/0060-local-community-wall.md) and
[member/sysop behavior](../../docs/community-wall.md). Tests must cover two board
policies, current authority, control/overflow rejection, edit conflict, stable-ID
moderation, explicit archives/deletion, classic fallback and actual native flows.
Snapshot 27 freezes 192 previous measured inputs and the original #149 record;
all forty old result identities remain unchanged. Turning the wall off is
reversible; removing schema 9 requires fresh-path restoration of a pre-upgrade
backup with its old binary, rather than dropping new tables in a live board.

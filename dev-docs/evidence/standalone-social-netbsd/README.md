# Standalone visit lounge software evidence

[Results](results.json) record authenticated local login/inherited-worker ANSI and
plain terminal journeys on the host and a marked NetBSD 11.0 ARM64 software VM
(two cores/1 GiB). Members register and receive actual in-BBS approval. No public
service or physical hardware is involved.

Twelve controls per OS cover the common opt-in Lounge, retained visit lease and parted chat
channels, safe menu refresh, explicit accept/decline/sysop paging, public bounded
status, current block/mute/UTC hours/resize, owner naming/custom theme hotkey,
escape-free plain inherited worker/local access, fresh hidden login, clean logout,
worker transport EOF, revocation/terminal restoration and a deliberately locked
chat-exit cleanup that must terminate instead of resuming retained joins. One measured 66-second
wait per OS runs three states in parallel: local menu, inherited-worker full-screen
editor and plain canonical room prompt. Original unfinished input survives pages.
Cooldown tests separately age owned fixture timestamps; quick preflight mode is
explicitly excluded from sustained evidence.

The Visit owner borrows the existing Board, with no additional connection,
helper/thread, listener, dependency or schema. Current lease/epoch/privacy bounds
remain. Abrupt process termination still relies on original stale expiry. Shared
service sources remain unchanged; its earlier live evidence is retained separately,
not relabeled as a new shared test. Shared-only transfer/game adapters and optional
idle notifications remain separate. Complete recovery and maintained terminal
regressions also pass.

Snapshot 43 and [its manifest](../boardwide-social-baseline-manifest.json) preserve
326 previous input files and the whole boardwide result, retaining all 56 historical
identities. Final selected sources and native/installed binaries are pinned.
The owned VM/private inputs/vendor are removed after validation; saved owner
playground stays stopped with its disk/login/keys preserved. No new capacity,
physical-SBC, private-message-body, federation, secure-erasure or hardening claim.
See [GitHub issue](https://github.com/gonz0w/faction-bbs/issues/137) and
[ADR 0071](../../decisions/0071-standalone-social-visits.md).

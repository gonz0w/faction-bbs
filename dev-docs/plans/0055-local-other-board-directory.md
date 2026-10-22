# 0055: Curated other-BBS directory and connection cards

Issues #27/#21. Status: implemented; validated; PR handoff pending.

## Useful member-facing slice

After checked #130 (head 315b098, merge 09cbf23), add an optional local directory
curated by the sysop. Approved members browse with arrows/hotkeys and open safe
SSH connection cards. Each destination still requires its own account/approval.
No auto-connect, DNS/network probe, credential/agent/key forwarding, automatic
host-key trust, public registry or federation. In-BBS member submissions follow
later; define deliberate out-of-band submissions/local review/removal now.

## Steps

- [x] Merge exact checked #130 head and preserve all 173 prior runtime/harness/
  build inputs plus whole recovery result in snapshot 22, `sources/pre27-directory`.
- [x] Bounded immutable directory pack, metadata/date/endpoint validation and
  shared text presenter/navigation; symbolic examples only.
- [x] Opt-in theme action and shared/local/worker routes, with current approval
  on every view/action. No schema, extra owner thread/listener or default change.
- [x] Handle full grids via classic ordered fallback; retain every action/hotkey.
- [x] Validate host/native PTYs, real member SSH/native client, revoke/resize,
  disabled/default and bad config; measure enabled directory separately.
- [x] Docs/README/decision, immutable results, cleanup and focused PR.

## Interface and bounds

A host-owned `boards.toml` pack, <=32 KiB / 32 listings, loads only at startup.
Use existing reviewed TOML dependencies in terminal presentation, separate from
Theme data. Shared owner retains one Arc catalog, each connection only a pointer
and cursor. File ownership/modes/no-symlink checks; no file reload per keypress.
Service `directory` and local/worker `--directory` are explicit; normal installer
leaves disabled. Offline `faction directory-check PACK` validates before enable.

Fields: slug/name/summary, host/port/login, reviewed/valid-until and optional public
SSH fingerprint hint. Dates describe editorial review, never live reachability.
Listings past valid-until are marked OLD; future review/unknown clock says DATE?.
Only printable ASCII/validated endpoint tokens reach connection cards. A display
hint does not authorize a host key or remote identity. No remote account copying.

Use O:Other BBSes unless the theme explicitly assigns contacts to another key;
fail clear on conflicts. Existing 5x2 grids can hold ten actions; an eleventh
optional action uses the ordered fallback, fitting 40x18. Default/theme previews
remain unchanged while the directory is off. Cards paginate long metadata and
never truncate a command into a different endpoint. Resize returns to main menu.

## Validation

Rust parser/navigation/action/compact-budget tests, local/worker PTYs and actual
shared SSH/native-client routes on a fresh owned 2-core/1-GiB NetBSD 11 ARM64 clone.
Keep source bytes/whole records frozen before later edits. All credentials/private
argv/logs stay outside Git. Saved playground stays off. Hardening remains later;
no physical SBC/new 250-member or protocol/federation-support claim.

## Outcomes

[Actual software evidence](../evidence/directory-netbsd/README.md): 188 Rust tests
per OS, 118 Python, eleven directory controls per OS, twelve real SSH/native
controls, existing operator/backup/serial/art regressions. All 165 selected native
inputs matched copied bytes; seventeen host harness/compile-compatibility inputs
are separately pinned. Four standalone manifests pass locked all-target checks.
The shared fixture needed prompt/pause expectation corrections before passing;
no production gate or runtime was relaxed for it.

Maximum catalog with one member: owner 5,312 KiB RSS / four LWPs; twelve samples,
minimum VM free 352.36 MiB. This is not new 250-member support or a memory delta.
No schema, external dependency, helper/listener or ordinary limit changed. Keep
member submission/moderation UI and trusted native visit handoff as follow-on;
#27 stays open for those product increments. Hardening remains later.

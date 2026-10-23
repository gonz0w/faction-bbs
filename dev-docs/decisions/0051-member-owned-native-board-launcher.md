# 0051: Member-owned native board launcher

Date: 2026-10-22. Status: selected; software validated. #27/#21/#14.

## Decision and alternatives

Add an optional private `--boards` catalog to the native terminal client, retaining
`--profile` as the original single-session path. Local ANSI pages, confirmation
and an explicit keyboard escape let a member visit independent configured boards
without restarting the client. One session/SSH child at a time keeps the lifecycle
and local working set simple; reconnects create fresh filters/nonces/link history.

A server-supplied visit frame/profile import would require a new trust/selection
contract and is deliberately deferred. A stack of suspended sessions needs output,
keepalive/grant and member expectations beyond this slice. An external shell/SSH
launch loses the integrated terminal feel and complicates return/key policy. An
optional browser remains independent no-JS/private-portal research. These are
tradeoffs, not claims to have implemented or benchmarked those alternatives.

## Authority and bounds

A <=16-KiB/sixteen-entry member-owned 0600 catalog contains only ASCII labels/IDs
and absolute local profile paths. Existing profile/key validation is reused and
repeated after selection. Only local keyboard input can open/choose/confirm;
server output has no catalog/selection API. Existing fixed accounts, verified host
keys, SSH options and independent local BBS approval remain. No automatic imports,
trust, credential/agent forwarding, file transfer or private-mail authority.

Ctrl-] B asks before disconnecting; double Ctrl-] preserves immediate door escape,
and a lone prefix forwards after one second. This applies only in catalog mode,
not the existing single-profile path. Native Files retains its foreground flow.
Old remote session/child drops before the next; graphics cleanup precedes picker.
No owner runtime, schema, server listener/helper or dependency change is needed.

## Validation and reversal

[Plan 0056](../plans/0056-native-board-launcher.md): host/native Rust/PTY checks,
actual two independently hosted boards with separate state and member host keys,
confirmation/host-key refusal, independent approval and Files return. Use two
owned marked software VMs because the production owner intentionally pins its
socket path. Do not relax that policy for a fixture. Preserve actual inputs and
old records; no new capacity or physical-hardware claim.

Use `--profile` to return to the old workflow; no server/database migration or
configuration change is required. Removing the private local catalog is enough.

[Actual software results](../evidence/board-launcher-netbsd/README.md) pass the
scoped host/native two-board controls. Server inputs/policy remain unchanged;
directory-to-profile import and member submissions remain follow-on work.

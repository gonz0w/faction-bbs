# Deliberate native travel settings

The native client can request a compact text visit and defer files above a chosen
payload ceiling. This is the first interactive increment of #159. Ordinary SSH
and the normal client experience remain available; offline packets/replies (#145)
and other transports are separate follow-on work.

In the local Boards picker, press **T** before connecting. O chooses ordinary
terminal presentation; C chooses compact text; B sets a decimal per-file ceiling
from 0 to 16777216 bytes. **S applies**, Q or Ctrl-C cancels. Your highlighted board
stays selected. Settings apply to all subsequent visits in this client run, with
no SSH child active in this panel. Nothing is saved as travel history or written
to your profiles, keys, catalog or vault. The next process starts with ordinary
terminal mode and the existing 16 MiB per-file maximum.

For direct profile visits or an initial picker preference:

```sh
faction-client --profile /private/board.toml --compact --file-budget 65536
faction-client --boards /private/boards.toml --file-budget 0
```

Zero defers every file transfer, including empty files. The ceiling is per file,
not a running session total or a bandwidth/rate limit. Use the connection summary
to review it before a picker visit. Change it in T after returning to Boards;
a direct-profile visit needs a new invocation. No background health probes,
automatic retries, automatic file downloads or destination changes are added.

## Compact and classic terminals

Compact asks OpenSSH for `TERM=faction-compact` through the existing private profile. Faction
recognizes this explicit native text hint and uses its existing plain/classic login, menus, article reader and text art
editions; content, approval and permissions are unchanged. Commands use Enter.
It disables optional board bitmap graphics for that visit. The service still emits
the existing bounded native Files advertisement for this explicit hint; ordinary
`TERM=dumb` sessions remain escape-free. The hint grants no file/member authority. Small ASCII identity
text may still be shown. It does not create a new trusted renderer, strip controls
into commands, or override terminal dimensions/read acknowledgments.

The local picker, Files and travel panel still use bounded ANSI text controls;
they fit the existing 40x18 minimum. This is a request for the remote board's
classic presentation, not a promise of zero escape bytes across all local UI or
other servers. `--compact` and `--kitty` together are rejected before startup.
If the picker was launched with `--kitty`, its explicit local capability probe
has already happened; applying compact prevents board graphics in the next visit.
This negotiation needs the updated Faction client and service; older boards may
not recognize the hint. No Tor/Reticulum support or fallback route is implied.

## Known payload sizes and deferral

Upload checks the open local source's size before asking for an area, reservation
or transfer grant. Download first inspects the current authorized object and shows
its filename, payload size and checksum, then checks the ceiling before asking
for a destination or requesting a download grant. Over-limit or malformed sizes
stop that operation and return to Files. They do not allocate an object, create a
local partial file or launch SFTP. Download metadata inspection still costs a small
board exchange. The existing explicit confirmation is required for an allowed file.

Sizes are **file payload bytes**. SSH/IP framing, encryption, metadata, prompts,
connection setup and retries cost additional bytes, so this is not an exact
on-wire cost estimate or prepaid link budget. Existing size/checksum/grant binding,
no-overwrite/partial cleanup, quotas, pending upload review, private-area grants
and one-file key-bound SFTP remain authoritative. Exact-ceiling files are allowed;
future size changes do not bypass the existing immutable object/grant checks.
No new resume, whole-session traffic counter, automatic retry or recovery claim.

The existing 32 KiB buffers/chunks, bounded native frames, progress cancellation
and network deadlines remain. Tiny/interrupted radio links and named synthetic
high-latency profiles still need separate #159/#145 validation; no physical-radio,
supported-throughput or capacity claim follows from this UI.

See [native client](native-client.md), [board launcher](board-launcher.md),
[ADR 0064](../dev-docs/decisions/0064-native-travel-preferences.md),
[plan 0069](../dev-docs/plans/0069-native-travel.md) and
[software evidence](../dev-docs/evidence/travel-netbsd/README.md).

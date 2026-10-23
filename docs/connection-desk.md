# Local connection desk

Status: native client preview for existing saved-board SSH profiles. No live
route probe, contact verification, Tor adapter or supported release is claimed.

In the [local Boards picker](board-launcher.md), highlight a board and press **I**.
The desk shows which terminal and Files endpoints that local profile names,
whether it parses safely, and whether the admission-key, public-key and host-pin
files pass the existing local file checks. **N/P** page, **R** reload, **Q** return;
commands use Enter. Ctrl-C also returns with your board selection retained.
Pages fit 40x18 terminals. No SSH connection is started by the desk.

A readable private-key file may be locked, invalid or unrelated to the configured
public key. The desk does not try to decrypt it or request a passphrase. Local
file readiness, live host identity, SSH admission and BBS login/approval are
separate stages. Your SSH agent/key setup and the destination sysop's approval
remain independent; reading a profile proves neither.

## Inspect stored host pins

**K** deliberately asks base `/usr/bin/ssh-keygen` to find stored terminal and
Files host pins, including hashed known-host entries. This is a local file lookup,
not `ssh`, a network probe or live host verification. Up to four distinct stored
SHA256 fingerprints per endpoint are displayed. Comments, raw keys and host fields
from helper output never render. CA/revoked entries are flagged as policy needing
OpenSSH's decision; they do not grant trust. Empty results and helper failures are
reported without guessing remote availability or member status.

Each lookup has a two-second deadline and 32-KiB output cap; only one helper runs
at a time and it is reaped on success, failure or cancellation. Ctrl-C cancels.
No helper stderr becomes terminal content. The checked bounded host file is passed
as stdin rather than reopening a member path inside the helper. A changed/missing
file can make the lookup unavailable; R refreshes file readiness. The real visit
re-reads its profile and retains strict OpenSSH host checking.

Verify expected keys out of band with the destination sysop. No host keys,
profiles, agent configuration or OS settings are changed. This client has no
verified-contact age, route/invitation credential model or Tor support to report;
those #158 follow-ons stay explicit. No silent fallback path is introduced.

## Share a redacted check

**S** previews the exact share summary locally; S again returns to details. It
omits labels, endpoints, ports, paths, keys, fingerprints, comments and timestamps.
No report is saved or sent. You can also run a plain-text check without a TTY:

```sh
faction-client --check-profile /home/member/.config/faction/board.toml
```

The command performs no pin-helper or network operation and prints only the
redacted readiness summary. Exit 0 means the local profile and referenced files
passed structural/readability checks; exit 1 means attention is needed. Neither
means a remote connection or membership will work. Use normal terminal copy if
you choose to share this summary. Malformed settings/errors never echo parser
content, local paths or raw file bytes in this report.

Your catalog/profiles/keys remain member-owned; there is no new disk history or
remote-controlled local action. Ordinary SSH/plain BBS access and the separate
local encrypted vault remain available. Hardening is deferred.

## Software validation

Twenty-one client Rust tests and fourteen real local PTY controls pass on macOS
and NetBSD 11 ARM64, including hashed pins, redaction, cancellation during the
helper, output bounds and small-terminal recovery. Launcher/travel/offline vault
regressions also pass. Full host workspace validation passes 361 Rust and 130
Python tests with strict clippy. The [evidence record](../dev-docs/evidence/connection-desk-netbsd/results.json)
pins exact source/binary identities and the unchanged server boundary. This is
software VM evidence; it makes no SBC/capacity/live-host-authentication claim.

# Connecting original classic game servers

Faction's optional classic doorway connects an approved member's terminal to a
sysop-selected Telnet game server. The original game runs on that separate
server or compatibility host. Faction does not rewrite it or launch a local DOS
executable. This connector has owned-fixture NetBSD SSH evidence; **no original
TradeWars/LORD title has yet been played through it**. See the
[reuse inventory](../dev-docs/research/0039-classic-door-reuse.md).

Both original game servers and isolated original DOS binaries are accepted reuse
paths. [#79](https://github.com/gonz0w/faction-bbs/issues/79) tracks actual TradeWars
server validation; [#78](https://github.com/gonz0w/faction-bbs/issues/78) tracks one
original DOS door. Existing Lua games remain original examples alongside them.

## Configure an optional server

Build the workspace normally. Stop member SSH/backend for an update and preserve
your configuration and private backup. Install the new non-setuid shared backend
with the normal owner/root permissions; no new service, UID or dependency is
needed for the connector. Existing database schema 3 is unchanged.

As root, add entries at the **end** of `/etc/faction/service.toml`, after all its
ordinary keys. This example is a placeholder for a server/tunnel you operate:

```toml
[[classic_games]]
slug = "tradewars"
title = "TradeWars on the private game host"
address = "127.0.0.1:2002"
```

Use literal IPv4 or bracketed IPv6 with a nonzero port. Hostnames/DNS, member-entered
addresses, wildcard/multicast/mapped-IPv6 destinations, extra settings, duplicate
slugs/endpoints and more than six entries are rejected. Keep the existing trusted
config ownership/mode; do not put credentials in it. The normal first installer
leaves the list absent, so the feature is disabled. Remove the entries and restart
to disable it again. Existing service preflight validates the new config.

The legacy **Telnet hop is unencrypted**. Use loopback with a separately managed
protected tunnel or an appropriately protected private network. SSH protects the
member-to-Faction leg; it does not encrypt the next hop. Game-server operations,
accounts, backups and licenses belong to that server's operator. No public service,
public game account or game purchase is installed/created for you.

Start the ordinary backend and member SSH explicitly:

```sh
/etc/rc.d/faction_broker onestart
/etc/rc.d/faction_sshd onestart
```

Without Lua, **[D]oors** opens the classic server list directly. With the updated
Lua example lobby installed, **[C] Classic game servers** opens it; **[L]** on
that list returns to Lua. Existing custom lobbies can add:

```lua
bbs.option("C", "Classic game servers", "classics")
-- Or show the host-owned destination confirmation for one configured slug:
bbs.option("T", "TradeWars", "server:tradewars")
```

Both targets are hidden when no classic destinations are configured. The Lua API
cannot create/change a destination, open a socket or bypass confirmation. Update
your edited Lua files deliberately; the first-install helpers are not upgraders.
The optional Lua installer places `doors_uid` before TOML array tables so either
extension can be installed first.

## Member experience

Use arrows/Enter or displayed hotkeys to select a server. Faction shows its name,
exact destination and legacy-link notice. Choose **[C] Connect** to open it; going
back/canceling opens no connection. The game may require its own login. Faction
never automatically sends your BBS handle, password, role, room data or session.
Use separate credentials. Game traffic and typed input are not logged by Faction.

**Ctrl-] returns to Faction immediately.** Ctrl-C/D and normal ASCII keys go to the
game; Backspace is sent as BS, and Enter becomes Telnet CRLF. Input is not echoed
locally; the server controls its own prompts/echo. Modern UTF-8 non-ASCII keyboard
input is not converted to CP437 yet. Disconnect/error, malformed excessive controls,
resize or 120 seconds without keyboard input closes the door. Local membership
revocation closes the BBS connection. The normal total session deadline remains.
Remote disconnect or local escape leaves rooms/chat usable. A changed terminal
size closes the game; re-enter with the new geometry. Plain terminals strip ANSI;
a title that needs full screen positioning may still require an ANSI terminal.

## Compatibility and boundary

The bridge handles streaming Telnet ECHO/SGA, terminal type ANSI/DUMB and bounded
window-size negotiation, CP437-to-UTF-8 display, classic 16-color SGR and bounded
cursor/clear/save/restore operations. Unknown options are refused. Clipboard/title
strings, device queries, alternate buffers and unsupported terminal commands
never pass through to the member terminal. RIP/sixel/graphics, arbitrary UTF-8
server streams, 256-color/truecolor, auto-login, rlogin/dropfiles and title-specific
terminal quirks are outside this first connector.

The game executes remotely; only bounded safe-Rust network/terminal parsing runs
in the shared backend. This expands its protocol surface and trusts that Rust/OS
boundary; it is not a separate parser-process or hostile-code sandbox. No local
classic process is launched, no backend process/thread limits are raised and Lua
still has no network/account/SQLite API. At most eight remote connections exist
in the backend, with a 4-KiB pending-write cap, 1-KiB reads per loop, 3-second connect,
5-second pending-write and 64-KiB-per-window/4-MiB-total input limits. Excess closes
only the door. The byte window resets after one second; this is a simple bounded
window, not a hard sliding-window bandwidth guarantee. Normal membership/admission
and two credential slots are unchanged. No new capacity claim is made.

Remote game characters, saved universes and scores stay on their own server.
They do not enter local BBS scoreboards or private database exports. This is an
explicit member game connection, not automatic federation or account replication.
The separate experimental network/isolation profiles have not been adopted/tested
for this connector; do not loosen their rules automatically to make a game work.

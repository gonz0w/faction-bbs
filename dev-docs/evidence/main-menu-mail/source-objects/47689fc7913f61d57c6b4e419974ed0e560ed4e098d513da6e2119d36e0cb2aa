# Encrypted home mail from the main menu

A sysop can add an optional **Encrypted home mail** action to a custom main menu.
The shared BBS offers a handoff to your native client; your own device opens and
decrypts the inbox. You do not need to enter Files, and the board does not need
file areas or its transfer service enabled.

## Member flow

Configure the optional native client's [home-mail paths](picker-home-mail.md) and
[encrypted home mail](online-mail.md). Select the sysop's displayed mail hotkey or
use arrows and Enter. In the native client's local view, choose **Open** or
**Back**. Open asks for a fresh local confirmation and hidden checkpoint unlock.
An offer from the board alone cannot unlock a store or contact your home.

Use the existing local inbox/history, compose and explicit acknowledgement actions.
Opening, listing or viewing mail never silently marks it read. **Lock and exit**
locks the store before returning to the same BBS main menu. Cancel and a home
outage also return when the board connection remains valid. Every new mail entry
requires another deliberate confirmation and unlock.

During the private view the client does not render board output or send your mail
keyboard input to the board. Bodies, senders, subjects, unread counts, home account
credentials and mail keys remain on your device. The board can observe its menu
selection and connection timing; this is not traffic-analysis protection. Broad
Who presence remains generic Menu, without mail activity or private details.

The client clears the viewport and drops queued input at each boundary; it does
not erase terminal scrollback. Existing buffers/backpressure and the server's
120-second non-menu prompt timeout still apply. A timed-out, disconnected or
revoked board cannot be resumed. The client reports the lost visit rather than
redialing or retrying a request automatically. Mail itself remains independent of
visited-board approval; the handoff grants no mail or BBS identity.

## Classic and unconfigured clients

Ordinary SSH and older native clients see a readable explanation and an Enter/back
return. A new native client without the optional mail build or local home paths
explains that locally and returns safely. Standalone/local and inherited terminal
adapters provide an explanatory main-menu return; they do not perform the shared
native handoff or decrypt private mail on the server.

Existing [parked Files mail](local-unread-mail.md) and Files/SFTP controls remain
available through their current workflow. Notices stay off by default, local,
stale/unknown and opt-in. This main-menu action adds no background refresh, cached
unlock, automatic acknowledgement, private live chat or encrypted attachments.
It does not intercept a room draft, game, chat, transfer or terminal probe.

## Sysop configuration

Add an entry to a valid owner-managed [theme pack](theme-packs.md), using a key not
already used by that pack:

```toml
[[menu]]
action = "home_mail"
key = "M"
label = "Encrypted home mail"
description = "Open mail on your device"
```

This is one entry to add to a complete valid menu, not a standalone theme file.
Existing required actions, entry counts, layout, ownership and validation rules
still apply. Default packs do not add or reserve a mail key. Seasonal colors/art
continue to use the same base command map. The label does not grant approval,
enable file areas or configure a member's home account.

The handoff has a bounded budget of 64 consumed nonces per board connection.
Exhaustion leaves ordinary BBS use available and explains that a new mail handoff
is unavailable on that visit. Invalid/replayed/out-of-state controls fail closed.
The [wire contract](../dev-docs/main-menu-mail.md) documents its exact scope.

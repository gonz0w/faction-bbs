# Visit boards with the native launcher

The optional native launcher keeps a private list of your own connection profiles.
Choose a board, connect deliberately and log in using that board's local account.
Disconnecting returns to the launcher so you can visit another community without
restarting the client. This is a Unix ANSI-terminal preview, including NetBSD.

## Make your local catalog

First configure and verify each [native connection profile](native-client.md#connect).
Each profile still contains the member and Files endpoints, an admission key and
pre-enrolled host keys. The launcher does not discover or enroll these for you.
File-service fields remain required by the current profile format even if the
board leaves Files disabled. Ask the destination sysop for the correct settings;
all SSH accounts, key validation and private-network requirements stay unchanged.

Create a file owned by your device's login user with mode `0600`:

```toml
version = 1

[[boards]]
id = "night-radio"
label = "Night Radio"
profile = "/home/member/.config/faction/night-radio.toml"

[[boards]]
id = "the-vault"
label = "The Vault"
profile = "/home/member/.config/faction/the-vault.toml"
```

Replace the symbolic paths with your own absolute paths. Run:

```sh
faction-client --boards /home/member/.config/faction/boards.toml
```

The catalog is at most 16 KiB and contains one to sixteen unique entries. IDs use
lower-case ASCII letters/digits/hyphens up to 32 bytes; labels are printable ASCII
up to 26 bytes. Profile paths use the existing absolute-path rules, up to 512
bytes, without parent traversal, control or SSH expansion characters. Duplicate
IDs, labels or profile paths and unknown fields reject. Symlinks, hard links,
nonregular files and catalogs accessible by another Unix user reject. Keep parent
folders private too. Profiles/keys keep their existing ownership and mode rules.

Only the catalog's labels/paths load at startup. The selected profile is read and
validated again on every visit; a bad or removed profile returns to the launcher.
No profile or host key is created/edited, and no other board is contacted for you.

## Navigation and connection

Arrows/Tab select a board; Enter or A–D chooses from the current four-entry page.
N/P or Left/Right changes pages. Q or Ctrl-C quits from the picker. Four-entry
pages fit a 40x18 terminal; the full client still requires ANSI and a real TTY.
Ordinary BBS SSH/plain fallback is unchanged.

The local connection screen shows the chosen label, SSH and Files endpoints.
**Y then Enter** connects with that profile; every other answer cancels. Unknown
or changed host keys fail before BBS login and return to the local launcher.
Verifying a key out of band remains your responsibility; a structural profile
check is not proof of a remote host's identity. The client retains strict host-key
checking and its existing restricted OpenSSH options.

After a normal logout or failed connection, Enter returns to the picker. During
an ordinary terminal session, press **Ctrl-] then B** within one second to ask for
disconnection. Confirm Y to discard drafts/leave the current board; other answers
or Ctrl-C stay connected. While this local prompt is open the board's normal
session/output timeouts still apply. Return from the native Files menu with B
before using the launcher shortcut; it does not interrupt an active transfer.

Press **Ctrl-] twice** to send one Ctrl-] immediately, including the classic game
door's local return key. A lone Ctrl-] passes through after one second; an unknown
following key passes through with it. The existing `--profile FILE` mode keeps
its old byte-transparent keyboard behavior and exits after disconnecting.

There is one active SSH child/session. Leaving closes/reaps it before another
connection starts; the old session is not suspended or resumable. Every visit
creates fresh terminal filters, Files negotiation nonce/sequence and recent-file
history. Optional `--kitty` still requires local capability approval; trusted image
cleanup runs between visits. No board can open the launcher or choose/confirm a
profile through text, file frames or graphics. The local keyboard controls it.

## Other BBSes and independent communities

Use [Other BBSes](other-boards.md) on a board to discover a destination. Obtain its
settings and verify keys independently, create your own local profile/catalog
entry, then choose it in the local launcher. Directory cards are editorial hints;
they never become trusted device configuration or automatic connection requests.
The directory is local to the sysop; your catalog is local to your device.

Each destination has its own admission and BBS registration/approval. Approval on
one board gives no access on another. Keys are used locally for the selected SSH
connection; no agent is forwarded and no other board's account, password, device
keys, Files grants or recent links are handed over. Select separate admission
keys in profiles when desired. No credentials are remembered for BBS login.

This increment adds no roaming identity, encrypted mail/XMPP, remote profile import,
public registry, browser launch or automatic federation. Those remain separately
scoped work; member-held mail keys are not part of a connection profile. The BBS
owner, database/schema, server helpers/listeners and ordinary limits are unchanged.

[Plan 0056](../dev-docs/plans/0056-native-board-launcher.md) records the exact
software validation and remaining work. Hardening remains a later phase.

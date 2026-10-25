# Encrypted home mail from the board picker

A `network-mail` build can open your existing native home-mail console between
board visits. **E** is home mail; **M** remains the separate v1 offline vault.
Mail keys, plaintext and credentials stay on your device. A visited board cannot
open this screen or choose its files.

First [build and configure manual home mail](online-mail.md), initialize the
checkpoint deliberately, and verify your peer. Add this optional entry at the
root of your private [board catalog](board-launcher.md), before `[[boards]]`:

```toml
version = 1
home_mail = { store = "/home/member/.faction-mail-v2", config = "/home/member/.config/faction/home.toml" }

[[boards]]
id = "night-radio"
label = "Night Radio"
profile = "/home/member/.config/faction/night-radio.toml"
```

Replace the paths with your own absolute paths. Keep the catalog owned by your
OS login user, mode `0600`, and its directory private. Both `store` and `config`
are required; extra fields, relative paths, traversal, controls, quotes,
backslashes, `%` and `$` in paths reject. Home configuration, CA and checkpoint ownership/no-link rules
remain those of the existing mail console. The catalog contains paths, not mail
passwords, private keys or recovery secrets.

Run:

```sh
faction-client --boards /home/member/.config/faction/boards.toml
```

Choose **E**, then enter your local checkpoint passphrase at the hidden prompt.
The picker does not open a store, load home credentials or connect merely by
showing the menu or changing board selection. Builds without `network-mail`
refuse this configured entry; they do not create or upgrade a store. A catalog
without `home_mail` keeps its existing behavior.

## Read, send and return

The console is the same [manual home-mail interface](online-mail.md): **H** reads
authenticated history, **C** composes and queues, **O** inspects persisted pending
ciphertext, and **N** makes one deliberate home cycle. For an archive-enabled
store, N sends only and **A** fetches one bounded archive page. **G** remains an
explicit local format upgrade; selecting E never performs it for you. See the
[archive guide](archive-mail.md) for duplicate, full-store and stale-position limits.

Use **L** to lock and return to the same board selection. Cancellation also locks.
A stopped local operation shows a generic notice and, when requested, waits for
Enter before returning. Missing, busy, unsafe or wrong-secret state is not
initialized, replaced or recovered automatically. Unlock is required on re-entry.

Append `--unicode` to `--boards` in a network-mail build to use the existing
Unicode history/composer rendering. This setting applies to the local mail view;
it does not grant a remote board extra terminal authority. Classic rendering and
the compact 40x18 picker remain available.

A home outage leaves local history available. A connection-refused outage before any
entry is accepted leaves the saved cursor and pending ciphertext unchanged. If
a later archive entry fails after earlier entries were accepted, that committed
history/cursor progress remains durable; archive outcomes do not retire pending
ciphertext. Lock the mail view and visit a board normally;
its independent login, approval and restricted file service still apply. BBS SFTP
files are sysop-readable community files, separate from encrypted mail attachments.

This is an explicit action before/between visits. It adds no mail overlay during
an SSH session, background polling, unread badge, shared login, roaming or
attachment support. Ordinary SSH does not decrypt member-only mail.

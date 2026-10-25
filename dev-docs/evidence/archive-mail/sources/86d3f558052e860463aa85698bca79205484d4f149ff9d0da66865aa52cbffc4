# Catch up retained encrypted mail

The optional local mail console can fetch one page of ciphertext retained by your
own home. You can receive while the peer is offline, provided the homes retained
and forwarded the messages. Keys, decryption and accepted history stay on your
device. A home may omit, expire or reorder archive entries; an empty or completed
page does not prove that you have every message.

Use the [online-mail setup](online-mail.md) to build with `--features network-mail`,
initialize your checkpoint, configure independently verified TLS details, and
verify the peer's complete Ed25519 identity and device ID. Your home must support
the selected MAM2 archive profile and retain these encrypted messages. Ask its
operator about retention. The client requests storage on outgoing messages, but
that request does not guarantee storage or delivery.

## Enable archive checkpoints deliberately

Open the trusted-local console with absolute paths:

```sh
./target/release/faction-client --mail-online \
  "$HOME/.faction-mail-v2" "$HOME/.faction-home/home.toml"
```

Append `--unicode` if desired. Enter your local checkpoint passphrase. The existing
file ownership, private configuration and hidden home-password rules still apply.
Keep local unlock secrets out of remote BBS terminals and home configuration.

1. Use **B** to make an encrypted history-only backup in a new absolute directory.
   Keep it privately. It preserves readable history, not an active session to resume.
2. Use **G**, read the format warning, and type `upgrade` to enable archive
   checkpoints. This is a local operation and does not contact your home.
3. Confirm that the menu reports archive checkpoints enabled. Repeating **G** keeps
   the existing archive position; it does not start over.

The upgrade preserves existing keys, verified peer, history, recovery secret and
pending ciphertext. It changes the encrypted inner checkpoint format. Older
clients cannot open the upgraded store. There is no automatic upgrade, downgrade
or cursor reset. Enabling requires an active, unexpired verified peer and room in
both history and outbox; recovered/history-only or revoked state cannot enable it.

## Send and receive separately

After the upgrade, **U** still publishes public device material and **K** still
prepares the chosen first sender once. Only one side should prepare initially;
the recipient starts by accepting the first authenticated message. See the
[first-message instructions](online-mail.md#exchange-your-first-message).

Use **C** to review and queue a message locally, then **N** to send pending
ciphertext. In archive mode these actions do not announce online availability,
and **N** finishes after sending. It does not receive live mail.

Use **A** to receive one archive page. Enter the home account password at the
hidden prompt. The client connects once to your own home and requests at most
four entries for the already verified peer, starting after the saved position.
It does not publish keys or retry pending sends during catch-up. Use **H** to read
accepted history and **A** again when you deliberately want another page.

The page count includes duplicates and your own retained sent copies, so four
entries need not mean four new received messages. A recognized sent copy advances
the position without decrypting or creating another history record. Neither that
copy nor the server's completion flag is a delivery or read receipt.

**O** continues to show the same pending operation IDs. Sending, catch-up and
accepted duplicates do not retire them. **N** retries the same persisted
ciphertext. Use **T** only for your explicit decision to stop those retries.

## Interrupted or refused catch-up

Each action makes one connection attempt within the existing cooperative
ten-second deadline. Catch-up stops after two seconds without the next stanza;
it does not poll or reconnect in the background. Ctrl-C/D cancels and locks.

A page's wrapper and finishing response must validate before any entry is saved.
Then each accepted entry and its archive position are saved together. If a later
entry fails authentication, earlier accepted entries can already be durable.
Inspect history and retry explicitly. After ambiguous local I/O, lock and reopen
the canonical store. Do not remove a live lock or substitute a temporary file.

If the home no longer recognizes the saved position, the client preserves it and
retained history. Ask the home operator about archive retention. Automatic reset,
skip and complete-history reconstruction are unavailable.

The existing limits remain: one verified peer, 64 history records, eight pending
operations and a 30-day session lifetime. A full history or outbox stops catch-up,
even for duplicates. There is no automatic history eviction. Recovered backups
keep their archive position inert. Rotating local secrets does not reactivate
one. A deliberate fresh device starts a new archive scope with no saved position,
requires pending retirement and renewed peer verification, and keeps old history.
See [checkpoint recovery](checkpoint-console.md#recovery-and-failures).

This feature adds manual retrieval from a selected home archive. It does not add
background notifications, roaming, multi-device synchronization, attachments,
a guarantee of complete history, or private-mail access through a BBS server.
Those broader work areas remain under [#90](https://github.com/gonz0w/faction-bbs/issues/90),
[#99](https://github.com/gonz0w/faction-bbs/issues/99),
[#91](https://github.com/gonz0w/faction-bbs/issues/91) and
[#88](https://github.com/gonz0w/faction-bbs/issues/88).
The [technical contract](../dev-docs/native-mail-archive.md) defines the supported
archive profile and transaction boundary.

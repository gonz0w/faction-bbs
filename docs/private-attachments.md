# Private attachments in native home mail

The default-off `network-mail` native console has a **M: Private attachments**
desk for explicitly queueing, transferring and saving private files. Use it in a
trusted local terminal after setting up [manual home mail](online-mail.md) and
independently verifying your peer's full identity and device. Ordinary SSH does
not decrypt this mail. Attachment-capable storage must be available at the homes;
this preview does not deploy a provider or establish device attestation.

The optional [private provider operator](provider-operator.md) can serve preapproved same-home policy on loopback. It is a software preview and requires independently supplied credentials; it does not enroll devices or recover keys.

## Optional device-provider transport

Use [offline preparation](provider-preparation.md) to create the existing private
profile from an unlocked device and separately export a public card for a trusted
operator. These actions do not establish approval or remote readiness.

The explicit `device-provider` client feature adds an optional local profile for
attachment transfer. Select it with `--device-provider PRIVATE_PROFILE` alongside
`--mail-online` or `--boards`; without selection the existing IQ route remains.
Default, `native-mail` and `network-mail` builds exclude the provider. Both peers
must belong to the configured local home; the provider connection is loopback only.
The profile is local trust configuration, never supplied by a board, message or
attachment locator. Its expected account, device and full identity must match the
current unlocked vault. Transport certificates and keys are independent of the
mail identity and BBS login. TLS possession is not OMEMO identity attestation.

Keep the selected private root and its credential directories owned by you with
mode `0700`. Configuration, CA, certificate and key must be bounded, owned regular
files with mode `0600` and one hard link; path traversal follows no symlinks.
The profile is a TOML file of at most 8192 bytes. Each nonempty PEM file is at
most 65536 bytes and must be beneath the profile’s parent directory. For example,
replace every illustrative value below with independently verified local values:

```toml
version = 1
home = "home.example"
endpoint = "127.0.0.1:12345"
name = "home.example"
pin = "0000000000000000000000000000000000000000000000000000000000000000"
account = "alice@home.example"
device = 101
identity = "0000000000000000000000000000000000000000000000000000000000000000"
ca = "/home/alice/.faction-provider/ca.pem"
certificate = "/home/alice/.faction-provider/device.pem"
key = "/home/alice/.faction-provider/device-key.pem"
```

The endpoint is a numeric loopback address with a nonzero port; `name` is the
verified TLS server name and `pin` is the lowercase SHA-256 hex digest of its
complete DER leaf certificate. `identity` is the complete vault public Ed25519
identity in lowercase hex, and `device` is its integer device ID. The zero values
are placeholders, not trusted credentials. Unknown fields, including passwords,
are refused. Build explicitly with `--features device-provider`, then start either
`--mail-online STORE CONFIG --device-provider PRIVATE_PROFILE` or
`--boards CATALOG --device-provider PRIVATE_PROFILE`; `--unicode` is optional.

The profile is checked at attachment actions, before spool or connection, rather
than during ordinary mail. A selected provider failure stops the attachment action
without falling back to the IQ route. Ordinary mail, BBS and files remain independent.
Device fetch needs no home password. Upload asks for the home password only for
the later ordinary encrypted-mail release after authenticated storage confirmation.
If a response is lost, storage may already have changed: reopen and retry the exact
persisted operation; a lost reply is neither rollback nor permission to release mail.


## Enable and retain

Choose the desk's explicit attachment opt-in. It upgrades the encrypted checkpoint
to `FACTCP05` without connecting and preserves trust, history, pending messages,
and existing read and archive state. Older clients that do not support this format
cannot open it. Back up before upgrading and keep a compatible client available.

Ciphertext is stored lazily in a separate `attachments` directory under the trusted
vault root. Unlocking, listing and text mail do not create it. The spool permits
eight objects, retained at most seven days, and each attachment holds at most
1 MiB of plaintext. Expiry is not extended by retry. The spool is separate from
the `mail.fkv` checkpoint and needs its own private backup and retention decisions.
A failure between spooling and checkpoint publication can leave an orphan. There
is no inventory or sweep interface in this preview; do not assume a mail backup
contains the attachment bytes. Backups can retain copies beyond normal expiry.

## Compose and upload

Select an absolute local source path. It must name a current-user-owned, regular,
owner-readable file with one hard link and no group or other write permission;
`0644` is allowed. Symlink components, dot or traversal components, control
characters and relative paths are refused. The client reads a bounded snapshot;
it does not expand shell syntax or execute the file.

Review the complete recipient identity and device, selected name, media type and
size, and the subject and body before the fresh queue confirmation. Cancellation
queues nothing. Queueing persists the encrypted descriptor and exact message wire
locally; it does not upload. Select the durable operation ID for a separate
foreground upload. Retries use that same persisted operation and ciphertext, so
the original source can be removed after successful queueing. A network write is
not proof of delivery or reading.

## Fetch, verify and save

Select an authenticated received history record by its displayed original
number. The client binds that selection to its exact received frame. The displayed name, media type and size are private metadata, not authority
to connect or choose a destination. Unsupported, stale, sent and text-only records
cannot be fetched through this desk. Listing and inspecting metadata make no network
connection and do not open the spool.

Fetch requires the current healthy verified peer and identity to be eligible for
network transfer. The selected device-provider route rejects a foreign-home sender.
For the existing IQ route, when the authenticated sender belongs to a foreign home, give
fresh explicit consent to that home. Its identity comes from the authenticated
sender, never from an attachment locator. All traffic and the hidden password use
your own configured pinned home; the sender can be offline. Refusal connects to
neither home.

After fetching, explicitly verify the locally stored ciphertext against the exact
record, then choose a new absolute destination and confirm saving. The final parent
must already be owned by you with mode `0700`; the new file is `0600` and an
existing destination is never overwritten. Nothing automatically opens, executes
or extracts the attachment. A `BeforePublication` failure means publication did
not occur or a destination collision was definite. For `PublicationUncertain`,
inspect the destination before retrying: it may already contain the complete file.

Local verify and save remain available while the unexpired spool binding belongs
to the current local principal, even when network transfer is no longer eligible,
subject to the SDK's checks. They do not promise recovery on a new device. The
older manual ciphertext APIs have broader historical consumption rules; see the
[technical contract](../dev-docs/attachment-mail.md).

## Foreground transfers and limits

One explicitly requested transfer may make up to 65 SDK calls over a cooperative
60-second total budget. The existing IQ route uses one hidden password for your
own home until the action returns. Each SDK call retains its ten-second bound and
at most four 4096-byte chunks. The provider allows eight attempted requests per
session, five seconds for admission including TCP/TLS, a ten-second accepted cycle
and five seconds for drain. Upload includes the later mail release in its call bound.
The action continues only on successful forward progress and stops at the first
error. It reports actual partial or completed bytes; there is no background retry
or automatic reconnect. Ctrl-C or Ctrl-D cancels and locks. Retry deliberately
after checking the reported state. These limits do not change normal board budgets.

The desk is a software preview. A [NetBSD 11 ARM64 software qualification](../dev-docs/evidence/netbsd-attachments/README.md)
exercised the production console and a 1 MiB transfer with both test homes, the
client and handler on loopback inside one guest. This does not establish deployed
provider behavior, device attestation, independent client interoperability,
same-SSH tracing, physical hardware, a security audit or capacity. The separate
[device-provider integration proof](../dev-docs/evidence/native-provider-client/README.md)
exercises actual same-home mTLS transfers, ordinary encrypted-mail receipt and
private save in classic and Unicode modes on host and fresh NetBSD. It retains
the original failure observations and unchanged transfer limits. Text mail
and classic board access remain separate and available.

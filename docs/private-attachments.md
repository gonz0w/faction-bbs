# Private attachments in native home mail

The default-off `network-mail` native console has a **M: Private attachments**
desk for explicitly queueing, transferring and saving private files. Use it in a
trusted local terminal after setting up [manual home mail](online-mail.md) and
independently verifying your peer's full identity and device. Ordinary SSH does
not decrypt this mail. Attachment-capable storage must be available at the homes;
this preview does not deploy a provider or establish device attestation.

The [delivery research](../dev-docs/research/0061-private-attachment-delivery-contract.md)
rejects the unchanged modern-client candidate. Real provider deployment and
independent client interoperability remain unqualified.

The [corrected headless reference research](../dev-docs/research/0064-corrected-headless-source-profile.md)
remains unqualified. The subsequent [CMake qualification](../dev-docs/research/0065-cmake-bootstrap-qualification.md)
rejects the selected unmodified native build profile because configuration would
generate JavaScript/QtScript; complete tool and license closure remains unknown.

The optional [private provider operator](provider-operator.md) serves separately approved device and service policies on loopback. For invited homes, each operator supplies fixed peer endpoints and independently obtained credentials, then uses the composed init, apply and serve commands. Stop the owner before policy changes; inspect retained state after an uncertain outcome and use a freshly authorized exact retry. Reset clears active approvals, so recovery requires a newly approved complete policy. The operator does not enroll devices or recover keys.

The optional own-home route supports transfers between independently invited
homes. Each device contacts only its own configured home; that home uses separately
preapproved policy to reach the other home. A compromised trusted home could
impersonate its device claims and obtain ciphertext, but does not receive the
attachment keys or plaintext. Vouching does not enroll encryption devices or replace
verified identities. See the [trust contract](../dev-docs/provider-integration.md#invited-home-attachment-trust).

## Optional device-provider transport

Use [offline preparation](provider-preparation.md) to create the existing private
profile from an unlocked device and separately export a public card for a trusted
operator. These actions do not establish approval or remote readiness. The
[manual enrollment guide](provider-enrollment.md) describes the separate approval
and dedicated SSH-forward workflow.

The explicit `device-provider` client feature adds an optional local profile for
attachment transfer. Select it with `--device-provider PRIVATE_PROFILE` alongside
`--mail-online` or `--boards`; without selection the existing IQ route remains.
Default, `native-mail` and `network-mail` builds exclude the provider. A version 1 profile requires both peers to belong to the configured local home;
the provider connection is loopback only.
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

For the separately enabled, default-off `own-home-provider` build feature, select a private
version 2 profile using the same `--device-provider` option. Build with
`--features own-home-provider`. It has the same fields
and requirements as version 1 above, with `version = 2` and the additional field
`route = "own-home"`. The existing preparation and public-card commands produce
version 1 material; they do not generate or enroll version 2 profiles. Arrange the
credentials and the homes' separately approved policies with their operators; see
[provider operation](provider-operator.md). Selecting a profile grants no approval.

The unlocked vault's full current identity and its verified peer snapshot remain
required. For two local principals the existing binary session is used; a local
sender and foreign recipient use own-home publication, while a local recipient and
foreign sender use own-home retrieval. A mismatched local full principal, two foreign
principals or an unsupported action is refused before a socket opens. The selected
route stays fixed for the action: errors or Busy never trigger another endpoint or
IQ fallback. All provider connections use the same configured home endpoint.

The profile is checked at attachment actions, before spool or connection, rather
than during ordinary mail. A selected provider failure stops the attachment action
without falling back to the IQ route. Ordinary mail, BBS and files remain independent.
Device fetch needs no home password. Upload asks for the home password only for
the later ordinary encrypted-mail release after authenticated storage confirmation.
If Busy, an error or a lost response stops an action, inspect its retained exact
state before a deliberate, freshly authorized retry of that same operation. Storage
may already have changed; a lost reply is neither rollback nor permission to release
mail. A retry does not renew the budget or switch routes, and completion is not
automatic.


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
the original source need not remain after the encrypted spool is confirmed. A network write is
not proof of delivery or reading.

## Fetch, verify and save

Select an authenticated received history record by its displayed original
number. The client binds that selection to its exact received frame. The displayed name, media type and size are private metadata, not authority
to connect or choose a destination. Unsupported, stale, sent and text-only records
cannot be fetched through this desk. Listing and inspecting metadata make no network
connection and do not open the spool.

Fetch requires the current healthy verified peer and identity to be eligible for
network transfer. A version 1 device-provider profile rejects a foreign-home sender. With the
explicit own-home profile, the sender home is displayed and its separately
preapproved home policy applies; the normal transfer confirmation remains. This
route neither asks the legacy IQ invitation question nor gives the client authority
to invite a home. For the existing IQ route, when the authenticated sender belongs to a foreign home, give
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
The existing worker permits eight SDK requests per ten-second cycle. Each own-home
carrier operation stays within that worker's original deadline; it cannot renew or
extend it. Its eight transport attempts are separate from the worker's SDK request
limit. Submit, close and paced collection share that attempt and deadline bound;
each fresh collection waits inside the budget, with at most six paced collects. Only a fresh authenticated Store progress response with the exact binding,
complete offset and committed status releases the queued mail. Receipts, Busy, EOF and timeouts
never release it; retry the exact persisted operation after checking the state.

The action continues only on successful forward progress and stops at the first
error. It reports actual partial or completed bytes; there is no background retry
or automatic reconnect. Ctrl-C or Ctrl-D cancels and locks. Retry deliberately
after checking the reported state. These limits do not change normal board budgets.

This preview requires updated provider software at both homes for the waiting
collection protocol. An older resource provider rejects the request, and the
transfer stops without an automatic retry. The brief resource wait does not extend
the transfer deadline. The newer device own-home waiting path additionally requires
matching updated operator and device software; an older peer's refusal fails closed.
Controlled host checks cover that path, but its native qualification and full
maximum-file CI remain pending. See [#358](https://github.com/gonz0w/faction-bbs/issues/358).
The earlier [extension's host observations](../dev-docs/evidence/bounded-resource-collection/README.md)
include both console modes. The older native qualifications below record earlier
collection behavior; they are not fresh NetBSD tests of this extension.

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

Host and fresh NetBSD software exercises completed maximum 1 MiB transfers in
both classic and Unicode modes. The NetBSD exercise used the maintained client,
1 GiB RAM and two CPUs, and the original 60-second budget: each 1,048,592-byte
ciphertext transfer took 65 upload and 65 fetch calls. Its compiler needed a
temporary 2 GiB setting; 1 GiB was restored before qualification. The two homes
had separate runtime, configuration, keys, stores and approved policies, with one
opaque test XMPP delivery/archive queue. This does not establish deployed S2S,
independent client interoperability or provider adoption. See the
[software evidence](../dev-docs/evidence/own-home-attachments/README.md).

A host interruption exercise recovered a persisted upload, but a later fetch
stopped and remains unexplained. A separate fresh fetch resumed partial state and
completed; that does not establish automatic retry or a fix. Inspect retained state
and retry deliberately with fresh authorization when needed. Ordinary text mail,
BBS access and SFTP also passed separate software outage exercises. See the
[software evidence](../dev-docs/evidence/own-home-attachments/README.md) for scope.

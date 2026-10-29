# Offline device-provider preparation

These separate local actions prepare an existing device for the optional same-home
attachment provider. They do not enroll or approve a device, contact a provider,
create or recover an identity, or change the vault checkpoint or attachment spool.
Run the member commands on your own computer, outside the BBS SSH stream.
Ordinary board access and mail remain independent. Use a trusted local terminal and
an independently trusted operator; see [private attachments](private-attachments.md)
and the [operator guide](provider-operator.md).

## Prepare the private profile

After the selected-source, license and no-JavaScript preflight, build explicitly
with Rust 1.96 and the existing OpenSSL 3 development installation:

```sh
RUSTUP_TOOLCHAIN=1.96.0 cargo +1.96.0 build --locked --offline -p faction-client --features device-provider --bin faction-client
```

Use the resulting `faction-client` and absolute paths for all command arguments.
An existing private vault and `mail.lock` are
required. Unlock uses the normal hidden local password prompt. The current device
must be usable: a fresh device need not have a peer or started session, but expired,
history-only or revoked state is refused. No missing vault lock is created.

Independently obtain the server TLS name and leaf pin and preload a CA, client
certificate and matching private key. Under an owned `0700` profile parent, place
these owned, single-link `0600` regular files without symlinks. Create a private
`0600` template with exactly these eight fields, replacing all illustrative values:

```toml
version = 1
home = "home.example"
endpoint = "127.0.0.1:12345"
name = "home.example"
pin = "<64 lowercase hex characters>"
ca = "/home/alice/.faction-provider/ca.pem"
certificate = "/home/alice/.faction-provider/device.pem"
key = "/home/alice/.faction-provider/device-key.pem"
```

The endpoint must be numeric loopback with a nonzero port. The pin is the server
leaf's SHA-256 DER digest; verify it and the server name independently. The template
must not contain account, device or identity: those come from the unlocked vault.
The template and output profile are each limited to 8192 bytes; each PEM is limited
to 65536 bytes. Credentials must be beneath the output profile's parent. Local
checks parse the bounded credentials and check the certificate/key pair; they do
not add client-chain, purpose or current-certificate-date policy. Remote TLS and
current authority decide transport validity. Credentials are independently supplied;
the product is not a certificate issuer.

```sh
faction-client --device-provider-prepare VAULT_DIR PRIVATE_TEMPLATE PRIVATE_PROFILE_OUTPUT
```

`--unicode` may be added. The new profile uses the existing eleven-field version 1
format and is checked by the maintained loader. Do not add `--device-provider` to
either setup action. Success does not establish remote readiness or approval.
Keep the private profile and key private; never send them to the operator.

## Export the separate card

```sh
faction-client --device-provider-card VAULT_DIR PRIVATE_PROFILE PRIVATE_CARD_OUTPUT
```

This action unlocks the vault again and checks the profile against it. `--unicode`
is optional. The card is a flat version 1 TOML file containing exactly `version`,
`account`, `device`, `identity` and `certificate_sha256`. It records the full public
Ed25519 identity and the selected **client** transport leaf's SHA-256 DER digest,
not the server pin. It contains no endpoint, pin, local paths, password, private key,
message, attachment name or locator. The certificate must be one unambiguous public
certificate in standard OpenSSL PEM form; the card is at most 8192 bytes.

The card is self-asserted local data, not proof of account ownership, OMEMO or
transport-key possession, enrollment, membership or approval, and is not a network
protocol. Its account, device and identity are deliberately shared only with an
independently trusted operator. Despite its public content the file is private
`0600` by default. Supply the operator the card and the separately selected public
client certificate through your trusted channel, never the member profile or key.

Each output requires a new path in an owned `0700` directory, with no symlink
components; publication is atomic and never overwrites an existing file. The two
outputs are separate operations, not a transaction. A failure after publication may
leave a valid visible output: inspect it and use a fresh path if retrying; do not
assume automatic rollback.

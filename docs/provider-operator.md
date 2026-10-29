# Private attachment provider operator

This optional standalone executable prepares and serves a same-home attachment
provider on numeric loopback. It is software preview, not provider deployment or
production enrollment. It needs a trusted operator's preapproved public policy and
independently provisioned TLS material; it does not issue certificates, enroll
members, grant BBS membership or recover device keys. Ordinary BBS access, files
and encrypted mail remain independent. See the [attachment guide](private-attachments.md)
and [durable policy contract](../dev-docs/provider-policy.md).

## Build and prepare

After the repository's selected-source/license/no-JS preflight and with Rust 1.96
and an existing OpenSSL 3 development installation, explicitly build the operator:

```sh
RUSTUP_TOOLCHAIN=1.96.0 cargo +1.96.0 build --manifest-path crates/faction-provider/Cargo.toml --locked --offline --features operator --bin faction-provider
```

Use the resulting `faction-provider` executable below. Prepare separate, existing,
empty policy and ciphertext directories, owned by the operator and mode `0700`.
Place the configuration, proposal and independent PEM files in an operator-owned
`0700` configuration directory. Files must be regular, single-link, owned by that
user and mode `0600`; nested directories must also be owned `0700`. Paths are
absolute, without `.` or `..`, and symlinks are refused. Policy and blob roots
must be distinct and cannot contain one another. Protect the TLS private key.

Example configuration (replace every path and home with your own):

```toml
version = 1
home = "home.example"
endpoint = "127.0.0.1:0"
policy_root = "/private/operator/policy"
blob_root = "/private/operator/blobs"
ca = "/private/operator/ca.pem"
certificate = "/private/operator/server.pem"
key = "/private/operator/server-key.pem"
```

The configuration file's parent is the private configuration root; proposal and
PEM paths must be beneath it. The config is at most 8192 bytes; proposals and each
PEM are at most 65536 bytes. Unknown fields and versions are refused. The endpoint
must be a literal numeric loopback address. Port zero explicitly requests an
ephemeral port; `serve` prints `ready PORT`, and a client must use that actual
nonzero port. Before binding, the server verifies the current server certificate's
chain against the single supplied local CA, server purpose, home name and key
match. Supply these credentials independently; this tool does not create them.

Create a private proposal with independently verified values. The hex strings below
are **placeholders**, not usable device credentials; replace each with the exact
64 lowercase hexadecimal characters for the certificate's SHA-256 DER digest and
the device's independently verified identity respectively. Replace the account,
device and validity interval as approved. Times are Unix seconds.

```toml
version = 1
generation = 1
reset_epoch = 0
homes = ["home.example"]

[[bindings]]
certificate_sha256 = "<64 lowercase hex characters>"
account = "member@home.example"
device = 1
identity = "<64 lowercase hex characters>"
valid_from = 1700000000
valid_until = 4102444800
approved = true
account_enabled = true
device_enabled = true
generation = 1
```

All binding generations must equal the proposal generation. The operator accepts
exactly its configured home and local account bindings, at most 32 bindings and
homes, and no extra fields. Approval and device identity are operator inputs, not
proof of OMEMO key possession or BBS membership.

## Commands and recovery

```sh
faction-provider init /private/operator/config.toml /private/operator/proposal.toml
faction-provider inspect /private/operator/config.toml
faction-provider serve /private/operator/config.toml
# After the serving process has stopped:
faction-provider apply /private/operator/config.toml /private/operator/next.toml
faction-provider reset /private/operator/config.toml 1
```

`init` is only for a new empty policy root. `apply` requires a higher generation
and a nondecreasing reset epoch. `reset` requires a higher epoch, advances the
generation and clears **all active bindings**, leaving every device unauthorized.
Certificate tombstones and history remain; reset does not issue or recover keys.
To restore selected rightful devices, first inspect the retained floors, then apply
an explicitly preapproved proposal with a higher generation and `reset_epoch` at
least the retained reset floor. A retained certificate digest cannot be remapped to
another principal. `inspect` prints only generation, reset epoch, availability,
binding count and history count. Commands return a fixed refusal diagnostic on
failure.
Offline commands refuse while another process owns the policy lock. `serve` runs
in the foreground; TERM or INT stops admission and drains owned work within the
existing five-second budget. There is no daemon installer or automatic reload.

Never delete or reinitialize established state to work around an error. Missing or
corrupt state refuses reopening. If publication fails or its outcome is uncertain,
the current runtime stops serving until reopened; do not assume rollback. Stop the
owner, inspect retained state and deliberately prepare a valid higher-generation
proposal after determining the actual state. An invalid proposal leaves the current
validated policy intact. Whole-root restoration to an older trusted copy is not
detected. See [policy and failure semantics](../dev-docs/provider-policy.md).

The provider allows one active connection, eight attempted operations, five seconds
for TCP and TLS admission and a ten-second accepted cycle. Attachment upload and
mail release retain their separate client limits. This preview does not establish
independent client interoperability, public listening or own-home delegation.

# Private attachment provider operator

This optional standalone executable prepares and serves an attachment provider
on numeric loopback. Its ordinary mode serves a same-home provider. It is a
software preview, not provider deployment or production enrollment. It needs a trusted operator's preapproved public policy and
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

## Check a complete proposal offline

The member can separately prepare a [public device card](provider-preparation.md)
and supply it with the selected public client certificate. Obtain the complete
proposal through your independent approval process: it is the source of every
binding, validity interval, flag, generation and reset epoch. The card and
certificate are self-asserted data, not proof of possession or approval. Never
request the member's private profile or key.

Place the card, public certificate and complete approved proposal under the private
configuration root, with the same owned `0700` directories and single-link `0600`
file rules above. Use a fresh output path there, outside policy and blob roots and
the configured key path:

```sh
faction-provider prepare /private/operator/config.toml /private/operator/card.toml /private/operator/client.pem /private/operator/approved.toml /private/operator/candidate.toml
```

This offline data check compares exactly one account/device target, its full identity,
home and client certificate digest against the complete proposal. It preserves every
binding and field present in that complete input, including disabled, revoked or false
flags, generation and reset epoch, in canonical output. It does not infer approval, insert or enable bindings,
compare the input with a retained policy snapshot, read private keys, connect or start
a listener. It may run while
the owner serves: it does not take the policy lock. A candidate is not applied or
approved by this command, and it does not export or merge the current registry.

Publication uses a new private file and does not overwrite. If a failure occurs after
publication, inspect the destination before retrying with a fresh path; do not assume
rollback. The card is at most 8192 bytes and the certificate and proposal at most
65536 bytes each. The certificate must contain one unambiguous public leaf.

Choose **one** separate authority-changing step: `init` for a new empty policy root,
or `apply` for an established root after stopping its owner. Current floors and
certificate history are checked there, so a stale candidate can still be refused.
`apply` replaces the whole registry with the supplied proposal. Review the complete proposal against the actual retained state; never rebuild it
from a partial card.

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
corrupt state refuses reopening. If a policy snapshot atomic write fails or its
publication outcome is uncertain, the current runtime stops serving until reopened;
do not assume rollback. Stop the owner, inspect retained policy state and deliberately
prepare a valid higher-generation proposal after determining the actual state. An
invalid proposal leaves the current validated policy intact. Whole-root restoration
to an older trusted copy is not detected. See [policy and failure semantics](../dev-docs/provider-policy.md).

The provider allows one active connection, eight attempted operations, five seconds
for TCP and TLS admission and a ten-second accepted cycle. Attachment upload and
mail release retain their separate client limits. This preview does not establish
independent client interoperability or public listening. The composed mode below
is under validation; native cross-home delivery and mail release remain incomplete.

## Explicit composed mode

This private software preview is under validation. Each operator independently
approves a complete policy and supplies certificates and fixed loopback routes.
There is no automatic enrollment, trust decision or discovery. Device and service
approval are separate; a card or certificate does not itself grant either. Keep
content, plaintext, vault and OMEMO keys on the member device. The ordinary version
1 commands and their same-home refusal remain unchanged.

Select both features explicitly, after the preflight in Build and prepare:

```sh
RUSTUP_TOOLCHAIN=1.96.0 cargo +1.96.0 build --manifest-path crates/faction-provider/Cargo.toml --locked --offline --features operator,delegation-verifier --bin faction-provider
```

For each home, create separate private configuration, policy and blob roots as
above. The following illustrates the configuration for `alpha.test` with one
invited peer `beta.test`. Replace *all* paths, ports, names and digest placeholders
with independently verified values. These names and paths are illustrative; they
are not provisioned by the operator. The peer must separately configure its own
home and reciprocal policy. Use fixed, nonzero numeric loopback ports: composed
mode cannot use the version 1 ephemeral-port example.

```toml
version = 2
home = "alpha.test"
endpoint = "127.0.0.1:41001"
policy_root = "/private/alpha/policy"
blob_root = "/private/alpha/blobs"
ca = "/private/alpha/config/alpha-ca.pem"
certificate = "/private/alpha/config/alpha-server.pem"
key = "/private/alpha/config/alpha-server-key.pem"
client_cas = ["/private/alpha/config/alpha-ca.pem", "/private/alpha/config/beta-ca.pem"]
publication_clients = ["publisher"]

[local_relay]
address = "127.0.0.1:41001"
http_host = "alpha.test"
server_name = "alpha.test"
leaf_pin = "<alpha server leaf SHA-256 DER digest>"
ca = "/private/alpha/config/alpha-ca.pem"
certificate = "/private/alpha/config/alpha-relay.pem"
key = "/private/alpha/config/alpha-relay-key.pem"

[[peers]]
home = "beta.test"

[peers.publication]
address = "127.0.0.1:41002"
http_host = "beta.test"
server_name = "beta.test"
leaf_pin = "<beta server leaf SHA-256 DER digest>"
ca = "/private/alpha/config/beta-ca.pem"
certificate = "/private/alpha/config/alpha-publisher.pem"
key = "/private/alpha/config/alpha-publisher-key.pem"

[peers.introspection]
address = "127.0.0.1:41002"
http_host = "beta.test"
server_name = "beta.test"
leaf_pin = "<beta server leaf SHA-256 DER digest>"
ca = "/private/alpha/config/beta-ca.pem"
certificate = "/private/alpha/config/alpha-inspector.pem"
key = "/private/alpha/config/alpha-inspector-key.pem"

[peers.resource]
address = "127.0.0.1:41002"
http_host = "beta.test"
server_name = "beta.test"
leaf_pin = "<beta server leaf SHA-256 DER digest>"
ca = "/private/alpha/config/beta-ca.pem"
certificate = "/private/alpha/config/alpha-relay.pem"
key = "/private/alpha/config/alpha-relay-key.pem"
```

The configuration parent is the private configuration root; put the file itself
there, for example `/private/alpha/config/config-v2.toml`. Proposal and PEM paths
must be absolute paths beneath that root. The ownership, mode, regular-file,
single-link and no-symlink rules above apply. The config, proposal and each PEM
are limited to 65536 bytes; the CA and certificate PEM inputs each contain one
certificate. Config arrays and peer lists are nonempty and at most 32 entries;
peer homes are distinct and cannot be the local home. Unknown fields are refused.
Each digest placeholder must be replaced by exactly 64 lowercase hexadecimal
characters; a leaf pin is the SHA-256 digest of the server leaf in DER form.
`client_cas` supplies incoming client trust roots. `publication_clients` restricts
incoming publication client IDs; it does not register or approve them. `local_relay`
must match the listening socket, and its `http_host` must equal `server_name`; the
same equality applies to each peer resource carrier. Each endpoint's CA verifies
the remote server; its certificate and key identify the local outbound service.
The peer's independently approved incoming policy must recognize that credential.

Prepare a complete version 2 proposal under the configuration root. It is not
generated by these commands. Include `version`, `generation`, `reset_epoch`,
`homes`, `local_home`, `issuer`, and the following arrays of tables. `homes`
includes the local home and invited homes; `local_home` equals config `home`.
Bindings are local device principals only. Each binding has the same fields as the
version 1 example above and its generation equals the proposal generation. Times
are Unix seconds. The 32-credential and history limits are aggregate across device
and service identities, and the home limit is 32.

| Proposal array | Required fields | Purpose |
| --- | --- | --- |
| `[[bindings]]` | `certificate_sha256`, `account`, `device`, `identity`, `valid_from`, `valid_until`, `approved`, `account_enabled`, `device_enabled`, `generation` | Independently approved local device and certificate. |
| `[[services]]` | `home`, `client_id`, `purpose`, `dns_san`, `certificate_sha256`, `enabled`, `valid_from`, `valid_until` | Independently approved service identity, certificate and role. Digest is the service certificate's SHA-256 DER digest. |
| `[[issuer_routes]]` | `sender_home`, `audience`, `relay_client_id`, `introspection_client_id` | Approved issuer route for a sender home and its relay/introspection IDs. |
| `[[resources]]` | `recipient_home`, `issuer`, `audience`, `relay_client_id`, `outbound_client_id`, `endpoint_id`, `address`, `http_host`, `ca_sha256`, `remote_generation`, `remote_reset` | Fixed retrieval and introspection association. |
| `[[publications]]` | `recipient_home`, `outbound_client_id`, `endpoint_id`, `address`, `http_host`, `ca_sha256`, `remote_generation`, `remote_reset` | Fixed outgoing publication association. |

All five arrays must be present; service, route, resource and publication arrays
are nonempty and at most 32 entries. For each association, `ca_sha256` is the
SHA-256 DER digest of the peer CA. Its address, HTTP host, CA and outgoing client
credential must match the configured endpoint. The `endpoint_id` refers to the
peer's `issuer-endpoint` service registration, whose DNS SAN and certificate
digest provide the expected server name and leaf pin; it is not a wire client ID.
Publication HTTP host equals the recipient home; resource HTTP host equals its
issuer. The publication and resource tuples for a peer must agree on exact remote
generation and reset values. Coordinate complete proposals at both homes when
those values change; intermediate mismatches deny operations and are never adopted
automatically. Retained remote floors cannot be lowered or forgotten.

For the example, the following registration roles illustrate the reciprocal
relationship. The IDs are operator-chosen examples, and each row requires all
`[[services]]` fields above with independently verified certificate digest, DNS
SAN, validity and enabled flag. The other home needs its own corresponding rows.

| Registration in alpha's proposal | Home and ID | Certificate represented / use |
| --- | --- | --- |
| `relay` | alpha.test, `relay` | alpha relay client; used by `local_relay` and `peers.resource`. |
| `outbound-observation` | alpha.test, `publisher` | alpha publisher client; used by `peers.publication`. |
| `outbound-introspection` | alpha.test, `inspector` | alpha inspector client; used by `peers.introspection`. |
| `observation` | beta.test, `publisher` | beta publisher client accepted for incoming publication; its ID is in `publication_clients`. |
| `introspection` | beta.test, `inspector` | beta inspector client accepted for incoming introspection. |
| `resource-client` | beta.test, `relay` | beta relay client accepted for incoming resource operations. |
| `issuer-endpoint` | beta.test, `endpoint` | beta server leaf and DNS name; referenced by both association `endpoint_id` values. |

For this paired example, the issuer route uses `sender_home = "beta.test"`,
`audience = "beta.test/attachments"`, `relay_client_id = "relay"`, and
`introspection_client_id = "inspector"`. The resource association uses
`recipient_home = "beta.test"`, `issuer = "beta.test"`,
`audience = "alpha.test/attachments"`, `relay_client_id = "relay"`,
`outbound_client_id = "inspector"`, and `endpoint_id = "endpoint"`. The
publication association uses `recipient_home = "beta.test"`,
`outbound_client_id = "publisher"`, and `endpoint_id = "endpoint"`. Both
associations use beta's fixed socket, host, CA digest and exact approved remote
policy generation/reset. The exact values must be coordinated and approved; these
illustrative IDs do not create trust or authorize a service.

```sh
faction-provider init-composed /private/alpha/config/config-v2.toml /private/alpha/config/proposal-v2.toml
faction-provider inspect-composed /private/alpha/config/config-v2.toml
faction-provider serve-composed /private/alpha/config/config-v2.toml
# After the serving process has stopped:
faction-provider apply-composed /private/alpha/config/config-v2.toml /private/alpha/config/next-v2.toml
faction-provider reset-composed /private/alpha/config/config-v2.toml 1
```

Initialize only a new empty policy root. Inspect prints generation, reset epoch,
availability, binding count and history count, not the complete policy. Serve
runs in the foreground and prints `ready PORT`; TERM or INT stops admission and
drains owned work under the existing five-second budget. Offline commands require
the policy lock; stop the owner before apply or reset. Apply replaces the complete
registry and service policy and requires a higher local generation and a
nondecreasing reset epoch. Reset requires an epoch above the retained floor,
advances generation and clears active device and service bindings. It preserves
certificate history and remote floors. To recover, inspect floors, independently
approve a complete higher-generation proposal with a reset epoch at least the
retained floor, and apply it; retained certificate identity mappings still apply.
Never delete or reinitialize established state to bypass refusal. If a policy snapshot atomic write fails or its publication outcome is uncertain,
the current runtime stops serving until reopened. Stop the owner and inspect retained
policy state before preparing a valid next proposal; do not assume rollback. This
inspection reports policy, not Store objects. For an uncertain attachment Store
result, use a freshly authorized exact retry; do not release mail based on a receipt
or an uncertain result. Applying policy does not repair an attachment. Whole-root
rollback to an older trusted copy is not detected. See [policy and failure semantics](../dev-docs/provider-policy.md).

Routes come from approved policy and fixed configuration, never a request URL.
One Runtime per home shares the existing limits, with at most one continuation;
the original eight attempts and ten-second accepted cycle remain. A receipt is
routing data, not a Store commit or mail release. Collection requires a fresh
authenticated connection. See the [carrier contract](../dev-docs/delegation-verifier.md).
This preview does not establish native cross-home mail delivery, a deployed provider,
public listening, independent interoperability, hardware or capacity support.

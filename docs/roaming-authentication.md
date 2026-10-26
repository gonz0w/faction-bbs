# Experimental roaming terminal authentication

This disabled-by-default software proof connects standard OpenSSH certificate
admission to an independently approved local Faction member. It is not an online
issuer or a production single-sign-on service. [Issue #223](https://github.com/gonz0w/faction-bbs/issues/223)
owns the proof; [the comparison](../dev-docs/research/0056-roaming-identity-handoff.md)
explains its alternatives and adoption gates.

## Member experience

An operator first approves your local membership and separately records your
consent to associate that member with an invited home and stable subject. Your
display handle alone cannot establish the association. A dedicated authentication
key and short-lived OpenSSH certificate enter a restricted transport account.
Successful SSH admission opens the same BBS menus and current local permissions
as approved password login. It grants no host shell or imported home-board role.

The optional native client profile adds a local table:

```toml
[experimental_terminal]
login = "fixture-roaming-user"
identity = "/absolute/private/authentication-key"
certificate = "/absolute/private/authentication-key-cert.pub"
```

These three values are all required together. Keep the authentication key distinct
from the profile's ordinary file-transfer key and every private-mail key. The
Files connection continues to use the existing `identity` and `factionfiles`
account. OpenSSH verifies the certificate during connection; local file readiness
does not establish certificate validity, destination approval or successful login.
The ordinary profile with this table absent retains password-based BBS entry.

No mail is decrypted at the visited board. The existing [member-device mail
entry](local-unread-mail.md) remains a separate local action.

## Operator boundary

This proof requires one restricted OS transport UID for each binding. Root-owned
SSH policy admits only its exact invited CA and destination/home-subject principal,
with forced frontend and existing forwarding/command restrictions. A separate
root-owned manifest maps observed kernel UIDs to already-enabled durable local
bindings. It neither enrolls members nor enables bindings. The native frontend
still passes only a TTY and bounded TERM; terminal text cannot choose an identity.

The optional service configuration key is `experimental_roaming_manifest`. Without
it the new admission path is absent. Its absolute path and all ancestors must be
root-owned, without writable group/world permissions or symlinks. The manifest is
one regular single-link file of at most 8192 bytes. Version 1 contains:

```toml
version = 1
board_identity = "<64 hexadecimal characters from this board>"

[[entries]]
uid = 42001
binding_id = 1
binding_generation = 2
issuer_generation = 2
```

Replace the placeholder with actual board identity; it is intentionally not a
usable configuration. At most four entries are accepted. Root, service-owner,
shared terminal, transfer and door identities cannot be roaming entries. Startup
checks each entry against durable board identity, enabled generations and current
approved local membership before binding listeners. The manifest must already be
readable by the service owner; it contains operator-readable identity metadata.

### Filesystem-owner commands

The `faction` owner binary manages manual associations on an existing initialized
board. Run it as the filesystem owner in a local operator session. An in-BBS sysop
role does not grant access to this CLI. Never install it as an SSH forced command.
The member binary, inherited terminal and shared member RPC do not expose these
commands. Roaming commands require effective schema20 and never initialize or
migrate a board. Take a stopped complete backup and use the ordinary setup/upgrade
path first. SQLite checks committed WAL state as well as the main database.

Start with inspection; it creates no association or session:

```sh
faction --state-dir /absolute/private/board roaming state
faction --state-dir /absolute/private/board roaming member-id LOCAL_HANDLE
faction --state-dir /absolute/private/board roaming manifest-preview
```

State shows stable issuer/binding/member IDs, invited home and CA fingerprint,
subject, principal, UID, consent reference, generations and enablement. The explicit
local `member-id` lookup supplies an existing local member ID before its first
association; it does not prove home identity or grant approval. Treat this as
private operator metadata. The manifest preview contains only the strict
service admission fields. An empty manifest contains `entries = []`.

After verifying an invited home CA out of band and separately obtaining local
member consent, use the numeric IDs and current generations shown by inspection:

```text
faction --state-dir BOARD roaming invite HOME_NAME CA_SHA256_FINGERPRINT
faction --state-dir BOARD roaming issuer-enable ISSUER_ID EXPECTED_GENERATION
faction --state-dir BOARD roaming link ISSUER_ID EXPECTED_ISSUER_GENERATION STABLE_SUBJECT EXACT_PRINCIPAL APPROVED_LOCAL_MEMBER_ID FRESH_TRANSPORT_UID CONSENT_REFERENCE
faction --state-dir BOARD roaming binding-enable BINDING_ID EXPECTED_GENERATION
faction --state-dir BOARD roaming manifest-export NEW_PRIVATE_FILE
```

The capitalized arguments are placeholders, not runnable enrollment instructions.
A typed home name, member handle or directory listing does not prove home identity
or consent. Linking requires an already-approved local member and records a
bounded explicit consent reference. New issuers and bindings start disabled.
Stale generations, unapproved members, reused UIDs and exhausted limits fail;
inspect again before deliberately retrying. Do not loop blindly on a conflict.

Export writes a new private file and refuses overwrites, links and unsafe output
paths. Keep the destination in an existing private directory owned by the operator.
Preview/export checks current approval and enabled generations; it cannot enable
or repair authority. A stale or revoked eligible binding fails export. Restore
leaves authority disabled, so it produces no active entries until deliberate
reconciliation. Neither inspection nor export checks or installs root SSH policy,
CA files or OS accounts.

Root must separately provision each fresh restricted transport account, install
its exact CA/principal policy and reconcile the reviewed manifest with reserved
host UIDs and the service configuration. The service independently validates its
root-owned manifest at startup. Keep this step separate from owner CLI output;
the CLI never executes generated commands or writes canonical root configuration.

The `roaming_owner_fixture` example refuses unmarked directories and remains only
an owned disposable test utility. Normal deployment templates do not provision
experimental accounts or modify SSH policy.

## Revocation and recovery

Disable the binding in the board first:

```text
faction --state-dir BOARD roaming binding-disable BINDING_ID EXPECTED_GENERATION
faction --state-dir BOARD roaming issuer-disable ISSUER_ID EXPECTED_GENERATION
```

Use the binding command for one association or the issuer command for all of its
associations. Then remove the matching root SSH policy and drain its
processes/connections. Current checks deny the old bound session and derived
chat, game and file authority. Independent password sessions retain ordinary local
approval rules. Disabling an issuer disables its bindings too; enabling that issuer
again cannot enable them. A new registry generation requires deliberate owner
reconciliation and service restart.

Never remap or reuse a transport UID. Old processes and TTYs can retain it after a
generation change. Relinking uses a fresh unused UID and new consent. The bounded
proof retains up to 32 immutable UID records and two invited issuer records; it
allows four enabled bindings per board. These are experiment limits.

Take a stopped complete backup before schema 20. Export/restore retains association
metadata for operator review but disables issuers/bindings and rotates generations.
Restore also changes board identity. Old manifests cannot reactivate restored
state. Review local approval, root accounts/policy and consent before any deliberate
re-enablement. See [export/restore](export-restore.md).

## Limits

The core sees an OS UID, not the certificate's leaf key, serial or expiry. Any
properly certified key under the allowed CA/principal can enter that UID. A process
compromised under that UID can act as its binding. This proof does not provide
per-device attribution or device-key revocation through the core.

An unavailable home prevents fresh issuance. An already-issued unexpired credential
can enter an available board while its local issuer, binding and member remain
allowed. Home-side revocation is not instant remote revocation. Certificate expiry
governs new SSH admission; it does not terminate existing sessions. Active local
revocation uses the binding/issuer/member checks. Ordinary local-password access
remains subject to local approval.

The installed NetBSD signer remains rejected. Externally issued synthetic
certificates qualify only the tested verifier path. Production signer packaging,
online issuance/enrollment and operational adoption remain
[separate adoption work](https://github.com/gonz0w/faction-bbs/issues/91).

[Host and NetBSD software evidence](../dev-docs/evidence/roaming-uid/README.md) records the qualified bounded handoff, retained failures, costs and adoption limits.

A separate [manual signer qualification](../dev-docs/research/0057-netbsd-signer-qualification.md) now identifies an uninstalled portable OpenSSH10.6p1 build that passes the native profile. The installed failing signer remains rejected; this does not install a home issuer or enable roaming.

The owner [accepted this per-binding account and admission-only expiry model](https://github.com/gonz0w/faction-bbs/issues/91#issuecomment-6422333314) for the first optional prototype. Devices sharing a binding retain one BBS identity. Installation, online issuance/enrollment and production operation remain separate; current caps and local consent/approval are unchanged.


[Owner CLI host/NetBSD evidence](../dev-docs/evidence/roaming-owner/README.md) records the source-matched command, service, classic and recovery controls, preserved corrections and cleanup.

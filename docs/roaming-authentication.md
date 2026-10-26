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

Manual owner association is an experiment interface, outside member commands/RPC.
The `roaming_owner_fixture` example refuses unmarked directories and exists only
for owned disposable software validation. It is not an installation or enrollment
utility. Normal deployment templates do not provision these experimental accounts
or modify SSH policy.

## Revocation and recovery

Disable the binding in the board first, then remove its root SSH policy and drain
its processes/connections. Current checks deny the old bound session and derived
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
online issuance/enrollment and operator acceptance of this per-UID model remain
[separate adoption work](https://github.com/gonz0w/faction-bbs/issues/91).

[Host and NetBSD software evidence](../dev-docs/evidence/roaming-uid/README.md) records the qualified bounded handoff, retained failures, costs and adoption limits.

A separate [manual signer qualification](../dev-docs/research/0057-netbsd-signer-qualification.md) now identifies an uninstalled portable OpenSSH10.6p1 build that passes the native profile. The installed failing signer remains rejected; this does not install a home issuer or enable roaming.

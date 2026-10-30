# 0061: Private attachment delivery contract

[Issue #292](https://github.com/gonz0w/faction-bbs/issues/292) asks whether one
provider, deliberate enrollment, own-home retrieval and independent modern client
can satisfy private attachment delivery together. Decision owner: project owner;
baseline: Faction `main` at `58e65c5`. This is completed source research, not
runtime adoption. Official sources were retrieved 2026-10-29 local time. Existing
[0058](0058-encrypted-attachment-profile.md) and [0060](0060-provider-device-boundary.md)
remain historical records. No build, deployment or runtime measurement is claimed.

## Decision

**NO-GO unchanged:** Prosody 13.0.7, the private Faction provider, and Kaidan
0.16.0 with QXmpp 1.17.0 do not form the requested complete combination. Three
independent source incompatibilities below stop qualification. No package or
protocol is adopted, and stopping here does not establish a complete transitive
license, feature, tooling or no-JS pass. Source incompatibility has high
confidence; the limited tunnel continuation has moderate source-level confidence.
Execution and the complete dependency graph remain unknown.

| Gate | Result for the unchanged combination | Basis |
| --- | --- | --- |
| G1 no JS/TS | Fail | Kaidan's actual QML/JavaScript below. |
| G2 approved membership | Unknown | Current Faction approval exists; the candidate's complete retrieval route does not. |
| G3 local authority | Unknown | Cross-home delegation authority is not specified. |
| G4 independent operation | Unknown | Complete operational graph is unqualified. |
| G5 privacy and ownership | Unknown | Private fetch and full failure/retention behavior are not established. |
| G6 domain/network separation | Unknown | The candidate lacks the required private route. |
| G7 useful BBS baseline | Unknown | Coexistence and failure isolation are not established by upstream sources. |
| G8 bounded optional extensions | Unknown | Full lifecycle and resource behavior are unqualified. |
| G9 distributable inputs | Unknown | Complete dependencies and license obligations are unqualified. |

These are qualitative gates, not scores. A failure cannot be outweighed by a
favorable comparison; unknown is not a pass.

## The candidate and its limits

1. Kaidan's pinned [top-level build](https://github.com/KDE/kaidan/blob/542320ae59b619c7598ed5bf50f14965afdc3d40/CMakeLists.txt),
   [client build](https://github.com/KDE/kaidan/blob/542320ae59b619c7598ed5bf50f14965afdc3d40/src/CMakeLists.txt)
   and [main QML](https://github.com/KDE/kaidan/blob/542320ae59b619c7598ed5bf50f14965afdc3d40/src/qml/main.qml)
   show Qt Quick/QML and actual JavaScript expressions. Qt documents
   [JavaScript in QML](https://doc.qt.io/qt-6/qtqml-javascript-expressions.html).
   This conflicts with Faction's no-JS/TS gate.
2. Pinned QXmpp [crypto](https://github.com/KDE/qxmpp/blob/86431213cbd4f5e1a70bd9c7a7db197006cf4c29/src/base/Crypto.cpp)
   and [encrypted file provider](https://github.com/KDE/qxmpp/blob/86431213cbd4f5e1a70bd9c7a7db197006cf4c29/src/client/QXmppEncryptedFileSharingProvider.cpp)
   default to CBC; the GCM path omits the appended authentication tag and
   finalization required for Faction's exact 12-byte IV, 16-byte tag and ciphertext
   length. It cannot be treated as the strict profile unchanged.
3. QXmpp's [HTTP file provider](https://github.com/KDE/qxmpp/blob/86431213cbd4f5e1a70bd9c7a7db197006cf4c29/src/client/QXmppHttpFileSharingProvider.cpp)
   retrieves an HTTP URL; it does not implement Faction's private blob URN or
   authenticated own-home delegation. A locator is not authority to fetch.

The [QXmpp build](https://github.com/KDE/qxmpp/blob/86431213cbd4f5e1a70bd9c7a7db197006cf4c29/CMakeLists.txt)
identifies Qt Core/Network/XML, OpenSSL and libomemo for OMEMO. Encryption-related
LGPL-3 and GPL implications require review in the selected distribution. These
observations are not a complete transitive or build-tool license inventory. Kaidan
and its QXmpp library are not two independent implementations. The historical
[Gajim byte proof](0059-attachment-byte-proof.md) is not modern end-to-end
interoperability. [Dino 0.5](https://dino.im/blog/2025/04/dino-0.5-release/)
announces unencrypted XEP-0447 sharing, not a substitute for this encrypted route.

[Prosody 13.0.7](https://prosody.im/doc/release/13.0.7) and its
[HTTP file-sharing module](https://prosody.im/doc/modules/mod_http_file_share)
are source candidates, not a verified Faction integration. [XEP-0363 1.2.0](https://xmpp.org/extensions/xep-0363.html)
specifies upload and GET URLs; an upload Authorization header does not supply
recipient-device GET authentication or own-home delegation. [XEP-0447 0.3.1](https://xmpp.org/extensions/xep-0447.html),
[XEP-0448 0.2.0](https://xmpp.org/extensions/xep-0448.html), and
[XEP-0420 0.5.0](https://xmpp.org/extensions/xep-0420.html) are candidate
formats, not proof that the required private route or profile interoperates.

## Existing authority and data path

The [provider integration](../provider-integration.md) accepts binary mTLS only
on same-home loopback and bounds frames to 8192 bytes. Its IQ adapter is in
process, not a network endpoint. [Operator](../../docs/provider-operator.md),
[preparation](../../docs/provider-preparation.md),
[operator source](../../crates/faction-provider/src/operator.rs),
[TLS source](../../crates/faction-provider/src/tls.rs), and
[client source](../../crates/faction-client/src/mail_device_provider.rs) define
the present interface. The [descriptor](../attachment-metadata.md) has a strict
private ciphertext-hash URN; HTTPS locators are inert and grant no destination
authority. Wire text, an SSH account, an XMPP password or a home certificate alone
never creates a provider actor.

## Proposed enrollment contract

For the same-home continuation, the member would generate the TLS key and a
[standard CSR](https://www.rfc-editor.org/rfc/rfc2986.html) locally. The operator
must independently verify account control, the full OMEMO public identity and
device, and the TLS leaf digest. An independently supplied offline CA would issue
the certificate; the existing prepare/init/apply interfaces then install a
complete independently approved proposal. No CA issuer or enrollment workflow is
implemented by the current preparation commands. A public card is self-asserted, not approval. A CSR
self-signature proves only possession of its TLS key; the first real TLS operation
also proves that key, not the OMEMO private key. A copied card without the approved
TLS key fails. An approved TLS-key holder can misassert an OMEMO public identity
if the operator did not verify it: TLS cannot prevent that mistake. Device private
keys stay on the device; the operator CA key may reside on the operator host.

Replacement uses a higher generation, retires the old certificate and retains
history. Reset clears active bindings and requires fresh approval; a password reset
neither approves a device nor recovers a key. A lost OMEMO identity requires fresh
peer trust and does not recover prior history. Each operation must respect current
generation, expiry, revocation and reset; an uncertain commit requires retry of
the exact persisted operation, with fresh authorization.

## Limited same-home continuation

For [#293](https://github.com/gonz0w/faction-bbs/issues/293), source continuation
is feasible after review: member client → end-device mTLS within a separately
authorized SSH local forward → home-loopback Faction provider → existing ciphertext
store. Encrypted XMPP mail remains separate. The recipient performs the equivalent
fetch, verifies locally and explicitly saves without overwriting. This is a source
proposal, not a tested remote integration. Both client and provider loopback guards
and TLS CA, home name and leaf pin remain effective even over a localhost socket.
SSH reachability does not supply a provider principal.

Use a dedicated forward-only account/key, separate from ordinary BBS and SFTP
accounts and their `DisableForwarding` policy. The current OpenBSD
[sshd_config](https://man.openbsd.org/sshd_config) and [ssh](https://man.openbsd.org/ssh)
document the relevant semantics, not a pinned target binary: `MaxSessions 0`
blocks shell, login and subsystem sessions but is not a forwarding-channel or
connection cap; local-only TCP forwarding and exact `PermitOpen` loopback host/port
constrain the destination. Disable StreamLocal, agent, X11 and tunnel forwarding.
`DisableForwarding yes` overrides forwarding permissions and must not be weakened
on the ordinary lane. An explicit loopback local forward with `-N` avoids a remote
command. `ExitOnForwardFailure` establishes forward setup, not destination readiness;
a real TLS operation must establish the latter. Exact host and NetBSD binaries,
versions and full selected graph require qualification; a manual page or source
tag does not establish the identity or behavior of an installed executable.

Proposed observable boundaries include a real remote connection with both loopback
checks retained; independent manual approval and a complete file journey; denial for
missing/wrong key, copied card alone, substituted identity, expiry, live revocation
and reset; ordinary BBS/SFTP forwarding denial; restart, interruption and exact
retry; and independent ordinary mail, BBS and files during SSH/provider outages.
Use bounded helpers and cleanup, inspect scoped private store/log data, and exercise
host and NetBSD software fixtures in classic and Unicode modes. These are proposed
oracles, not observations; physical hardware, 250-user runs and hardening are not
routine gates. Source work requires the existing reviewed issue authority; real host
or account activation requires separate operational authority.

## Cross-home and independent-client choices

[#294](https://github.com/gonz0w/faction-bbs/issues/294) remains blocked on the
owner's choice: may an invited recipient home vouch for its currently approved
exact device, or must the sender independently verify the device credential? SSH
forwarding supplies neither delegation. The chosen contract must bind:

| Claim | Required scope |
| --- | --- |
| Issuer | Exact invited recipient home. |
| Subject | Account, device and full identity. |
| Audience | Exact sender home and service. |
| Resource | Committed ciphertext and allowed fetch/status operations. |
| Lifetime and policy | Bounded expiry and current issuer, device, peer and reset state. |
| Retry | Exact idempotent operation with fresh authorization. |

A compromised vouching home can impersonate its claims and obtain ciphertext, but
not thereby obtain the device key or plaintext. [RFC 8705](https://www.rfc-editor.org/rfc/rfc8705.html)
is a standard certificate-bound token mechanism, not an existing integration:
authorization server, claims, revocation and dependency graph remain absent. No
custom token cryptography is selected here.

[#295](https://github.com/gonz0w/faction-bbs/issues/295) remains blocked on the
smallest owner choice: permit a separately isolated, explicitly corrected headless
QXmpp-based reference with no JS and strict GCM, or keep interoperability pending.
Permission would not qualify that reference or adopt a product dependency. Actual
reciprocal encrypted mail, descriptor, blob and private-access exchange with a
pinned independent reference is required, as are its complete selected graph and
resolution of GCM and private authentication. The no-JS requirement remains intact.

Homes can observe ciphertext size, timing, routing and public identities, and can
deny service; retained copies cannot be recalled. Reversal of a future deployed
wire or identity choice may require coordinated migration and risks history loss,
so no migration or rollback guarantee follows from this research.

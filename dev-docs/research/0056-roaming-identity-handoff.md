# Roaming identity: a trusted path into local membership

Date: 2026-10-26. [Issue #222](https://github.com/gonz0w/faction-bbs/issues/222),
child of [#91](https://github.com/gonz0w/faction-bbs/issues/91).
Status: comparison complete; experiment proposed, no authentication profile adopted.
Decision affected: [ADR0031](../decisions/0031-home-identities-and-client-encrypted-mail.md).
Stopping condition: choose a falsifiable first handoff proof and name its limits.

## Recommendation and existing boundary

Prove OpenSSH certificate admission through a root-provisioned transport UID for
each local binding, followed by a real owner-created Faction session. Begin with
two invited homes, two boards, four bindings and one synthetic authentication
keypair per binding. Five-minute certificates are a proposed fixture bound, not
a product default. Keep ordinary local login and the normal resource envelope.
This is a proposed experiment. Online issuance, self-service enrollment and product
adoption remain outside it; the manual enrollment contract is specified below.

A home owns its namespace. Each visited board independently approves its local
member and assigns roles. Bindings identify an invited issuer plus a stable home
subject; display handles and addresses are presentation. Authentication keys stay
separate from mail-encryption
keys. No visited board receives home passwords, mail keys or automatic membership.
A directory listing is never issuer trust. Explicitly installed invited issuers
can support this experiment without implementing deferred contacts #155.

The inspected code at `92c7f629` has these concrete seams:

| Component | Current behavior and missing work |
| --- | --- |
| [Access framing](../../crates/faction-access/src/lib.rs) | `FBSTTY01` carries bounded TERM and one TTY descriptor; it has no authenticated principal field. |
| [Native frontend](../../native/faction-tty/main.c) | Passes a terminal, without a home identity assertion. |
| [Kernel bridge](../../crates/faction-unix/src/lib.rs) | Supplies peer UID; access code checks TTY ownership. |
| [Shared owner](../../crates/faction-service/src/main.rs) | Admits one configured terminal UID. A bounded trusted UID map would be new behavior. |
| [Core](../../crates/faction-core/src/lib.rs) | Creates opaque sessions after password verification and checks current member epoch/status. A restricted owner-only binding/session path would be new behavior. |
| [Native profile](../../crates/faction-client/src/profile.rs) | Fixes terminal login to `factionterm`; certificate/provisioned-login configuration would need explicit support. Keep file-transfer identity separate. |

Typed addresses, terminal frames, environment variables and writable
`SSH_USER_AUTH` files cannot fill that gap. Authorization lookup commands run
before completed key-possession proof; their invocation cannot attest a login.

## Compare the mechanisms

Primary sources were read on 2026-10-26. These are documented mechanisms and code
inspection, not new runtime or interoperability measurements.

| Candidate | Established mechanism | Remaining adoption gate |
| --- | --- | --- |
| OpenSSH certificates plus provisioned UIDs | SSH verifies the certificate, allowed principal and possession; the kernel gives the owner a UID/TTY identity. Existing NetBSD verifier controls are historical evidence. | Root account provisioning, exact CA/principal policy, UID lifecycle, real binding/session implementation and qualified signing package. |
| Shared UID plus privileged post-authentication relay | Could retain one OS account while carrying verified certificate metadata. | No maintained trusted relay into Faction was established. An environment/auth-info file or pre-auth lookup is insufficient. |
| OAuth device authorization | A device obtains authorization through a separate verification interface and polls a device code. | Qualified home/provider, HTTPS, user experience and an explicit board-owned pending transaction. It does not establish device-key possession by itself. |
| Authorization code with PKCE and OIDC | PKCE binds code redemption to the initiating client; OIDC validates identity as `(iss, sub)` with recipient-client, signature, expiry and applicable nonce checks. | Qualified native/provider graph and a trusted association with the actual board session. A token for the native client is not automatically for the board. |
| OAuth access tokens with DPoP | Resource audience/scope and HTTP proof-of-possession can constrain token use. | Correct token profile and HTTP checks plus independent SSH-session association. DPoP does not attest an SSH TTY or sign a general request body. |

Device authorization is distinct from PKCE. An OAuth access token need not be a
JWT; do not apply ID-token rules to every token. Private operation still needs TLS
for the authorization/token endpoints. The native-app loopback redirect exception
does not permit plaintext home APIs. See [RFC8628](https://www.rfc-editor.org/rfc/rfc8628.html),
[RFC7636](https://www.rfc-editor.org/rfc/rfc7636.html),
[OIDC Core1.0 with errata set2](https://openid.net/specs/openid-connect-core-1_0.html),
[RFC9068](https://www.rfc-editor.org/rfc/rfc9068.html),
[RFC8707](https://www.rfc-editor.org/rfc/rfc8707.html),
[RFC9449](https://www.rfc-editor.org/rfc/rfc9449.html),
[RFC8252](https://www.rfc-editor.org/rfc/rfc8252.html) and
[RFC9700](https://www.rfc-editor.org/rfc/rfc9700.html).

Prosody's [OAuth module](https://modules.prosody.im/mod_http_oauth2) is marked Alpha,
updated 2026-10-10, with incomplete OIDC discovery including missing JWKS.
Rust [oauth2 5.0.0](https://docs.rs/oauth2/5.0.0/oauth2/) documents client
PKCE/device flows; [openidconnect 4.0.1](https://docs.rs/openidconnect/4.0.1/openidconnect/)
is a client library, not a complete provider. No NetBSD build, no-JS dependency/UI
graph, DPoP interoperability or resource envelope is qualified for these candidates.
No library or new service is adopted by this comparison.

There is no weighted score: current integration evidence favors the SSH experiment;
unknown provider, authority and portability gates prevent a production winner.

## Proposed complete path

1. The member device holds a dedicated authentication private key. The experiment
   manually issues a certificate from its invited home authority for an exact
   destination-board/home-subject principal.
2. The visited operator provisions a restricted OS transport account and root-owned
   policy associating that UID with the exact CA and principal. No raw-key,
   password, alternate-command or forwarding route may bypass that policy.
3. OpenSSH authenticates the connection. Its public-key authentication signature
   includes the SSH session identifier and request. The SSH endpoints derive that
   identifier from the first key-exchange hash and its fresh exchange inputs; sshd
   checks the device signature against it. A signature copied into another session
   is not valid authentication. There is no new application nonce or one-use
   certificate; the certificate nonce is not a live login challenge. A certificate remains legitimately
   reusable with its private key during its validity. See
   [RFC4252 §7](https://www.rfc-editor.org/rfc/rfc4252.html) and
   [RFC4253](https://www.rfc-editor.org/rfc/rfc4253.html).
4. The existing frontend transfers the TTY. The owner verifies kernel peer UID and
   TTY ownership, then resolves a trusted local binding. Member text cannot choose
   a UID, issuer, subject, member ID or role.
5. A new owner-only core path checks that binding and the independently approved
   local member, then creates a genuine opaque session. Stored local passwords,
   password forwarding and mocked membership are not acceptable substitutes.
6. Every protected action checks member status/epoch and binding state/generation.
   Unlinking or disabling an issuer must invalidate affected active bindings too.

Root-owned per-user `authorized_keys` entries with `cert-authority` and exact
`principals` are documented building blocks. Keep CA and namespace restrictions
per binding; a global CA collection with shared principals can let one home assert
another's namespace. Preserve existing forced-command/forwarding restrictions;
`ForceCommand` alone does not disable forwarding. See
[NetBSD sshd](https://man.netbsd.org/sshd.8) and
[sshd_config](https://man.netbsd.org/sshd_config.5).

The deployed certificate format is an OpenSSH extension. Its IETF proposal,
[draft-ietf-sshm-cert-01](https://datatracker.ietf.org/doc/html/draft-ietf-sshm-cert-01),
is dated 2026-06-14 and expires 2026-12-16; it is not an RFC. The
[OpenSSH protocol references](https://www.openssh.org/specs.html) distinguish it
from the established SSH authentication RFCs.

## Proposed durable enrollment contract

Persist a versioned board-local binding keyed by board identity, invited issuer
identity/namespace and stable home subject. Store the local member ID, transport
UID, binding generation and enabled state; issuer enablement is independently
revocable. These are ordinary operator-readable identity metadata, not mail keys.
Use the existing private board storage/export boundary with explicit migration
and restore rules. No table or storage format is implemented by this report.

Only the local operator creates or enables the initial association in this proof.
Require an already-approved local member, that member's independently established
local consent, an explicitly invited home issuer, and the matching root-provisioned
OS account/CA/principal policy. A matching display handle is never enrollment.
Member input, a certificate lookup callback or a directory listing cannot create
or relink a binding. Keep the root transport policy separate from the BBS record;
leave the record disabled until both have been reconciled by the operator.

Unlinking disables the record and rotates its generation, invalidating active
bound sessions without deleting the local member or changing mail keys. Disable
the board's issuer/bindings before removing its SSH policy so active BBS use is
denied during that operational transition. Relinking requires fresh operator
review and local consent, a new generation and a fresh unused UID after draining
the old binding. Losing a key or home domain is not automatic account migration.
Local-password access remains subject to ordinary local approval and epoch checks.

Restore preserves binding metadata for review but leaves bindings disabled, with
fresh board/session authority. An old root manifest or matching UID number cannot
automatically reactivate them. Reconcile the actual accounts, issuer policy,
member consent and restored generation before deliberate operator enablement.

## Trust limits and recovery

The handoff attests OS identity. It does not give Board a fresh certificate serial,
leaf-key fingerprint or expiry deadline. A process compromised under that UID can
impersonate that binding; it must not select another binding. Root and the invited
home issuer remain trusted authorities for their respective roles.

One synthetic keypair per binding limits this first experiment; it is not a
server-enforced leaf-key pin. A fresh key with a valid certificate from the same
allowed CA/principal may authenticate. Several devices sharing a UID lose reliable
per-device attribution. Do not claim independent device revocation or grant mail
decryption/enrollment from this authentication result.

Certificate expiry and a KRL/CA change govern subsequent SSH admission. They do
not automatically end an existing BBS session. Binding generation/status and member
epoch must handle active revocation. If certificate-expiry termination or fresh
leaf-key attribution is required, this UID-only handoff is insufficient.

Never reassign a transport UID while old processes, TTYs or connections can retain
it. Disable the old binding, invalidate sessions, drain/reap and use a fresh unused
UID. A new generation alone cannot prevent an old process presenting a reused OS
UID. Restore must disable bindings until the operator reconciles account, issuer
and board mappings; restored state must not revive old sessions or import roles.

The [#92/#195 signer diagnosis](0052-netbsd-signing-exit.md) remains binding:
the tested installed signer is rejected despite producing artifacts before
abnormal exit. Require normal successful exit, independent verification, pinned
package/source identity and an accepted update path before operational issuance.
[OpenSSH10.6/10.6p1](https://www.openssh.org/releasenotes.html), released 2026-10-06,
does not itself qualify the tested NetBSD package. Externally issued synthetic
certificates can test a verifier; they do not prove native issuance.

## Explicit outage policy for the experiment

A home outage prevents fresh issuance/enrollment. An already issued, unexpired
certificate can still enter an available board when its local issuer/binding and
member remain enabled/approved; there is no online home introspection in this
first proof. Home-side account revocation therefore does not instantly revoke an
existing certificate at another board. Local deny/unlink/issuer disable or later
certificate expiry must provide the stated admission policy.

Expired or future certificates fail new SSH admission. A connected BBS session
continues only under its normal lifetime rules and current local member/binding
checks; expiry alone does not terminate it. Ordinary approved local-password
login remains an explicit fallback, never home-password forwarding. Other invited
homes remain independent. An unavailable board cannot create a destination
session; direct member-device mail remains separate and may work if its home is
available. Neither outage silently recovers keys, enrolls devices or copies mail.

## First proof and adoption gates

The experimental follow-up must exercise actual restricted SSH, native frontend,
client and owner-created core sessions against two independent homes and boards.
It must test same-handle/different-home separation, wrong CA/board/principal/UID,
wrong private key, expired/future credentials, current local approval, active
unlink/issuer/member revocation, session teardown, restart and disabled restored
bindings. Demonstrate legitimate certificate reuse and reject reuse of an
authentication signature for a different session with an appropriate SSH oracle;
state the layer actually tested. No invented one-use-certificate requirement.

Test one unavailable home while the other works, the documented unexpired offline
credential policy and ordinary local-password fallback. Use synthetic keys/data
and marked disposable software fixtures; keep the saved playground stopped and
prove cleanup. Retain classic access, no-JS, member-device mail keys and normal
eight-session, credential, helper, output and drain budgets. No physical SBC or
250-member gate is needed for this proof.

Owner choices before product adoption remain the account-provisioning cost, the
device granularity and admission-only certificate expiry. The native signer and
trusted binding/session implementation are independent gates. The next experiment
may qualify or reject this handoff without adopting a complete roaming protocol.

Confidence is high for the inspected existing boundary and cited mechanism
semantics, moderate for feasibility, and unknown for the unbuilt integrated
profile, no-JS provider graphs and costs. No new runtime test, provider, issuer,
VM or listener was created during this research. Reversal is cheap before adoption;
changing deployed bindings, UID assignments or key enrollment later requires an
explicit migration and recovery design.

The finite experimental follow-up is [#223](https://github.com/gonz0w/faction-bbs/issues/223).
Its issue owns acceptance and execution; this report grants no runtime adoption.

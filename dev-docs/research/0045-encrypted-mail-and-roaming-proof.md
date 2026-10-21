# Two-home encrypted mail and scoped roaming proof

Status: completed bounded experiment; product integration/adoption still pending.
Date: 2026-10-21. Parent [#84](https://github.com/gonz0w/faction-bbs/issues/84).
Owner: founder; ADR [0034](../decisions/0034-encrypted-mail-reuse-proof.md).
Stopping condition: actual encrypted cross-home/offline/archive behavior plus
native scoped-identity controls, dependency findings and concrete next build gates.

## Scope and decisions

The owner requires independent invited homes, member-device encryption and a native
terminal client. A visited board retains local approval; it never gets home mail
keys/passwords. Attachments stay [#88](https://github.com/gonz0w/faction-bbs/issues/88).
This increment follows merged #89 without changing any maintained product runtime,
DB schema, SSH configuration, owner/credential limit or default eight sessions.

We built a separate protocol experiment, not a Python product service/client. Two
Prosody 13.0.7 processes run under different UIDs and private stores on one software
NetBSD 11 ARM64 VM (two CPUs/1 GiB). They authenticate each other's domains with
fixture-CA TLS, explicit literal loopback routing and a fixed peer allowlist.
No public registration, HTTP listener, public DNS or real account/peer is used.
The Python reference devices run on the developer's macOS host, independently of
the BBS. Same-VM homes do not prove independent-machine compromise/failure isolation.

## Alternatives and gates

| Candidate | Observed use / decision |
| --- | --- |
| Server decrypts/rendered SSH mail | Reject: fails the owner's server-blind requirement. |
| Fresh Rust ratchet/crypto implementation | Reject as the first move; review/interoperate an existing implementation. |
| Prose Rust client | Its inspected manifest pins Signal's git library v0.41.2; not selected. Complete portability/source/license graph and protocol interop remain unknown. |
| Slixmpp-OMEMO plugin | Not integrated. It is AGPL-3.0-only, and its inspected 2.1.0 source prepares OMEMO 2 but lacks SCE packing/opening. The extension number alone is not interoperability evidence. |
| MIT OMEMO/Twomemo core + standalone SCE adapter | Selected **reference control only**. Real `urn:xmpp:omemo:2` envelopes and `urn:xmpp:sce:1` subject/body routing pass internally. No new crypto; no independent implementation/current-XEP conformance or native product adoption claim. |
| Private OAuth/OIDC device/PKCE login | Still a candidate. Prosody's module documents device authorization/OIDC but remains alpha, with partial discovery/JWKS support. No live OAuth proof or chosen provider/library. |
| OpenSSH user certificates | Native verifier control passes with board-scoped principals and per-issuer local grants. Online issuance, trusted identity handoff and BBS membership binding remain unimplemented. Native signing has a recorded failure. |

Custody, no-JS and local-approval requirements are gates, not weighted scores.
There is no numeric stack winner or claim that a library's release label proves
security. The weakest product gates remain native crypto reuse/interoperability,
keystore/device recovery and a trustworthy end-to-end local identity binding.

## Actual evidence

[Sanitized results](../evidence/encrypted-mail-netbsd/results.json) pin the actual
executed experiment sources. The two original format inputs are preserved under
`proof-inputs/`: after the network run, UTF-8-only decoding was independently
hardened and passed three focused host format tests, including alternate-encoding
refusal. The network matrix was not repeated after that format-only change;
separate hashes/results retain that distinction. [Reproduction](../../experiments/084-mail-proof/README.md)
contains build/setup/oracle commands. No dependency enters the product Cargo graph.

The **12 mail controls** pass: both home logins/public bundles; uninvited route
denial; encrypted subject/body cross-home delivery; ratcheted reply; missing trusted
key refusal; fresh ciphertext modification refusal; valid delivery after tampering;
plaintext downgrade refusal; queued ciphertext/same-device reconnect; bounded MAM
reads with repeated archive IDs; refusal of a newly server-added unverified device;
and absence of private markers from outgoing envelopes. Keys remain in client
memory; an existing device state is retained for reconnect, not recovered by password.
No legacy-OMEMO fallback is enabled in the control. SCE subject/body and sender/
recipient bindings are checked by a small format adapter; Twomemo performs crypto.

Known synthetic body/subject markers were absent from **36 actual server files**
(97,130 bytes) spanning private stores/logs/config. This is a narrow passive
observation, not a proof about arbitrary compromise. Required routing/timing/size
and public device information remain visible. The deliberate plaintext-downgrade
probe is itself plaintext and may be stored; it is not private mail. Local member
memory and the private test profile intentionally contain synthetic plaintext.

The **11 native certificate controls** pass: invited home A and independent home B
with the same local handle, wrong visited-board audience, an issuer claiming another
home's namespace, unknown issuer, expired/not-yet-valid certificate, missing device
private key, uncertified key, fixed forced command and local revoke with a positive
other-home control. Certificates use exact `board-b/alice@homea.test` style principal
allowlists tied to each CA. Reusable certificates are time-limited credentials;
SSH challenge/signature authentication supplies fresh connection proof. This is not
a custom token scheme or a one-use certificate. A home may assert identities only
within its invited namespace, and no role/approval is imported.

These were **macOS-issued synthetic certificates verified by native NetBSD SSH**.
No online issuer or Faction member login is implemented. Rejected attempts use
separate temporary guest loopback aliases to isolate certificate policy; base
source penalties stay unchanged. The test listener is private and removed.
Never trust a member-supplied principal, environment value or user-writable
`SSH_USER_AUTH` file as a completed identity handoff. #91 must prove a trusted
connection-bound path plus durable local membership approval/revocation.

A narrow detached-process sample observed **17,780/18,048 KiB RSS** for the two homes.
These are samples, not peaks, whole-system budgets, client costs, combined BBS load,
250-user support or physical SBC evidence. The packaged Prosody 13.0.3 supplies
build dependencies only; the actual executed source/runtime is 13.0.7.

## Contrary findings retained

- Prosody 13.0.7's NetBSD preset hard-codes Lua-5.2 paths. A recorded two-line build
  patch selects installed Lua 5.4; C modules also require `-D_NETBSD_SOURCE`.
  Skip unused upstream example certificates; create independent fixture TLS keys.
- Slixmpp 1.17.0's optional Rust/PyO3 graph contains a prohibited JS documentation
  asset. Reject that graph and use its upstream Python JID fallback in this
  **reference experiment**. Inspect all 21 selected runtime wheel/file identities;
  do not prune a linked dependency's JS files or call a filename scan an audit.
- OMEMO 2.1.0 rejects an explicitly positive per-message skipped-key cap because
  of a constructor condition. The experiment uses the supported default, bounded
  to 64 per session/message; the crypto library is unmodified. This is another
  review/adoption gate, not a reason to implement new cryptography.
- Base NetBSD `ssh-keygen -s` exits on **SIGSEGV (-11)** after producing certificates
  in four minimal cases: basic, validity, cleared extensions and forced command.
  [Safe reproducer](../../experiments/084-mail-proof/signing_probe.py) disables core
  dumps and removes synthetic keys. Do not accept failed signer output. Record
  exact binary identity and defer native issuance under [#92](https://github.com/gonz0w/faction-bbs/issues/92);
  no confirmed upstream cause/all-NetBSD claim or external bug report is implied.

## Sources, confidence and reversal

Primary sources retrieved 2026-10-21:

- [XEP-0384 0.9.1](https://xmpp.org/extensions/xep-0384.html) and
  [XEP-0420 0.5.0](https://xmpp.org/extensions/xep-0420.html): Experimental wire/SCE
  requirements and trust/metadata limits. Namespace use here is not complete
  conformance or independent-client interoperability evidence.
- [MIT core/backend API](https://github.com/Syndace/python-omemo) and
  [Twomemo](https://github.com/Syndace/python-twomemo): existing ratchet/crypto reuse.
  [Plugin source](https://github.com/Syndace/slixmpp-omemo) and
  [Prose manifest](https://raw.githubusercontent.com/prose-im/prose-core-client/master/crates/prose-core-client/Cargo.toml)
  establish the specific unselected dependency/profile findings.
- [Prosody source build](https://prosody.im/doc/installing_from_source),
  [S2S policy](https://prosody.im/doc/s2s), and
  [literal target override](https://modules.prosody.im/mod_s2sout_override): server
  reuse and private routing. Official HTTPS source hashes are pinned; no independent
  Prosody release-signature validation is claimed. NetBSD image provenance follows
  the previously signed official release manifest/fingerprint in research 023.
- [NetBSD sshd_config](https://man.netbsd.org/sshd_config.5) and
  [ssh-keygen](https://man.netbsd.org/ssh-keygen.1): standard CA/principal, key
  possession, expiry and forced-command controls. Native signing failure is our
  measured fixture result, not a documented universal defect.
- [Prosody OAuth/OIDC module](https://modules.prosody.im/mod_http_oauth2),
  [RFC 8628](https://www.rfc-editor.org/rfc/rfc8628.html) and
  [RFC 7636](https://www.rfc-editor.org/rfc/rfc7636.html): separate standards-based
  auth candidates, not proof of implemented Faction SSO.

High confidence in recorded local controls and observed failures; moderate in
reuse feasibility; unknown for the complete product security/recovery/interop and
resource contract. Changeable optional experiment, no private-state migration.
Protocol/key-history adoption would have much greater reversal cost and remains
behind native integration, independent review and recovery evidence.

Continue [#90](https://github.com/gonz0w/faction-bbs/issues/90) for native mail UI,
keystore/verified devices/recovery, [#91](https://github.com/gonz0w/faction-bbs/issues/91)
for issuer/local membership binding and #92 for signing. #84 remains open. Keep
#88's attachments device-encrypted, with private names/keys inside E2EE envelopes
and independent ciphertext quotas/save/recovery; current file areas remain distinct.

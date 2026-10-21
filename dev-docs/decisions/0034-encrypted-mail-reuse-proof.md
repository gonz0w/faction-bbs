# 0034: Separate encrypted-mail reuse and roaming-admission experiments

Status: accepted experiment boundary, 2026-10-21; no production crypto/SSO adoption.
Owner request: merge #89 and move #84 forward, retaining attachments under #88.

Use independent XMPP homes and existing member-side OMEMO-2 cryptography for a
bounded research proof. Keep home authentication, verified mail-device keys,
visited-board local membership and server transport authority separate. The
prototype contains no crypto implementation: MIT OMEMO/Twomemo performs ratchets
and payload encryption; a separate bounded SCE format adapter holds subject/body
and verifies sender/recipient. No server plaintext fallback or replicated mailbox.

Two separately owned Prosody 13.0.7 instances in one NetBSD VM and member-side
reference clients pass 12 mail controls. Native OpenSSH verifies 11 scoped
certificate controls with externally issued synthetic certificates. This establishes
specific feasibility, not Faction mail/client/roaming support or complete current
spec conformance. [Research 0045](../research/0045-encrypted-mail-and-roaming-proof.md)
and [results](../evidence/encrypted-mail-netbsd/results.json) contain contrary findings,
versions/provenance, observation limits and primary sources retrieved 2026-10-21.

Reject server-side decrypt/render and fresh protocol cryptography. Do not adopt
the AGPL plugin or its unfinished SCE path. The reference library's optional
Rust/PyO3 build violates the no-JS dependency policy; select its upstream Python
fallback **only for the research client**. The actual product remains Rust.
Review a maintained native adapter's complete dependency/profile/portability graph
and prove independent-client interoperability before selection. Python is not a
new product runtime or a permanent proxy around those gates.

Keep OpenSSH certificate admission as a candidate alongside private OAuth/OIDC
for scoped home identity. The native certificate signer has a measured crash;
externally issued control certificates do not clear that gate. No home-password
forwarding, reused mail keys or blanket peer privilege. Current BBS local login
stays. Require trusted connection binding and local approval/revoke before any
principal becomes a member session; SSH admission alone is not BBS SSO.

Reversal is cheap while optional scripts/fixtures remain separate from product
state. Wire/history/keystore adoption is costly and remains deferred. Native
mail integration and member-held encrypted keystore/recovery are #90, scoped
online issuer/membership binding #91, native signing investigation #92. #84 and
#88 remain open. Reopen decisions for actual interoperability, reviewed native
libraries, verified recovery and complete issuer-to-membership proofs. No product
owner/session/credential limits or 250-user/hardware claims change.

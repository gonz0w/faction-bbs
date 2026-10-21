# Encrypted mail and roaming identity proof (#84)

Status: bounded protocol research completed; product integration remains open. Owner authorized merging #89 and
continuing #84, with device-encrypted attachments retained under #88.

## Gates and approach

Keep independent invited home domains, a local native terminal client, member-side
mail encryption and separate visited-board approval. No plaintext mail fallback,
new cryptosystem, password forwarding, central identity hub, JS/TS or automatic
public listener. Existing BBS/client/runtime/defaults remain unchanged until a
concrete profile passes reuse/dependency/interoperability checks.

1. Verify exact-head CI/review state and squash-merge #89; start this branch.
2. Compare actual Rust/C/reference OMEMO profiles and dependencies, two-home XMPP
   routing/queue/archive, and standards-based scoped roaming proof candidates.
   Use existing implementations as controls before inventing integration code.
3. Build a private software fixture with synthetic accounts; verify encryption,
   strict device trust, offline return, tamper/downgrade refusal and server-store
   plaintext exclusion. Separately test identity issuer/target binding and local
   deny/revoke; do not label SSH admission alone as BBS SSO.
4. Record versions/provenance, actual pass/failure outcomes and remaining limits.
   Keep concrete implementation follow-ups on GitHub; #84 closes only when its
   full two-home/device/recovery/identity acceptance criteria are met.
5. Update current product/engineering docs/README/changelog and open a focused PR
   with relevant checks. Preserve all historical native evidence bytes before
   changing any measured input. No physical-hardware/capacity claim.

## Attachments

#88 remains separate: encrypt before upload, private names/keys inside encrypted
mail envelopes, bounded ciphertext storage and explicit saves, no public URL or
server-readable file-area substitution. Exact crypto/transport profile remains
subject to interoperability, trust/recovery and dependency evidence.


## Outcomes and next work

- Merged #89 at `7bfab46` after exact-head green CI and zero review conversations.
  No maintained product runtime/schema/config source was edited in this increment;
  previous native records and source snapshots remain unchanged.
- Current Prosody 13.0.7 builds natively with recorded NetBSD/Lua-5.4 build fixes.
  Two processes under independent UIDs/private stores on one two-core/1-GiB VM use
  fixture TLS, literal peer routing and closed registration/HTTP. This is not
  independent-machine isolation or a supported deployment profile.
- Twelve actual OMEMO-2/SCE mail controls pass, including private subject/body and
  reply, offline queue/reconnect, bounded repeated archive IDs, fresh tampering,
  missing verified key and server-added-device refusal. Actual stores/logs exclude
  the known encrypted private markers. Keys stay in developer-device memory; no
  encrypted keystore/history recovery or native Faction mail UI is claimed.
- Eleven native SSH certificate-verifier controls pass using externally minted
  synthetic certificates. No online issuer or trusted BBS member identity handoff.
  Four native signing cases produce an artifact then exit SIGSEGV; never accept
  failed signer output. Safe reproducer, exact binary identity and #92 retain it.
- The unselected AGPL plugin lacks complete SCE support; the optional Slixmpp
  Rust/PyO3 graph violates the no-JS filename policy. The reference uses inspected
  upstream Python fallback and 21 runtime/six build packages, not product adoption.
  The crypto backend's explicit positive skipped-key parameter also fails; use
  its supported bounded default for this experiment without patching crypto.
- Preserve exact network-run format inputs before independent UTF-8-only parser
  hardening; three host format tests pass. Do not claim the network matrix reran
  after that local format-only change. Current/historical evidence checks protect
  both versions. All fixture services, VM/disk, keys, profiles, logs and private
  dependency environment were stopped/removed after capturing sanitized results.
- #90 owns native Rust mail UI/keystore/trust/recovery and independent interop;
  #91 owns scoped online home proof and local membership binding; #92 native signing.
  #84 stays open. Device-encrypted attachments remain #88, never public/plaintext
  file-area substitution. No new crypto, native production helper, supported
  capacity, physical test or independent security audit is implied.

See [research 0045](../research/0045-encrypted-mail-and-roaming-proof.md),
[ADR 0034](../decisions/0034-encrypted-mail-reuse-proof.md) and
[sanitized results](../evidence/encrypted-mail-netbsd/results.json). Open a focused
PR referencing #84/#88/#90/#91/#92; this request authorizes no merge of that new PR.

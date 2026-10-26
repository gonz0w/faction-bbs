# Certificate-bound UID handoff software evidence

Issue [#223](https://github.com/gonz0w/faction-bbs/issues/223), 2026-10-26.
[The immutable record](results.json) qualifies the bounded experimental handoff:
restricted OpenSSH certificate admission → observed kernel UID/owned TTY → durable
local binding → real core Session. It does not adopt production SSO, an online
issuer or per-device identity. See the [operator/member guide](../../../docs/roaming-authentication.md).

## Observations

| Layer | Observed result |
| --- | --- |
| Default Rust workspace | 430 tests pass on macOS and NetBSD 11 ARM64; host strict all-target release clippy/format pass. |
| Core lifecycle | 15 roaming controls within 203 core tests; 19 marked owner-fixture CLI controls. |
| Native certificate entry | Two independent synthetic home CAs, two separate boards used sequentially, four distinct immutable transport UIDs and four local consents. All four native-client visits reach real approved BBS menus without a password bridge. |
| Native primary proof | 33 controls: issuer/namespace/principal/board/key/UID/validity/status rejection, legitimate certificate reuse, a fresh properly certified leaf, live unlink/member/issuer revocation, classic fallback, restart and restore. |
| Additional native transport | Eight controls include raw-key/password/alternate-command/forwarding denial and active-session shutdown/restart. Two kernel-level controls reject an unregistered peer UID and a registered UID carrying somebody else's TTY. |
| Root manifest | Eight real native filesystem/config controls, including the valid root-owned file and owner/mode/link/ancestor/size/unknown-key rejection. |
| SSH signature component | Seven controls on each OS through real uninstalled OpenSSH sign/verify code; changed session identifier/user/service/request key/verifying key fail. Synthetic identifiers, no captured wire/KEX replay claim. |
| Classic/recovery regressions | 33 host commands; five native legacy commands. Complete bundle recovery passes 21 controls on each OS, using distinct owner/file UIDs on NetBSD. |

The NetBSD build and full workspace tests ran with two cores and **2 GiB**.
The VM then rebooted with **1 GiB** (observed 1,054,990,336 bytes) for real SSH,
manifest, native-client and recovery controls. Four native-client starts took
313–344 ms in this local software fixture, after the unchanged admission delay.
The sampled service owner RSS was 6,528 KiB. These are small-fixture observations,
not whole-installation memory, throughput, WAN latency, physical SBC or capacity
claims. Normal eight sessions/5000-ms admission, two credential permits/kernel
slots, three owner helpers and five-second drain remain unchanged.

## Source and failure boundaries

The record retains a **717-input compiled/native-test context** and a separate
**719-input current-tool context**. Product Rust/native bytes are identical between
them. The latter adds the host evidence checker/tests and updates 37 maintained
smoke-script schema expectations/results from 19 to 20. Those tools are not all
executed on NetBSD; the named legacy commands identify actual execution. Both maps
were independently hashed inside the guest. Frozen content objects resolve the
old and new inputs; historical measurements/record hashes were not refreshed.

Actual RED covers missing interfaces, credential separation, active issuer
revocation, enabled export authority, malformed generations and fresh-UID relink.
The first core RED test bytes and initial checker test were recovered verbatim
from recorded file-write/replacement inputs after expansion/formatting; they are
explicitly identified as recovered inputs. All 22 original compiler positions and
the relevant runtime assertion positions match. Other retained copies keep their
stated collection time. No retrospective test-first execution is claimed.

Seven native failed/superseded setup candidates remain in the record: volatile
runtime-directory omission, a wrong one-room pager expectation, correct rejection
of a non-private export parent, an alternate-command expectation, missing OpenSSH
link objects, an inaccessible build-directory binary after UID drop and the
pre-review relink catalog. The first 27 primary controls passed before the export
parent refusal; a separate recovery continuation completed the remaining six.
The combined result does not pretend that candidate was one uninterrupted pass.

The first evidence assembly omitted a derived source-match summary flag. Its
failed guard run and exact record remain preserved; the actual observed map
already matched. The final assembly retains that observation and computes the flag.

The C oracle uses raw Ed25519 request/signature components from pinned OpenSSH
10.3p1. Its archive identity/build limits are in the [reproduction](../../../experiments/223-roaming-uid/README.md).
Actual certificate admission is a separate native SSH test. Externally minted
five-minute certificates require successful signing and independent verification.
The installed NetBSD certificate signer remains rejected.

## Privacy, recovery and limits

The home-outage control withholds the manual fixture CA's private file: new
issuance fails, another home still issues, and an already-issued unexpired
credential still enters the board. No online provider was deployed or network
partition simulated. The two boards have separate durable stores and policy but
share the fixed owner socket sequentially; there is no concurrent-board claim.

Every binding still needs independent local approval and consent. Restore disables
issuer/binding authority and changes board identity; old manifests fail before
listening. UID history is never freed for reuse. Current session/chat/game/file
checks preserve exact provenance, without revoking unrelated password authority.
Certificate expiry governs admission; it does not end established sessions.
The UID cannot attest a leaf key or independently revoke one device.

Only synthetic data and marked disposable fixtures were used. The VM/controller,
forwarded listeners and temporary fixture roots were removed; the saved owner
playground stayed stopped, with full disk/kernel hashes and metadata unchanged.
The publication payload was staged privately, checked against actual fixture
credential tokens and independently reviewed. Finder/AppleDouble metadata,
generated keys/certificates/profiles and raw native diagnostics are excluded.

Run `python3 tools/check_roaming_evidence.py` or the repository's normal unittest
discovery. The guard checks recorded hashes, source maps, fixed proof/resource
boundaries and teardown; it does not rerun the VM or prove production security.
[Parent #91](https://github.com/gonz0w/faction-bbs/issues/91) retains signer packaging,
online issuance/enrollment and operator adoption choices.

# Synthetic encrypted-mail / roaming controls (#84)

Research tools only: no product dependency, mail UI, keystore, online identity
issuer or supported federation deployment. See [research 0045](../../dev-docs/research/0045-encrypted-mail-and-roaming-proof.md),
[ADR 0034](../../dev-docs/decisions/0034-encrypted-mail-reuse-proof.md) and
[sanitized results](../../dev-docs/evidence/encrypted-mail-netbsd/results.json).

## Fixture and dependencies

Use a **new disposable** NetBSD 11 ARM64 VM, two vCPUs/1 GiB in the recorded run.
Enable root administration only through a private loopback SSH forward with a
serial-verified host key. Create `/etc/faction-disposable-test`. Supply its trusted
SSH argv as a private JSON array; never pass shell text as a profile destination.
The VM is destroyed after the experiment, including every key, account and log.
Nothing here may be pointed at a real board or existing member database.

Install native Lua-5.4 development/runtime libraries (LuaSec, LuaSocket, LuaExpat,
LuaFileSystem, ICU/libidn) and Python 3.13. The recorded pkgsrc 11.0_2026Q2 `prosody`
package provides these prerequisites; its 13.0.3 server is **not started**. Instead:

```sh
python3 experiments/084-mail-proof/build_prosody.py --admin-argv PRIVATE_ADMIN.json --private-log PRIVATE_BUILD.log --disposable-synthetic-fixture
```

This downloads the pinned official 13.0.7 source, screens filenames, applies the
recorded Lua-5.4 path patch and builds with native NetBSD declarations. It copies
source/modules into `/usr/local/faction-mail-probe`, creates no rc service and
starts no listener. The two-line patch is to MIT-licensed upstream `configure`;
source URL/hash and official HTTPS origin are recorded, not an independently
verified Prosody release signature.

Prepare a private local Python 3.13 venv. Install **only** pinned reference packages
from [the observed inventory](dependencies.json), with license/source/JS review
before use. The runtime roots are `OMEMO==2.1.0`, `Twomemo[xml]==2.1.0` and
`slixmpp==1.17.0`; build-only packages and hashes are recorded separately. Do not
install the AGPL Slixmpp-OMEMO plugin or select its legacy fallback.

No macOS Slixmpp wheel was available. Its optional Rust/PyO3 graph contains a JS
documentation asset and is rejected. Build its inspected source archive's upstream
**Python JID fallback** in an environment without a Rust compiler on PATH; optional
Rust compilation must be skipped, not silently linked. For example, after installing
the inspected build-only wheels into the private venv:

```sh
PATH=/usr/bin:/bin /absolute/private/venv/bin/python -m pip wheel --no-build-isolation --no-deps --no-cache-dir --wheel-dir PRIVATE_WHEELS INSPECTED_SLIXMPP_SOURCE
```

Confirm that the produced wheel has no native JID extension, `slixmpp.jid.__file__`
ends in `.py`, and all selected wheel/installed paths pass policy. Then install
runtime wheels offline and run:

```sh
PRIVATE_PYTHON experiments/084-mail-proof/inspect_dependencies.py --wheels PRIVATE_WHEELS --output PRIVATE_INVENTORY.json
```

These are hashes of the tested macOS artifacts, **not** a portable all-platform
installer/lock. Native NetBSD client crypto/library build and complete upstream
source audit remain gates. No Python helper or dependency enters the Rust product.

## Run the two-home control

Fetch and inspect the official `mod_s2sout_override` module separately, matching the
SHA256 pinned by `prepare_homes.py`. Give the tool a private owner-only output
directory. Forward host loopback ports to guest 2222/2223; S2S stays on guest
loopback 5269/5270. The tool requires absent `/home/maila` and `/home/mailb` and
creates only synthetic users/state, a private fixture TLS CA, fixed peer allowlist
and independent account credentials:

```sh
python3 experiments/084-mail-proof/prepare_homes.py --admin-argv PRIVATE_ADMIN.json --output-dir PRIVATE_DIR --s2s-override INSPECTED_OVERRIDE.lua --alice-port HOST_FORWARD_A --bob-port HOST_FORWARD_B --disposable-synthetic-fixture
PRIVATE_PYTHON experiments/084-mail-proof/mail_probe.py --private-fixture PRIVATE_DIR/mail-fixture.json --output PRIVATE_DIR/mail-results.json --disposable-synthetic-fixture
```

The profile contains synthetic credentials and known plaintext markers; keep it
private. The oracle uses actual OMEMO-2/Twomemo public bundles and crypto with a
bounded independent SCE format adapter, explicit trusted-device keys, no fallback
body/plaintext mail, genuine S2S/TLS, offline ciphertext and bounded MAM queries.
The harness compares keys directly on its two devices; this does not implement a
member enrollment UX. Device state is in memory and reused on reconnect, with no
encrypted keystore/recovery. Both ends share one implementation; namespace use is
not proof of independent-client interop or full current-XEP conformance.

Use fresh synthetic accounts or a new fixture for repeat runs: old public device
bundles deliberately remain untrusted when the local in-memory keys are lost.
A newly authenticated device/password reset must not inherit trust automatically.
Capture only result names/counts/source hashes. Inspect actual server stores/logs
for the known private subject/body markers, and retain metadata limitations. The
intentional plaintext-downgrade control may itself be stored in plaintext.

## Native identity controls and signing failure

`signing_probe.py` runs natively, generates only temporary keys, disables core
dumps and records exit codes. Four base NetBSD signing attempts produced files and
then exited on SIGSEGV; **never treat that as successful issuance** (#92).

To isolate the verifier, prepare synthetic short-lived control certificates on the
developer host, then transfer the directory privately into the guest as root-owned
`0700`. The prepare step removes CA private keys after signing; no mail keys are used:

```sh
python3 experiments/084-mail-proof/ssh_cert_probe.py --prepare-fixture PRIVATE_NEW_CERT_DIR --disposable-synthetic-fixture
```

Copy the oracle to the disposable guest and run these **inside NetBSD**:

```sh
python3.13 signing_probe.py --output PRIVATE_SIGNING_RESULT.json --disposable-synthetic-fixture
python3.13 ssh_cert_probe.py --preissued-dir PRIVATE_CERT_DIR --output PRIVATE_CERT_RESULT.json --disposable-synthetic-fixture
```

The test uses a separate root-owned temporary CA/principal file under `/etc`, a
private listener on loopback 2200 and a dedicated temporary UID. Exact per-issuer,
per-board principal grants prevent home A claiming home B. Temporary loopback
aliases isolate negative cases while keeping base SSH source penalties unchanged.
No shell/forwarding access, online certificate issuer, trusted principal-to-BBS
handoff or imported role/approval is implemented. Test accounts/listener/aliases
are removed; destroy the whole VM to remove all fixture and failed-candidate data.

Run `python3 -m unittest discover -s tools -p 'test_mail_envelope.py'` for the
standalone SCE negative controls without third-party crypto or network access.
`tools/check_mail_proof_evidence.py` checks recorded evidence only. #84 stays open;
#90/#91 build the native mail/identity integration and #88 retains encrypted attachments.

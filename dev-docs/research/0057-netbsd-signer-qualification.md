# Maintained NetBSD signer qualification

Date: 2026-10-26. [Issue #226](https://github.com/gonz0w/faction-bbs/issues/226)
compares maintained signing sources for the manual roaming experiment. The native
qualification is separate from production packaging, online issuance and the
operator decisions retained by [#91](https://github.com/gonz0w/faction-bbs/issues/91).
The [earlier diagnosis](0052-netbsd-signing-exit.md) remains immutable.

## Selection and scope

The selected unmodified **portable OpenSSH 10.6p1** source build passed five
native signing controls and their negative checks. The installed NetBSD signer
remains rejected after five crashes. The portable source has a returning
`do_ca_sign()` function without NetBSD's stale `__dead` annotation. These are
bounded component observations. The native harness regression and ordinary-access
preservation checks also passed. No installed package
or online issuer is qualified by the source-build result.

The bounded comparison has three packaging routes. They share OpenSSH's protocol
and much of its implementation; they are not independent cryptographic designs.

| Route and immutable identity | Maintenance and relevant difference | Selection |
| --- | --- | --- |
| NetBSD base: supported `netbsd-11`, signing-source change [5c6b80a](https://github.com/NetBSD/src/commit/5c6b80a579fcae9cf883a80f606d8a8df6e0f54f), 2026-05-07; OpenSSH 10.3. Also inspect development import [f73dc21](https://github.com/NetBSD/src/commit/f73dc21d75578cecd73546d9cfb13dc51bd9f07c), 2026-10-07; OpenSSH 10.6. | Both inspected signing files retain the stale annotation. Base updates follow NetBSD release-branch maintenance and system build/install procedures. A new version banner does not repair the inspected declaration. | Retain the exact failing installed binary as a negative baseline. Do not spend another native build on the same identified defect. |
| pkgsrc `security/openssh`: `pkgsrc-2026Q3` pull-up [a41868e](https://github.com/NetBSD/pkgsrc/commit/a41868e500da86345080d97ed45dac65b2b241a6), 2026-10-12, OpenSSH 10.6p1. Current-tree package change [ae5a7a8](https://github.com/NetBSD/pkgsrc/commit/ae5a7a8218f9f9ed307c05200d830ebe0a939acb), 2026-10-07. | Uses the same portable release archive, with five packaging/portability patches. The stable Makefile adds debugging symbols. None of these patches modifies `ssh-keygen.c` or adds the stale declaration. | Maintained packaging alternative, not built or installed here. A second build would mainly qualify packaging/options, rather than another signer implementation. |
| Official portable OpenSSH 10.6p1: [release](https://www.openssh.org/txt/release-10.6), 2026-10-06; tag `V_10_6_P1`, commit [a8c72b8](https://github.com/openssh/openssh-portable/commit/a8c72b842423f0774f542d3c2197247c3dd28d74). | Current upstream portable release at the research date. Use the release archive's generated configure script, with no local source patches. The source-build operator owns update qualification and local packaging. | Sole selected native candidate; uninstalled, explicit-path, manual signing only. |

[NetBSD's release page](https://www.netbsd.org/releases/formal.html) lists 11.0,
released 2026-07-30, and maintained 10/11 branches. It distinguishes supported
release branches from development snapshots. The
[pkgsrc 2026Q3 announcement](https://www.pkgsrc.org/news/2026Q3/) dates that branch
to 2026-09-23. Its [quarterly policy](https://www.pkgsrc.org/quarterly/) explains
stable-branch fixes and platform-specific package-set availability. The inspected
October pull-up establishes this package's maintenance; it does not establish
availability or correctness of a particular NetBSD ARM64 binary package.

The installed rejected signer remains SHA-256
`b1c2957c9645e7a81522492e82daad827d5d697339ae3991f2902e5edbb85648`.
The old corrected 10.3p1 comparison was an uninstalled diagnostic control, not the
maintained package selected by this report. Neither base source pin above claims
a bit-for-bit reconstruction of that installed executable.

## Source and patch provenance

The selected [official archive](https://cdn.openbsd.org/pub/OpenBSD/OpenSSH/portable/openssh-10.6p1.tar.gz)
is 2,355,244 bytes, SHA-256
`a9dc9565dffe8640f64d863cd29a32bc4a3dbdec0566a7fc44c5d6ee767d5f39`.
Its digest matches the base64-encoded SHA-256 in the release note and the
[stable pkgsrc distinfo](https://github.com/NetBSD/pkgsrc/blob/a41868e500da86345080d97ed45dac65b2b241a6/security/openssh/distinfo)
independently records the same archive's size, BLAKE2s and SHA-512.

Host `gpgv` accepted the [detached signature](https://cdn.openbsd.org/pub/OpenBSD/OpenSSH/portable/openssh-10.6p1.tar.gz.asc)
with normal exit 0 and fingerprint
`7168B983815A5EEF59A4ADFD2A3F414E736060BA`, signature dated 2026-10-06.
The verification key came from the official HTTPS
[release-key file](https://cdn.openbsd.org/pub/OpenBSD/OpenSSH/RELEASE_KEY.asc).
This checks the signature against that published key; independent prior trust in
the key was not established. Temporary public-key import retained a sandbox
agent-connection error; direct `gpgv` verification succeeded without a key agent.
No personal keyring was changed.

The archive contains 932 entries, with no absolute paths, parent traversal,
symbolic/hard links, special files or JavaScript/TypeScript filename extensions.
The selected archive's `LICENCE` SHA-256 is
`5bb5b160726ef5756e4f32fe95b35249c294962419650f48d05134b486d27ccb`.
Its [license text](https://github.com/openssh/openssh-portable/blob/a8c72b842423f0774f542d3c2197247c3dd28d74/LICENCE)
contains permissive component terms, including BSD, ISC, MIT-style and public-domain
material. Preserve the complete notices and source-file terms; Faction's
BSD-3-Clause license does not replace them. pkgsrc labels the package
`modified-bsd`.

The stable pkgsrc [patch directory](https://github.com/NetBSD/pkgsrc/tree/a41868e500da86345080d97ed45dac65b2b241a6/security/openssh/patches)
contains the following files. These are observed package inputs, not patches applied
to the selected portable build.

| Patch | Purpose | SHA-256 at the stable pin |
| --- | --- | --- |
| `patch-Makefile.in` | Use the package askpass path and let pkgsrc handle configuration installation. | `fafe0b12ec21eb0ce730eef360174ac0b145d6822f38e83e9ff1f333b5e0a4a0` |
| `patch-atomicio.c` | Select poll direction using `vwrite`, associated with NetBSD PR 60563. | `87853a3de31c72d8536ff4317224769cb4525363866cfdfc1ba94ce8c1890ae8` |
| `patch-configure.ac` | Let pkgsrc handle runtime search paths; adjust wtmpx probing and askpass reporting. | `9408e770081cfbca96f83d26124f3e522d9a7313f711e96696f18cb4356a5696` |
| `patch-defines.h` | Supply utmpx/wtmpx path definitions. | `96697586c3b2a198e1d8cf4524362affa252f9ff5080e6f798fd41014e565485` |
| `patch-sshkey.h` | Allow an explicitly selected legacy RSA minimum; it is not a recommended option. | `3bfb285de4e8e0ceee90f5707b1f349dd8c1cc2b947e9ec350005a7c093504ae` |

The current-tree and stable patches have identical changes, but two CVS revision
comments differ. Their complete-file hashes must not be interchanged. No source
patch, including the package's optional legacy RSA setting, is adopted here.

## Build and runtime boundary

The selected archive supplies C source, bundled OpenSSH/compatibility archives,
a generated POSIX-shell configure script and make rules. Its
[installation prerequisites](https://github.com/openssh/openssh-portable/blob/a8c72b842423f0774f542d3c2197247c3dd28d74/INSTALL)
and [signer target](https://github.com/openssh/openssh-portable/blob/a8c72b842423f0774f542d3c2197247c3dd28d74/Makefile.in)
require a C compiler, assembler/linker/archive tools, make, shell and ordinary Unix
text tools. The signer target links bundled `libssh`/`libopenbsd-compat` with
configured system libraries. OpenSSL/libcrypto and zlib are optional upstream
features; both were enabled in the observed build described below.

Autoconf regeneration is unnecessary for an unchanged release archive. Package
builds differ: the pinned
[pkgsrc Makefile](https://github.com/NetBSD/pkgsrc/blob/a41868e500da86345080d97ed45dac65b2b241a6/security/openssh/Makefile)
requests autoconf and Perl, regenerates configure, and includes zlib and NetBSD
S/Key support. Its
[options file](https://github.com/NetBSD/pkgsrc/blob/a41868e500da86345080d97ed45dac65b2b241a6/security/openssh/options.mk)
selects OpenSSL and editline, suggests PAM on NetBSD and FIDO where available,
and offers Kerberos. FIDO adds libfido2 and libcbor; autotools adds its m4/Perl
build tooling. These optional package graphs were not installed or qualified.

No JavaScript runtime, package manager or browser component is required by the
selected configure/make/signing path. The upstream archive also contains hosted
CI workflow descriptions; those workflows and their external actions are outside
the selected build and are not imported into Faction. A filename scan alone is
not a transitive dependency audit. Native configuration, linked-library identities
and the actually executed tool path delimit the no-JS qualification.

## Native component observations

The owned disposable fixture ran NetBSD 11 evbarm ARM64 with two virtual cores,
1,054,990,336 bytes of reported physical memory, native GCC 12.5.0 and OpenSSL
3.5.7. This is the approximately 1 GiB software fixture, not a physical SBC or
whole-system capacity claim. The unchanged release archive was built as the
unprivileged fixture user. Its generated configure script received
`--without-pam`, `--without-security-key-builtin`, `--disable-pkcs11` and
`CFLAGS=-O2`; the only make target was `ssh-keygen`, with one build job.
No autoconf regeneration or package installation ran.

Configure and make exited normally in 14,527 ms and 9,581 ms respectively. The
resulting executable is 601,096 bytes, SHA-256
`e9100697a7c62e84d5bfd42b6fbf79ba92dccad63059d57adea8306d08ba02e5`.
The build retained its generated configuration, Makefile and bundled archive
identities. `ENABLE_PKCS11` is undefined. Its Makefile names `-lutil` in `LIBS`
and `-lcrypto -lz` in `CHANNELLIBS`; the actual dynamic dependency closure was
six existing NetBSD libraries: `libutil.so.7`, `libc.so.12`, `libcrypto.so.16`,
`libcrypt.so.1`, `libz.so.1` and `libgcc_s.so.1`. Their exact hashes and sizes are
part of the build identity record. No additional authentication, hardware-key,
JavaScript or online provider was added to this selected native graph.

The final probe identity is SHA-256
`b420838208604fcffb6f18128a36b40f0a3ee6997ce70e76c453ce22ff67f39b`.
It uses the [bounded manual certificate profile](../../experiments/226-signer/README.md):
synthetic Ed25519 device/CA keys, a destination/home/stable-subject principal,
300-second user certificates, ten-second per-command timeouts and a 65,536-byte
combined output ceiling. Core dumps and SSH-agent access are disabled. The
installed base executable independently parses and verifies candidate output;
its known signing failure does not establish a parsing or verification failure.
This is implementation reuse, not a separate cryptographic implementation audit.

| Native control | Installed baseline | Portable 10.6p1 |
| --- | --- | --- |
| Five valid issuances | Five SIGSEGV exits after artifact creation; zero accepted. No failed artifact was parsed or verified. | Five normal exit-0 issuances; all artifacts independently parsed with expected fields and CA/leaf fingerprints. |
| Certificate-backed possession and signature | Not reached after failed issuance. | Five successful standard OpenSSH signature/verification controls. |
| Wrong CA, destination principal, private key and tampered message | Not reached after failed issuance. | Each rejected normally in all five cases; no signal or timeout counts as a denial. |
| Legitimate certificate reuse | Not reached after failed issuance. | A second signature verified for each valid certificate. |
| Expired and future certificates | Both signing attempts crashed; temporal verification was unavailable. | Both parsed windows had the expected direction, verified at an interior verification time and were rejected at actual current time. |
| Missing CA | Normal error, no accepted artifact. | Normal error, no artifact. |
| Synthetic key cleanup | Probe reports removal. | Probe reports removal. |

The portable whole probe took 1,839 ms, with 212 ms child user CPU and 233 ms child
system CPU. The rejected baseline probe took 377 ms; it skipped subsequent
verification after each crash, so that number cannot compare usable signing
performance. The native `/usr/bin/time -l` observation was 21,376 KiB maximum
resident size for the Python harness and its child accounting. NetBSD documents
that command/child scope in [time(1)](https://man.netbsd.org/time.1) and kilobyte
units in [getrusage(2)](https://man.netbsd.org/getrusage.2). It is neither
isolated signer RSS nor simultaneous aggregate RSS, and says nothing about BBS
memory or concurrent throughput. Compiler/configuration cost is separate from
this runtime observation.

The component probe runs no SSH listener and does not exercise the complete BBS
handoff. All 16 native harness tests passed, using the qualified portable tool for unit
fixtures while the actual qualification matrix retained the separate base verifier.
Synthetic saved-image CLI login/logout and administrative SSH passed before and
after; a post-build administrative TTY and sshd validation passed too. All selected
base and CLI binary hashes stayed unchanged. No current product
Rust runtime was rebuilt or newly qualified here. The prior UID-to-session proof
retains its own scope and immutable evidence.

## Update, recovery and adoption

The portable release is maintained upstream; this uninstalled build is not a
maintained NetBSD binary package. A later operator adoption must own its explicit
installation location, source/patch/configuration manifest, library dependencies,
permission checks and update schedule. Review upstream release/security changes,
verify the new archive and signature, then repeat the same positive and negative
native controls before changing the selected executable. Never update a measured
hash to describe new bytes or accept an artifact left by a failed process.

For this manual source-build route, keep a separately identified last-qualified
build and its complete dependency/provenance record. On update failure, stop fresh
issuance, retain the rejected evidence and deliberately restore only a previously
qualified and still acceptable build. The known failing base signer is not a
fallback. A rollback cannot undo issued credentials: use the destination's current
CA/principal/revocation policy for new admission and binding/issuer/member controls
for active BBS sessions. Suspected CA compromise requires stopping issuance and
explicit trust/key recovery, not merely replacing the executable.

The alternative pkgsrc route would use a consistent supported package branch,
package vulnerability checks and an explicitly preserved prior package/dependency
set. Its automatic package lifecycle includes files beyond `ssh-keygen`, accounts
and service configuration. Any later packaging adoption therefore needs its own
install/update/removal and ordinary-access recovery proof. Base recovery instead
belongs to a coherent NetBSD system update/rebuild. Neither route is authorized
or validated by source-build success.

The [roaming handoff contract](0056-roaming-identity-handoff.md) and
[operator guide](../../docs/roaming-authentication.md) remain binding. A qualified
manual signer does not settle per-UID account cost, per-device attribution,
admission-only expiry, issuer operation, enrollment/consent, CA custody or
production approval. Member authentication keys remain distinct from device-private
mail keys. No online provider, host-policy change or production issuance follows
from this research.

## Harness review and preserved inputs

Actual test-first changes cover missing interfaces, Python3.10-compatible hashing
and complete four-keypair setup. Independent review then showed that a rejected
expired certificate could masquerade as the future control. Both swapped-window
regressions ran RED; explicit UTC direction around issuance now rejects them while
retaining midpoint verification and current-time rejection. Sixteen host/native
tests pass with their assertions intact. Every executed source variant and failed
fixture launch is retained; no reconstructed test input is used.

The first native build wrapper had a nested-string syntax error before configure
ran. It is a preserved setup failure, separate from baseline signer crashes. The
corrected standalone wrapper built the same unmodified archive successfully.

[Source-bound host/native evidence and cleanup](../evidence/signer-qualification/README.md) retain the actual observations, failed setup and executed source variants. All disposable roots and forwarded listeners were removed; the saved owner playground remained stopped and unchanged.

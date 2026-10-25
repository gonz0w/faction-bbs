# NetBSD certificate signing: returning function marked noreturn

Date: 2026-10-24. Scope: [issue #92](https://github.com/gonz0w/faction-bbs/issues/92),
software research on NetBSD 11 ARM64, two cores/1 GiB. No tool was installed or
replaced. [Sanitized evidence](../evidence/signing-exit-netbsd/results.json) and
[reproduction](../../experiments/092-signing-exit/README.md) retain exact identities.

## Finding and decision

A stale NetBSD `__dead` annotation is a demonstrated defect in the tested signing
translation unit. Upstream changed `do_ca_sign()` from exiting the process to
returning to its caller, while NetBSD retained the noreturn annotation. The
annotated NetBSD source crashed in all five controls. Removing only `__dead`,
with the same compiler, configuration and libraries, produced five successful
exits with verified certificate-backed signatures and rejected negative controls.

The stock binary's matching failures and the source history strongly support this
explanation for the recorded fixture crash. This is not a bit-for-bit rebuild or
symbolized diagnosis of the installed base executable. Its private library has a
different build/namespace interface; direct relinking was unsuccessful. Preserve
that limit rather than attributing every NetBSD crash to this finding.

Keep the installed failing signer rejected. The tested correction is an
**uninstalled research candidate**, not an online issuer or supported package.
[#91](https://github.com/gonz0w/faction-bbs/issues/91) owns any later reviewed
issuer selection, packaging, enrollment and approval binding. Ordinary SSH/SFTP
and the Rust BBS are unchanged. No external report was sent or authorized.

## Controlled observations

The original frozen four-case probe reproduced its exact base executable hash
`b1c2957c9645e7a81522492e82daad827d5d697339ae3991f2902e5edbb85648`.
The new probe adds quiet signing and strict success/negative verification:

| Executable/source | Five signing outcomes |
| --- | --- |
| Installed NetBSD base | Five SIGSEGV exits after producing artifacts; none accepted. |
| Portable OpenSSH 10.3p1, ordinary returning function | Five zero exits; all certificate/signature checks pass. |
| Same portable source plus noreturn annotation | SIGSEGV and bounded timeout failures; none accepted. |
| Pinned NetBSD translation unit with `__dead` | Five SIGSEGV exits; none accepted. |
| Same NetBSD translation unit with only `__dead` removed | Five zero exits; all certificate/signature checks pass. |

All variants ran as the same unprivileged fixture user. A/B library hashes match.
The probe disables core dumps, removes SSH agent access, generates private temporary
keys and deletes them on success/error. A nonzero exit, signal, timeout, missing
artifact or failed verification never becomes success. No failed signer artifact
is verified or accepted. Quiet mode excludes success-message formatting as a
necessary trigger. Noreturn undefined behavior can hang as well as crash; do not
require one deterministic signal from the injected portable experiment.

For successful issuance, base `ssh-keygen -Y sign -f device-cert.pub` loads the
matching synthetic private key and embeds its certificate. Base `-Y verify`
checks it against an explicit synthetic CA/namespace/principal allowlist.
Tampered messages, wrong principals and wrong CAs must exit normally with an error;
a crashed verifier is not a valid rejection. Missing CA signing must also fail
normally without producing an artifact. No additional daemon/listener is needed.

## Sources and build limits

Primary sources checked 2026-10-24:

- [Upstream return/cleanup change 846987d](https://github.com/openssh/openssh-portable/commit/846987d1233f24bbe87ebed347e328f45525388a)
  removed the process exit from the signing function.
- [NetBSD import ccc3699](https://github.com/NetBSD/src/commit/ccc3699084cd26c65ef48b53821e9a3ac3ebd3c7)
  retained `__dead`; [pinned 10.3 source 4d24211](https://github.com/NetBSD/src/blob/4d24211097fc31930b5547e6e454385370758b67/crypto/external/bsd/openssh/dist/ssh-keygen.c)
  contains the tested declaration and returning body.
- [NetBSD compiler attributes](https://github.com/NetBSD/src/blob/4d24211097fc31930b5547e6e454385370758b67/sys/sys/cdefs.h)
  define the annotation's noreturn meaning. The installed header hash is recorded.
- [OpenSSH 10.3 release notes](https://www.openssh.org/txt/release-10.3)
  publish the archive checksum. Official HTTPS plus that SHA-256 was checked;
  independent release-signature validation was not performed.
- [Upstream PKCS11 queue fix](https://github.com/openssh/openssh-portable/commit/0118c30acaff308deb089fc25fe98ef59a149ca5)
  addresses a different exit crash and is already present in the inspected NetBSD
  source. Its release-note description alone does not establish this failure's cause.

The comparison uses native GCC 12.5.0 and OpenSSL 3.5.7, with `-O2 -g` and the
portable configure flags recorded in the experiment. This is not a base-system
rebuild: the comparison uses the portable SSH library instead of base private
libssh. System-library and A/B identities are recorded. The comparison binaries are about 1.9 MB with debug
information versus the stripped 160,280-byte base tool; this is not a product
memory comparison. Matrix durations include host SSH orchestration and retain
10-second timeouts. No physical hardware, capacity, production issuer or broad
NetBSD-architecture support claim is made.

# Native manual signer qualification

[Issue #226](https://github.com/gonz0w/faction-bbs/issues/226), 2026-10-26.
[The immutable record](results.json) qualifies one unpatched portable OpenSSH 10.6p1
source build for the bounded manual Ed25519 certificate profile. It does not
install a package, adopt an online issuer or qualify production roaming. See the
[comparison and update/recovery reasoning](../../research/0057-netbsd-signer-qualification.md).

## Observed results

| Subject | Result |
| --- | --- |
| Installed NetBSD baseline | Rejected: five CA-signing crashes, zero accepted issuances. Artifacts left by crashes never reach parsing or verification. |
| Uninstalled portable OpenSSH 10.6p1 candidate | Five normal successful issuances, independently parsed CA/leaf/serial/identity/principal/300-second profile, possession and signature proofs. |
| Negative controls | Wrong CA, principal, private key and tampered message fail normally. Certificate reuse succeeds. Expired and future certificates have correct UTC direction, verify inside their own window and fail now. Missing CA fails without an artifact. |
| Harness tests | 16 pass on the host and 16 on NetBSD. Native unit fixtures explicitly use the qualified portable reference; the actual native matrix uses the unchanged base verifier. |
| Ordinary access | Synthetic saved-image CLI login/logout and pinned administrative SSH pass before/after; post-build administrative TTY and sshd checks pass. Selected base/CLI executable hashes remain unchanged. This is not a new current-member-SSH trial. |
| Source identity | All893 regular extracted upstream files match the verified release archive after build. Five selected probe/compatibility inputs match their frozen host bytes inside the guest. |

The VM ran NetBSD 11 ARM64 with two cores and 1 GiB throughout compilation and
execution (observed 1,054,990,336 bytes). Configure took 14.527 s and make took 9.581 s.
The uninstalled candidate is 601,096 bytes, SHA-256
`e9100697a7c62e84d5bfd42b6fbf79ba92dccad63059d57adea8306d08ba02e5`.
It links the recorded base libutil/libc/libcrypto/libcrypt/libz/libgcc_s libraries;
the generated build configuration/flags and library identities are retained.
No Rust product/runtime or database format changed.

The complete native qualification command took 1.839 s, including Python, subprocess
and independent-verifier overhead. Its child CPU accounting was 212 ms user / 233 ms
system. Native `/usr/bin/time -l` reported 21,376 KiB maximum resident size for command/child
accounting. This is not isolated signer RSS, concurrent aggregate memory, BBS
memory or supported capacity. The crashing baseline's 377 ms is not usable signing
performance. No physical SBC, whole-system load or 250-member result is claimed.

## Provenance and failed candidates

The official release archive SHA-256 is
`a9dc9565dffe8640f64d863cd29a32bc4a3dbdec0566a7fc44c5d6ee767d5f39`.
Published release and stable pkgsrc digests agree; detached-signature verification
succeeds against the official HTTPS-fetched release key. Independent prior trust
in that key was not established. The record identifies the shortlist and the
unexecuted pkgsrc route. No upstream hosted CI or JavaScript tooling was imported
or run. Primary-source copies retain their upstream copyright/license notices;
Faction's BSD-3-Clause terms do not replace those component terms.

All executed prototype versions and original RED logs are retained. Actual RED
covered the missing probe, Python 3.10 hashing compatibility and an incomplete
fourth-keypair setup. Independent review then reproduced a swapped temporal window
that incorrectly qualified; both directions ran RED before the UTC checks fixed it.
The final probe is `b420838208604fcffb6f18128a36b40f0a3ee6997ce70e76c453ce22ff67f39b`.
A reviewer withdrew a hash sampled after concurrent source edits; the final review
compiled immutable byte buffers and checked hashes before/after execution.
No later hash is used to relabel that earlier observation.

The first native build wrapper failed to parse a nested string before configure
ran. Its source/diagnostic remains a setup failure; the corrected standalone
wrapper compiled the same untouched archive. Public-key import also retained an
agent-connection error before direct gpgv verification succeeded. Neither failure
is hidden or counted as a successful signing operation.

## Cleanup and limits

Only synthetic keys were used; the probes remove their temporary key/signature
folders on success and error. No generated keys, certificates, profiles, source
archive, keyring, raw signer output or Finder metadata is published. The exact
allowlisted payload was privately staged, hash-checked and independently reviewed
before repository publication.

The disposable VM/controller, forwarded sockets, temporary key-agent context and
all four private fixture/source roots are removed. Full saved playground disk and
kernel hashes and its metadata remain unchanged; it stayed stopped throughout.
Existing #92 and #223 records and their probes were not rewritten.

Run `python3 tools/check_signer_qualification_evidence.py` or normal unittest
discovery. The guard verifies recorded source bytes and every success/rejection
contract, compatibility and teardown; it does not rerun the VM or prove production
security. Package installation, CA custody/enrollment, online issuance and operator
adoption remain with [parent #91](https://github.com/gonz0w/faction-bbs/issues/91).

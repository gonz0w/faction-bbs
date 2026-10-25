# Disposable certificate signer comparison

Research-only controls for [#92](https://github.com/gonz0w/faction-bbs/issues/92).
See [the finding](../../dev-docs/research/0052-netbsd-signing-exit.md) and
[exact results](../../dev-docs/evidence/signing-exit-netbsd/results.json).
Never point the procedure at real keys or replace `/usr/bin/ssh-keygen`.

Use a new marked disposable NetBSD 11 ARM64 VM, private verified administration,
two cores/1 GiB, the existing GCC 12.5.0/base OpenSSL headers and an unprivileged
build user. Keep core dumps disabled. Download the official portable source:

```sh
umask 077
ulimit -c 0
mkdir -m 700 signing92
cd signing92
ftp -o openssh-10.3p1.tar.gz https://cdn.openbsd.org/pub/OpenBSD/OpenSSH/portable/openssh-10.3p1.tar.gz
test "$(sha256 -q openssh-10.3p1.tar.gz)" = 56682a36bb92dcf4b4f016fd8ec8e74059b79a8de25c15d670d731e7d18e45f4
```

Inspect archive paths/license/filenames before extracting: no absolute/traversal
paths, links or prohibited JS/TS files were found in the tested archive. No
independent PGP release-signature check is claimed. Preserve upstream copyright
and license notices. Build only the uninstalled comparison executable:

```sh
tar -xzf openssh-10.3p1.tar.gz
cd openssh-10.3p1
./configure --without-pam --without-security-key-builtin CFLAGS="-O2 -g"
make -j2 ssh-keygen
cp ssh-keygen ../ssh-keygen-portable-returning
```

Fetch `ssh-keygen.c` from NetBSD source commit
`4d24211097fc31930b5547e6e454385370758b67` at
`crypto/external/bsd/openssh/dist/ssh-keygen.c`. Its SHA-256 must be
`bff1b1a05c240359fe5c65ecbc16d26c5dc3d2e216eae07d73e6e433aeefb213`.
Keep an original copy, replace only the comparison tree's translation unit, rebuild
`make ssh-keygen`, and copy that binary under a distinct annotated name.
Apply [the one-line patch](netbsd-returning.patch) to this source, rebuild with the
same flags/libraries, and retain the corrected binary under another name. The
patch changes only the stale annotation; no signing, verification or cleanup code
is removed. It is not an installer or an approved base-system patch operation.

The additional portable mechanism control adds only
`__attribute__((__noreturn__))` to its original `do_ca_sign` declaration and rebuilds
with the same flags/libraries. It can crash or hang; keep the timeout and never
accept the resulting file. This control is not a rebuilt NetBSD base binary.

Copy the current repository probe into the private fixture and run each trusted
binary as the same unprivileged build user. Use absolute paths for the binary and
report, for example:

```sh
python3.13 tools/probe_signing_exit.py --disposable-synthetic-fixture --keygen /absolute/private/ssh-keygen-netbsd-returning --verifier /usr/bin/ssh-keygen --output /absolute/private/returning.json
```

Run the installed base tool and each comparison candidate separately. The probe
creates its own synthetic CA/device keys, checks five cases plus missing CA, and
uses certificate-backed SSHSIG verification for zero-exit outputs. It exports only
outcomes/hashes, never keys/certificates/raw stderr. Exit-zero from the **probe**
means a matrix was recorded; inspect its `result` and each `accepted` field. An
expected failing candidate is an observation, not a successful signing operation.

Capture OS/compiler/source/patch/executable and linked-library identities. Compare
A/B library hashes. Record actual failures and timeouts. Verify the base executable
hash is unchanged, export sanitized observations, then destroy only the owned VM
and private build/key/log files. Confirm forwarded ports closed and the saved
owner playground stayed stopped/intact. Do not run cleanup before evidence export
has succeeded. Issuer selection/installation is separate work under #91.

Host regression tests require Python stdlib and an existing OpenSSH `ssh-keygen`:

```sh
python3 -m unittest discover -s tools -p test_signing_exit.py
```

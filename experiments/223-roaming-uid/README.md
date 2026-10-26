# SSH authentication signature component oracle

[Issue #223](https://github.com/gonz0w/faction-bbs/issues/223) tests a trusted
UID-to-BBS-session handoff. This small C test qualifies only the public-key
signature component of RFC4252 section 7. It uses OpenSSH's real `sshkey_sign`,
`sshkey_verify` and `sshbuf` implementations. It introduces no product dependency,
cryptographic primitive, listener or issuer.

The request constructor follows the ordinary publickey path in
[OpenSSH 10.3p1 auth2-pubkey.c](https://github.com/openssh/openssh-portable/blob/V_10_3_P1/auth2-pubkey.c).
It uses synthetic session identifiers, not a live KEX transcript. Seven controls
accept the original and identical reconstructed request, then reject a changed
session identifier, username, service, request key and verifying key. A negative
must return `SSH_ERR_SIGNATURE_INVALID`; a crash or arbitrary error cannot pass.
This is not captured wire replay through sshd. Real SSH certificate admission is
verified separately. Certificates are legitimately reusable during their validity.

## Reproduce in an owned disposable build

Use a C compiler and OpenSSL/zlib development libraries. Download the official
[portable archive](https://cdn.openbsd.org/pub/OpenBSD/OpenSSH/portable/openssh-10.3p1.tar.gz)
into your own temporary directory. Its SHA-256, published in the
[release notes](https://www.openssh.org/txt/release-10.3), is
`56682a36bb92dcf4b4f016fd8ec8e74059b79a8de25c15d670d731e7d18e45f4`.
Require the exact match before extraction. The test does not install OpenSSH.

From the extracted source:

```sh
./configure --without-pam --without-security-key-builtin CFLAGS=-O2
make -j1 libssh.a ssh-pkcs11.o ssh-sk-client.o
make -C openbsd-compat -j1
cc -Wall -Wextra -Werror -I . -I openbsd-compat   /absolute/faction-bbs/experiments/223-roaming-uid/replay-test.c   ssh-pkcs11.o ssh-sk-client.o libssh.a openbsd-compat/libopenbsd-compat.a   -lcrypto -lz -lutil -o /absolute/owned-temp/replay-oracle
/absolute/owned-temp/replay-oracle
```

On macOS the observed build additionally used `--with-ssl-dir` and explicit
OpenSSL include/library paths, and linked `-lresolv` instead of `-lutil`.
Successful exit emits a counts-only JSON result identifying this component layer.
Key material exists only in process memory. No generated signature or private key
is printed or retained. The native fixture builds this oracle as an unprivileged
user; it does not replace or qualify the rejected installed NetBSD signer.

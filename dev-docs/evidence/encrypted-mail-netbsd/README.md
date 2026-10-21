# Encrypted-mail and scoped-identity reference evidence

Recorded 2026-10-21; [sanitized results](results.json),
[research/limits](../../research/0045-encrypted-mail-and-roaming-proof.md),
[reproduction](../../../experiments/084-mail-proof/README.md).

Two separately owned Prosody 13.0.7 processes on one NetBSD 11 ARM64 software VM,
and macOS member-side reference clients, pass twelve OMEMO-2/SCE mail controls.
Eleven native SSH verifier controls use externally minted certificates. Four
native signing cases retain a SIGSEGV observation rather than a false success.
Known private markers were absent from actual home stores/logs. No real members,
mail, public network peers, private keys, raw transcripts or core dumps are included.

A same-implementation internal protocol proof is not a supported mail product,
independent-client interoperability, encrypted keystore/recovery, online issuer,
trusted BBS membership binding, security audit or hardware/capacity measurement.
All maintained product source and previous historical evidence stay unchanged.
The CI checker validates source/result identity and declared limits, not a live VM.

The exact network-run SCE/test bytes are preserved in `proof-inputs/`. Subsequent
UTF-8-only parser hardening is separately hashed and host-tested (three tests);
the record explicitly does not relabel the earlier network run as a rerun of it.

# Certificate signing exit comparison

[results.json](results.json) is the sanitized 2026-10-24 software observation for
[issue #92](https://github.com/gonz0w/faction-bbs/issues/92). A `pass` report means
the diagnostic comparison met its acceptance, including rejected candidates; it
does not mean the installed base signer passed. It still exits on SIGSEGV.

The [research](../../research/0052-netbsd-signing-exit.md) separates the source
finding, measured A/B outcomes and inference about the base executable. The
[experiment](../../../experiments/092-signing-exit/README.md) identifies source
archives, exact patch, compiler/configuration, commands and cleanup. Current and
original probe hashes are recorded; the original #93 proof and its four failing
controls remain unchanged.

The 25-case matrix includes installed base, portable returning/annotated and
NetBSD-source annotated/corrected executables. Both returning variants passed all
five certificate-backed signature and negative-verification controls. Bad annotated
candidates failed or timed out and were never accepted. The direct base-private-
library relink did not succeed; no exact base rebuild is claimed. Costs record
binary sizes/library hashes and host-observed matrix durations, not BBS RAM or
supported capacity. Core dumps were disabled and private synthetic keys removed.

Teardown fields must be true before completion: evidence export precedes removal
of the owned VM/private inputs, and the saved owner playground remains stopped
and intact. No base tool, service, BBS runtime, issuer, public peer or member
credential was changed; no external bug report was sent.

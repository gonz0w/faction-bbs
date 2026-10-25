# Native source provenance

The [recorded #203 choice](https://github.com/gonz0w/faction-bbs/issues/203#issuecomment-6396919934)
selects a default-off experimental member-only library. This directory vendors
six required generated-C/HACL/OpenSSL include files from Picomemo 1.3.1 revision
`835d8277108b96c691fb6a992512f37315cb52d7`, with the qualified `ad-order-v1`
correction. The original interoperability experiment and evidence remain immutable.

Acquisition: [exact upstream archive](https://codeload.github.com/mierenhoop/picomemo/tar.gz/835d8277108b96c691fb6a992512f37315cb52d7),
SHA-256 `7b1519bb1edfd6a86d6595f6357270b6999a0393cafdb63d76257beb95647301`.
Apply the [preserved correction](../../experiments/200-omemo-interop/picomemo-ad-order.patch),
SHA-256 `8bc5d7560065865c81947099c7dabd401cdb21de3c3002d4466e3ce77f395465`,
to a separate extracted tree, using `patch -p1`. Copy only `gen/omemo2.c`,
`gen/omemo2.h`, `hacl.c`, `openssl.c`, `driver.h`, and `omemo.h` after comparing
all six against [the corrected qualification pins](../../dev-docs/evidence/omemo-interop/results.json).
[SHA256SUMS](vendor/SHA256SUMS) records each vendored file. `build.rs` independently
contains the same six accepted hashes and rejects changed input before compilation.

The correction changes the three OMEMO-2 source/header files to retain the active
initiation role for associated-data order and require canonical role persistence.
The other three files remain original. These are the exact corrected bytes;
modification attribution lives here so qualification hashes remain unchanged.
No primitive, KDF or ratchet is implemented in Faction Rust or shim code.

The HACL amalgamation is the existing upstream file. Picomemo's pinned Makefile
names HACL revision `504c2987452f87fe44bce9b9f12e19d6e051761f`; its [full Apache
license](https://github.com/hacl-star/hacl-star/blob/504c2987452f87fe44bce9b9f12e19d6e051761f/LICENSE)
is copied as `vendor/LICENSE-HACL-APACHE` (SHA-256
`c5accbbd8546e94c34aed24afe689a617627d18eed5a6c48277e48db57c23851`).
`vendor/LICENSE-HACL-MIT` reproduces the full MIT notice already embedded in
`hacl.c`. Every embedded original copyright/license notice remains unchanged.
`vendor/LICENSE-ISC` is Picomemo's actual root license; generated and backend files
also retain their original 2026 notices. These terms coexist with Faction's BSD
license; no relicensing or audit claim follows.

There is no source tree override, generation, network acquisition or profile
fallback during builds. The selected native source graph has only C/headers;
there are no JS/TS assets or dependencies. System OpenSSL is dynamically linked
and retains its own distributor terms; its sources are not vendored here.

SCE parsing uses exact `roxmltree` 0.21.1, default features disabled, only `std`.
The crate archive SHA-256 is
`f1964b10c76125c36f8afe190065a4bf9a87bf324842c05701330bba9f1cacbb`.
Its inspected manifest has no build script, only the existing Rust `memchr`
dependency; the selected source archives contain no JS/TS. `roxmltree` uses
MIT OR Apache-2.0 and `memchr` uses Unlicense OR MIT; their license texts are
provided by the standard Cargo source distributions. Getrandom/zeroize use the
workspace's existing pinned versions. The independent Python reference and its
AGPL dependencies remain isolated research inputs, outside the product graph.

Updates need a finite GitHub issue, source/license/no-JS review, revised archive,
patch and per-file provenance, preserved prior executed bytes, and reviewed host,
native NetBSD and unchanged independent reciprocal-interoperability results.
Never refresh an old evidence pin or silently replace this correction with an
upstream release. A change in experimental/support status is a separate decision.

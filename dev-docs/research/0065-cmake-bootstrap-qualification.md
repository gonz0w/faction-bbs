# 0065: CMake bootstrap qualification

**NO-GO for the selected unmodified CMake 3.31.8 minimal native NetBSD profile:** ordinary configuration would write JavaScript/QtScript into the build tree. The source control flow and actual template content demonstrate the forbidden edge. Configuration was not run, so no generated output or script execution was observed.

[Issue #331](https://github.com/gonz0w/faction-bbs/issues/331) examined the required build-tool gap in the [corrected headless reference study](0064-corrected-headless-source-profile.md). Its explicit forbidden-edge exit permits this finite rejection while leaving complete dependency and license qualification unknown. This result neither qualifies the proposed QtBase correction nor changes the selected isolated QXmpp 1.17.0 + QtBase 6.12.0 reference. The normal QtBase profile's [0062 NO-GO](0062-headless-reference-qualification.md) and all historical observations remain unchanged. [#295](https://github.com/gonz0w/faction-bbs/issues/295) and [#88](https://github.com/gonz0w/faction-bbs/issues/88) remain incomplete.

## Candidate and provenance

Source inspection: 2026-11-01, America/Denver; Faction baseline `2300e9dbe106e110f251c9e43512492a3c4046e3`.

| Input | Primary source | Bytes | SHA-256 |
| --- | --- | ---: | --- |
| CMake 3.31.8 source | [Official archive](https://cmake.org/files/v3.31/cmake-3.31.8.tar.gz) | 11713806 | `e3cde3ca83dc2d3212105326b8f1b565116be808394384007e7ef1c253af6caa` |
| Upstream checksum list | [Official SHA-256 list](https://cmake.org/files/v3.31/cmake-3.31.8-SHA-256.txt) | 1646 | `8d08ce0586bf40ff6b80e46fffe2db8855bab5b030db8b3782f11f68b6274ad8` |

The acquired source archive matches the upstream checksum entry. Its [embedded version](https://github.com/Kitware/CMake/blob/dbe9d4593f4b7a2b1eb87826949f09af0268240a/Source/CMakeVersion.cmake#L1-L24) identifies 3.31.8 and abbreviated source identity `dbe9d4593f CMake 3.31.8`; the abbreviation alone does not establish a full commit. A separate read-only query of the official Git repository observed tag object `a7c63f57a5ac6b0c847c0a9f861c101f4c29f146` and peeled commit `dbe9d4593f4b7a2b1eb87826949f09af0268240a` for `refs/tags/v3.31.8`. That commit matches the embedded abbreviation; no signature verification was performed. The source links use this separately observed commit. The archive digest above pins the examined bytes, and the witness table below identifies all 13 inspected archive bodies. CMake 3.31.8 exceeds QtBase's supported floor of 3.25 recorded in 0064. This exact research pin is not a latest-version or security recommendation.

The archive's 28,715 member names and types were inventoried, with no links or unsafe paths/types found. Two `.js` filename candidates occur under tests; `.qs` and `.qs.in` names also occur. Those observations alone neither qualify nor reject a selected graph. Only the actual content and reachable generation below establish this rejection. No upstream member was extracted or materialized; source bodies were inspected in memory.

## Rejected inert profile

The initial exploration considered host and NetBSD builds. The recorded replan selected one future native NetBSD 11 ARM64 build and runtime path. macOS ARM64 served only as inert inspection tooling; no host-built Qt tool, cross-compiler or macOS SDK was selected into the reference graph. This table specifies the rejected proposal, not a runnable recipe or qualified file allowlist.

| Component | Proposed selection and demonstrated limit |
| --- | --- |
| Build route | Unmodified source bootstrap with `Unix Makefiles`; native `/bin/sh`, Unix utilities, C/C++ compiler, make, assembler, archiver, linker, compiler resources and loader/runtime are required graph nodes. Exact selected tool implementations and closure remain unqualified. |
| CMake libraries | `--no-system-libs` selects CMake-provided third-party libraries. Their source bodies, transitive dependencies and individual terms were not fully qualified. System compiler/runtime dependencies remain required. |
| Optional interfaces | `--no-qt-gui` and `--no-debugger`; the bootstrap recognizes both flags. Proposed `BUILD_CursesDialog=OFF` bypasses the curses probe/discovery in the inspected build-utilities file and the curses subdirectory in `Source/CMakeLists.txt`. |
| Tests and documentation | Proposed `BUILD_TESTING=OFF`; no Sphinx generation option enabled, and all Sphinx output disabled as a profile requirement. Documentation sources, tests and examples were proposed exclusions only where actual reachability and legal obligations permit; no complete exclusion boundary was proved. |
| Packaging | No package target or installer invocation selected. Ordinary CMake configuration still reaches the QtIFW template loop below. |
| Tools, modules and installation | Retain required CMake executables, reachable modules, utilities and legal material. Complete generated, copied, downloaded and installed content remains unknown. No claim that every internal generator or download is excluded. |
| Reference libraries | Retain the previously proposed corrected QtBase Core/Network/Xml and QXmpp encrypted OMEMO reference, with required tools, OpenSSL and libomemo-c. Their remaining sources, licenses and transitive graph are unqualified. Disabling encryption would not qualify that reference. |

The inspected [bootstrap](https://github.com/Kitware/CMake/blob/dbe9d4593f4b7a2b1eb87826949f09af0268240a/bootstrap#L686-L748) declares these bootstrap options and the Unix Makefiles generator. The [build-utilities file](https://github.com/Kitware/CMake/blob/dbe9d4593f4b7a2b1eb87826949f09af0268240a/Source/Modules/CMakeBuildUtilities.cmake#L345-L379) supplies the curses controls. These local controls do not establish the profile's global no-JS requirement. Proposed source-directory exclusions were never applied. `CMake_TEST_EXTERNAL_CMAKE` selects a different external-CMake test mode and does not qualify this source bootstrap. No guard injection, missing-template bypass or corrected CMake source was selected.

## Demonstrated forbidden generation

The ordinary native build follows this source chain:

1. The [root build file](https://github.com/Kitware/CMake/blob/dbe9d4593f4b7a2b1eb87826949f09af0268240a/CMakeLists.txt#L397-L477) enters ordinary `NOT CMake_TEST_EXTERNAL_CMAKE` branches at lines 397 and 444. The latter includes `CMakeCPack.cmake` at line 475, outside the `BUILD_QtDialog` condition. NetBSD is not excluded.
2. [CMakeCPack.cmake](https://github.com/Kitware/CMake/blob/dbe9d4593f4b7a2b1eb87826949f09af0268240a/CMakeCPack.cmake#L58-L59) includes `Modules/CPackIFWConfigureFile.cmake` at line 59. Its [unconditional loop at lines 202–208](https://github.com/Kitware/CMake/blob/dbe9d4593f4b7a2b1eb87826949f09af0268240a/CMakeCPack.cmake#L202-L208) processes `CMake.Dialogs.QtGUI`, `CMake.Documentation.SphinxHTML` and `CMake.DeveloperReference.HTML`, reading each `Source/QtIFW/<name>.qs.in` and specifying `<build-directory>/<name>.qs` as output.
3. The [macro at lines 41–65](https://github.com/Kitware/CMake/blob/dbe9d4593f4b7a2b1eb87826949f09af0268240a/Modules/CPackIFWConfigureFile.cmake#L38-L68) performs `file(READ)`, placeholder substitution, then `file(WRITE)`. Its surrounding condition is the ordinary include guard, not a GUI, docs, tests, packaging or platform selection.
4. The inspected [QtGUI template](https://github.com/Kitware/CMake/blob/dbe9d4593f4b7a2b1eb87826949f09af0268240a/Source/QtIFW/CMake.Dialogs.QtGUI.qs.in#L1-L21) contains a function declaration, prototype function assignment, strict equality and installer/component calls. This is actual JavaScript/QtScript, despite the `.qs.in` suffix. The Windows condition is inside the template content; it cannot prevent CMake from reading and writing that content on NetBSD.

The template is 483 bytes, SHA-256 `b3ae6cc7950b393aa2d39765e5edab15edbb05a88135a86d25d7002fcf16846c`. Disabling the GUI, Sphinx docs and tests, or avoiding package execution, does not guard this loop. Source therefore establishes a mandatory forbidden write on this configuration path if reached successfully. It does not establish that configuration succeeds or that any resulting script executes. One classified template is sufficient for this positive rejection; the other two templates were not content-classified.

## Inspected graph prefix and unknown remainder

The first checkpoint read four CMake bodies: `bootstrap`, root `CMakeLists.txt`, `Copyright.txt` and `Source/CMakeVersion.cmake`. It stopped at the required system-tool boundary: the shell and Unix utilities precede compiler and make discovery. Host `cc`, `c++`, `make`, `ar` and `ld` paths shared identical dispatch-stub bytes, which do not identify their actual implementations or SDK closure. Their hashes were observations, not a qualified host build.

The native-only replan acquired official NetBSD 11.0 evbarm-aarch64 base and compiler sets once and checked both against the upstream [SHA512 list](https://cdn.netbsd.org/pub/NetBSD/NetBSD-11.0/evbarm-aarch64/binary/sets/SHA512). Exact archive identities are retained below. Inventories covered 6,132 base and 18,794 compiler-set members. They checked member paths and types only; symlink and hard-link targets were not resolved or validated. No filename language candidate establishes absence of JavaScript or TypeScript. No native member body, ELF dependency, compiler resource, header or legal-content inspection closed the graph, and the distribution candidates were not shown to match the earlier #329 VM.

Inspection then reached the demonstrated CMake edge and stopped after 13 unique CMake bodies, within the issue's 80-body limit. No additional Qt/QXmpp archive bodies were read. The source prefix identifies bootstrap/system-tool dependencies, ordinary CMake utility selection and the forbidden template generation. The following remain unknown:

| Remainder | Qualification gap |
| --- | --- |
| Native tools and runtime | Actual selected shell/utility/compiler/make/linker bytes, resources, headers, ELF dependencies, runtime closure and their licenses. Archive checksums are distribution provenance only. |
| CMake bundled sources | Build utilities name KWIML, RHash, zlib, curl, nghttp2, expat, bzip2, zstd, lzma, libarchive, JsonCpp, libuv and llpkgc. Naming them does not qualify their selected bodies, generated inputs, external edges or terms. |
| CMake modules and output | Full compile/configure/module/install/copy graph, other templates, internal generators, possible downloaded inputs and complete installed assets. |
| Qt/QXmpp encrypted reference | Proposed Qt correction, selected Qt tools and third parties, OpenSSL, libomemo-c and remaining library/runtime/license closure described in 0064. |

The inspected CMake `Copyright.txt` contains BSD-3-Clause terms and third-party notices. Those notices are not a complete classification of bundled-source or native-tool obligations. No complete license clearance, distributability decision or dependency adoption follows. A prebuilt CMake would move the required provenance/content/license analysis to that binary distribution and its runtime graph; it supplies no opaque fallback qualification.

## Witnesses and distribution identities

All paths below are relative to `cmake-3.31.8/`. Each complete body was read; the lead and report author rechecked already inspected bytes without enlarging the unique-body count.

| Path | Bytes | Lines read | SHA-256 |
| --- | ---: | --- | --- |
| `bootstrap` | 67073 | 1–2127 | `391c107eefbc11a5f1e77a86d8624bc72103fa7daf1b0606ea55449875c11f5e` |
| `CMakeLists.txt` | 21927 | 1–564 | `df0c49d17be8cfa37a92dfffb0104fe15a48f9d444a291e723cef1043346fb4c` |
| `Copyright.txt` | 5477 | 1–136 | `0676e30827684487aa0ea8f72f01e9460a6320fa34a2888df0ad1dd3e8a06eb8` |
| `Source/CMakeVersion.cmake` | 2670 | 1–84 | `5fce23547efc1f3180db83ce3975aa6fde98ea4fbd81c92f68a3244dc7d764c6` |
| `CMakeCPack.cmake` | 10085 | 1–269 | `9e94e38f0528cb3360bff0864cfb192c56f863a3135bb8b44826b4c32065b218` |
| `Source/CMakeLists.txt` | 38995 | 1–1383 | `56c8faec94c80f1899ab3ffab2c5ae5b5e204da435603732b73a0dd08e2c2965` |
| `Utilities/CMakeLists.txt` | 635 | 1–21 | `6ee0b1ee260971651caa7f9c5c3580002ed9eb4c5a73ad2a008d9cdbabdf776b` |
| `Tests/CMakeLists.txt` | 144592 | 1–3586 | `0cc8020f9cc71b79c1d865168e6f99b253abb097b960b55570d074329bad5b62` |
| `Auxiliary/CMakeLists.txt` | 725 | 1–16 | `17f38abe09c90989435487b5660464ed4c9bfc0aa5698302bfa0072cfeee6f75` |
| `Source/CMakeInstallDestinations.cmake` | 2341 | 1–59 | `c0cea29968f77ff521946186c64347cc2151835eae4203b52cddbfecd96625fa` |
| `Source/Modules/CMakeBuildUtilities.cmake` | 14221 | 1–404 | `938c7a69efea96a17eeb2c6a49830374dee632964d33d08aabba8872c2e17fbb` |
| `Modules/CPackIFWConfigureFile.cmake` | 2268 | 1–68 | `19fcbfa39428313599763cd740b977486dbf588bd72a118d7bcf22f1852f0519` |
| `Source/QtIFW/CMake.Dialogs.QtGUI.qs.in` | 483 | 1–21 | `b3ae6cc7950b393aa2d39765e5edab15edbb05a88135a86d25d7002fcf16846c` |

The checksum-verified native distribution candidates were [base.tar.xz](https://cdn.netbsd.org/pub/NetBSD/NetBSD-11.0/evbarm-aarch64/binary/sets/base.tar.xz) and [comp.tar.xz](https://cdn.netbsd.org/pub/NetBSD/NetBSD-11.0/evbarm-aarch64/binary/sets/comp.tar.xz). Neither was installed or qualified.

| Archive | Bytes | SHA-256 | SHA-512 |
| --- | ---: | --- | --- |
| `base.tar.xz` | 44843188 | `28759aadf02f2a597d0c9686f7cac70d4c48a4d6f6a4cd2e3839c13fc8fc19c8` | `d17b3253959e110edba1755f481e707fd79a1a4a35bd7096a305c2959d6a5964713d4c35f439d76ca03d9252f8652d73f521ea6fc07a27ffa5ca22a80df6e7c5` |
| `comp.tar.xz` | 64706252 | `43ff8ea12ea32cab362af8e0b298f8c83ce6efddc29888dc4f50249a5bc379bd` | `0719f3d94fd5cdedf83ad7418f1e254c898b388c320f28d3d39b8abf96fe6ea58b605be8dc9e02d4d76021f87e3f0cd2bc327560e887abc572fbae04742305fd` |

Private retained observation records are identified by digest, without private paths or vendor content. These identities preserve inspection provenance; they are not executable verification tooling or fresh runtime results.

| Record | SHA-256 |
| --- | --- |
| `acquisition.json` | `6e64d8e4f0bdb7ed62a9e9a0e41b95246fb2c7963fc849b736c9f0a20c384eaf` |
| `analyst-source-reads-phase2.json` | `233e4cd850bd55fbcebe5c6c6cfbcd3021d88a9108e36ea1f3293a2452c3b6e9` |
| `analyst-phase2-no-go.json` | `7d62022c5fe9244f179f3670acbac213f6e3becf1303bf4298dcd636673d9c4e` |
| `lead-forbidden-edge-verification.json` | `b9bc9d6fb6313c5bb846a74e8c7782d689d6c5f65101acfe981506049c3c3867` |
| `netbsd-acquisition.json` | `6b8b7c954e35b38178d5aaef55adcdfdb7ba49c44121fa820e44d7893d339487` |
| `netbsd-member-inventory.json` | `88f0ddf2b5b10fa3757ef07217c3b436c84495406a0d628b3d9a5245dc062876` |
| `host-artifact-inventory.json` | `34659f30c6b521669ae303a71824ac582bb0305100d74b117d066e681dc8ea10` |
| `netbsd11-SHA512` | `91460f8143168dda91554ef570dcb33e1833e8b8714c909a69c8feb288564324` |
| `cmake-official-tag-ref.json` | `f11fe1d0db88bf17e45085924622e0556f4f7d15ebc63e486bc35ba319dc88dc` |

## Evidence and consequence

The tables above retain all 13 source witnesses and hashes of the private acquisition, analyst and lead verification records. The lead separately rechecked the four bodies supporting the forbidden edge. Private owned archives and inspection records remain outside Git; no upstream vendor bodies, keys, private endpoints or member content are included in this repository.

No CMake bootstrap/configure/build/install, reference build, native execution, strict-GCM RED/GREEN, descriptor/private-fetch implementation or reciprocal exchange occurred. No VM, listener or owner helper was launched; the saved owner playground stayed stopped and intact. This source result establishes no platform, hardware, capacity, production or cryptographic interoperability support. Rust, primary NetBSD, device-private keys/mail, classic access, default-off behavior and existing budgets are unchanged. Further correction or execution requires separately accepted scope; this report grants none.

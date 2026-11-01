# 0064: Corrected headless reference source profile

[Issue #327](https://github.com/gonz0w/faction-bbs/issues/327) examines one proposed correction to the isolated QXmpp reference selected for [#295](https://github.com/gonz0w/faction-bbs/issues/295). The result is **NOT QUALIFIED**: the required build-tool graph is unresolved. This is not a new demonstrated JavaScript violation. The normal QtBase profile's [0062 NO-GO](0062-headless-reference-qualification.md) remains unchanged. Neither result establishes independent attachment interoperability; #295 and [#88](https://github.com/gonz0w/faction-bbs/issues/88) remain open.

## Inputs and stopping point

Source inspection: 2026-10-31 (America/Denver).

The exact official archives were reacquired once for inert inspection, separate from the removed #297 cache and its historical evidence. Their byte lengths, SHA-256 values and embedded commits match 0062: QXmpp has 949,159 bytes, `29e2a2b76f8bde89b32b58e2f270593e104b17ada0e2da4820dd2201edd84a7d`, commit `86431213cbd4f5e1a70bd9c7a7db197006cf4c29`; QtBase has 68,693,288 bytes, `dff6570582c272cbe744d2e460700318262af505460f4215276459970ae31b46`, commit `025bdad181de241e81bf853c8a2d7bf3d19261a9`. Safe member/link inventories covered 526 and 27,583 entries respectively. No upstream archive member was extracted or materialized and no upstream code was executed. The eight original witness hashes in [0062](0062-headless-reference-qualification.md#witnesses-and-licensing-limits) matched again; historical records were not refreshed.

The first unresolved required edge is from the pinned QtBase [root build file](https://github.com/qt/qtbase/blob/025bdad181de241e81bf853c8a2d7bf3d19261a9/CMakeLists.txt#L10-L31) to the CMake executable, modules and their tool graph for both host and NetBSD. The root declares a parser floor of 3.16; [`.cmake.conf`](https://github.com/qt/qtbase/blob/025bdad181de241e81bf853c8a2d7bf3d19261a9/.cmake.conf#L19-L29) declares a supported floor of 3.25 for build, consumption, shared, static and Apple cases. Neither establishes an actual selected tool pin or proves enforcement. The same file identifies 6.12.0 alpha1; the tag alone does not establish stability. The exact CMake version, source, build, modules, transitive inputs, licenses and no-JS/TS graph are unknown. CMake content itself was not classified. The stop rule ended inspection after 11 unique source files; remaining configure reads, recursive copies, generators, installation, dependency discovery and generated or downloaded inputs are unknown. No fresh full content scan or representative translation classification followed the stop. XML translation `.ts` is not automatically TypeScript, but a suffix alone cannot qualify content.

Three additional QtBase witness SHA-256 values identify newly inspected files: `.cmake.conf` `e78b439fa15619a942f8b78d9321021778eb44dafba7f7067a1962637a5a4b64`, `src/CMakeLists.txt` `deeed002f55e34887312626f630131ac883e5b36e6d862668b04e584d72caba4`, and `cmake/QtBuild.cmake` `039543381b30bc02fce4f252eaa74cf89d9e4d30ae180cc6febf372b84641c34`. The private lead witness-verification record has SHA-256 `1d297f790874155a9d975c6a49720fac16a4adac9de3026e64e265be38333756`.

## Proposed correction and inert selection manifest

For this one candidate, remove only the documentation subdirectory traversal at [`QtBaseHelpers.cmake` lines 285–286](https://github.com/qt/qtbase/blob/025bdad181de241e81bf853c8a2d7bf3d19261a9/cmake/QtBaseHelpers.cmake#L285-L289), preserve normal `src` traversal at lines 288–289, and omit the forbidden documentation payload from any future materialized selected graph before configuration. Preserve applicable license, REUSE and notice material. This is a proposed build-input correction, not an applied patch or a qualified corrected source. Whether other configuration, generation or installation paths reach the payload, and whether the correction affects required library behavior, remain unknown. `QT_BUILD_DOCS=OFF` did not remove the old copy; standalone parts also skip normal `src` and are not a qualified remedy.

The following is a source-selection sketch, not an executable configuration or complete allowlist. Retention and exclusion are conditional on closing the actual reachable graph and legal obligations.

| Group | Proposed treatment and unresolved boundary |
| --- | --- |
| QXmpp source and notices | Retain independently as an isolated reference; no product dependency is adopted. |
| Qt root metadata and reachable CMake files | Retain; actual closure and configuration reads remain unknown. |
| Qt documentation traversal and payload | Remove the specified traversal and omit forbidden payload as above; other reachability remains unknown. |
| Qt Core, Network and Xml | Retain required implementations; selected files and transitive closure are unknown. |
| Host tools | Retain required `syncqt`, conditional bootstrap, `moc`, `rcc`, trace generators and automoc parser; [source traversal](https://github.com/qt/qtbase/blob/025bdad181de241e81bf853c8a2d7bf3d19261a9/src/CMakeLists.txt#L16-L29) does not establish their complete closure. |
| Third parties | Bundled or system choices including PCRE2 and zlib are unqualified; [source traversal](https://github.com/qt/qtbase/blob/025bdad181de241e81bf853c8a2d7bf3d19261a9/src/CMakeLists.txt#L33-L48) is not a complete inventory. |
| Examples, tests, GUI and platform assets | Proposed exclusions only where reachability proves them unnecessary; no blanket deletion is qualified. |
| Legal material | Retain applicable licenses, REUSE metadata and notices; complete obligations remain unknown. |
| External build inputs | CMake, generator, compiler, linker, SDK, base libraries and pkg-config sources, versions and licenses remain unknown; CMake is the first missing required edge. |
| Encryption dependencies | OpenSSL and libomemo-c are required for the intended encrypted OMEMO reference and remain unqualified; disabling OMEMO cannot prove that mail profile. |

The pinned QXmpp [root](https://github.com/KDE/qxmpp/blob/86431213cbd4f5e1a70bd9c7a7db197006cf4c29/CMakeLists.txt#L25-L56) requires Qt Core, Network and Xml and maps package/feature dependencies; its [library build](https://github.com/KDE/qxmpp/blob/86431213cbd4f5e1a70bd9c7a7db197006cf4c29/src/CMakeLists.txt#L347-L367) links Qt and encrypted-file edges, with the [OMEMO edge](https://github.com/KDE/qxmpp/blob/86431213cbd4f5e1a70bd9c7a7db197006cf4c29/src/CMakeLists.txt#L460-L461) remaining to qualify. These mappings do not close any dependency. The isolated reference's GPL/LGPL and corresponding-source/notice obligations require review; they do not change Faction's BSD license or constitute adoption.

## Consequence and next gate

No configure, build, install, corrected-source qualification, native execution, GCM RED/GREEN, descriptor, private retrieval or reciprocal exchange was run. The next gate is one coherent qualification of the exact CMake and required bootstrap-tool source graph for this same profile, including provenance, versions, licenses and content. Only after closure can a later scoped behavior correction use actual RED/GREEN evidence. No JS/TS waiver, runtime change or target change follows. Rust, primary NetBSD, device-private keys and mail, classic access, default-off behavior and existing budgets remain unchanged; the owner playground stayed stopped.

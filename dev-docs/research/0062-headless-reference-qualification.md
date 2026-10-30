# 0062: Headless reference qualification

[Issue #297](https://github.com/gonz0w/faction-bbs/issues/297) qualifies the
owner-selected, isolated headless QXmpp reference considered in
[0061](0061-private-attachment-delivery-contract.md). The selection authorized
qualification; it did not guarantee a pass or relax Faction's no-JS/TS rule.
This report records source preflight, not an adopted client or runtime. Primary
sources were retrieved 2026-10-30 local time. The selected Faction baseline was
`abf332d`.

## Decision

**NO-GO for the selected normal QtBase 6.12.0 source/build profile:** its normal
configure path copies actual JavaScript into the build tree. This fails G1 before
qualification of the reference's complete source and build graph. The conclusion
is source-based and specific to this profile; it does not establish that all Qt
versions or configurations fail, or that a headless runtime executes JavaScript.

| Gate | Result | Evidence or limit |
| --- | --- | --- |
| G1, no JS/TS | Fail for the selected normal build profile | QtBase's configure-time documentation copy below. |
| Complete graph and distributability | Unknown | Transitive sources, tools and license obligations were not closed. |
| Strict GCM and correction | Not run | Preflight stopped before correction or a red/green claim. |
| Descriptor and private access | Not run | No descriptor acceptance or private-access exchange was performed. |
| Interoperability and native execution | Not run | No reference build, execution or reciprocal exchange occurred. |

A failed gate is not offset by other attributes, and unknown is not a pass.
This finite research result can close #297 only after review; #295 and #88 remain
incomplete. The independent [#293](https://github.com/gonz0w/faction-bbs/issues/293)
SSH workflow is separate from this qualification. No JavaScript exception or new
target selection follows from this result.

## Selected inputs

| Input | Identity and archive | Bytes | SHA-256 |
| --- | --- | ---: | --- |
| QXmpp 1.17.0 | commit `86431213cbd4f5e1a70bd9c7a7db197006cf4c29`; [archive](https://codeload.github.com/KDE/qxmpp/tar.gz/86431213cbd4f5e1a70bd9c7a7db197006cf4c29) | 949159 | `29e2a2b76f8bde89b32b58e2f270593e104b17ada0e2da4820dd2201edd84a7d` |
| QtBase v6.12.0 | commit `025bdad181de241e81bf853c8a2d7bf3d19261a9`; [archive](https://codeload.github.com/qt/qtbase/tar.gz/refs/tags/v6.12.0) | 68693288 | `dff6570582c272cbe744d2e460700318262af505460f4215276459970ae31b46` |

Both archive-embedded commit IDs were checked using gzip and
`git get-tar-commit-id`. The official [Qt 6.12 listing](https://download.qt.io/official_releases/qt/6.12/)
exists, while the selected source's `.cmake.conf` identifies an alpha1 version.
The tag and listing alone do not establish release stability or support status.

QXmpp's pinned [root build](https://github.com/KDE/qxmpp/blob/86431213cbd4f5e1a70bd9c7a7db197006cf4c29/CMakeLists.txt)
requires Qt 6.4 or newer with Core, Network and Xml; its
[library build](https://github.com/KDE/qxmpp/blob/86431213cbd4f5e1a70bd9c7a7db197006cf4c29/src/CMakeLists.txt)
links those modules. The QXmpp archive inventory found no `.js`, `.ts` or `.qml`
suffixes. Its HSLuv attribution to a JavaScript implementation describes ancestry
of C code, not a JavaScript source dependency or a violation by itself.

## Required copy edge

The pinned QtBase [root CMake file](https://github.com/qt/qtbase/blob/025bdad181de241e81bf853c8a2d7bf3d19261a9/CMakeLists.txt)
invokes `qt_internal_qtbase_build_repo`. In its
[helper](https://github.com/qt/qtbase/blob/025bdad181de241e81bf853c8a2d7bf3d19261a9/cmake/QtBaseHelpers.cmake),
the normal branch, when `QT_INTERNAL_BUILD_STANDALONE_PARTS` is false, adds
`doc` unconditionally before `src`. The [doc CMake file](https://github.com/qt/qtbase/blob/025bdad181de241e81bf853c8a2d7bf3d19261a9/doc/CMakeLists.txt)
iterates over `global` and `config`, invokes `qt_copy_or_install`, and copies
those directories during configure when the project is QtBase or a superbuild.
The copied global tree includes [main.js](https://github.com/qt/qtbase/blob/025bdad181de241e81bf853c8a2d7bf3d19261a9/doc/global/template/scripts/main.js)
and [extras.js](https://github.com/qt/qtbase/blob/025bdad181de241e81bf853c8a2d7bf3d19261a9/doc/global/template/scripts/extras.js),
which contain JavaScript cookie and DOM functions.

This copy is independent of documentation generation: disabling `QT_BUILD_DOCS`
does not remove it. Disabling GUI, QML or WebAssembly likewise does not remove
this normal branch. `QT_NO_SET_QTBASE_DOC_FILE_DEPS_COPYING` only controls
registration of file dependencies, not the copy. The standalone-parts branch also
skips normal `src` traversal; it was neither attempted nor qualified as a remedy.
This is a required build edge, not a finding based only on unused WASM files.
No configure, build, install or JavaScript execution was performed.

## Witnesses and licensing limits

The following SHA-256 values identify the inspected files within the pinned
archives. Exact GitHub blob reads also matched the local witness Git blob hashes.
Paths are relative to the respective archive root; no upstream source is copied
into this repository.

| Input | Relative path | SHA-256 |
| --- | --- | --- |
| QXmpp | `CMakeLists.txt` | `cdc1f9bf3ad37669f633d60c2c3fa11df0e2e398aed7d002344c2fede1efbd58` |
| QXmpp | `src/CMakeLists.txt` | `ecedf4445769df2eefcfc5f1f20cd176b9652d93e896a4bc1d22ab0f1e7bab29` |
| QtBase | `CMakeLists.txt` | `19284751242b33ff39b86d0b75afd587aae465cbca5d8347f1fe71d1bd9113d2` |
| QtBase | `cmake/QtBaseHelpers.cmake` | `d76bbecc2aee9469e84f1fc4ffff02e37da74194809c1c96e96c8d5d20f5f0f8` |
| QtBase | `doc/CMakeLists.txt` | `82ce20d82c29fb6c6498fabea31c07a7dd72708f8564a89eb50c0deefcf8806b` |
| QtBase | `doc/global/template/scripts/main.js` | `a0b090a1ce644cb323ce6e99dc36dc00cfa1eaacda114d99bef80ab774fb5f10` |
| QtBase | `doc/global/template/scripts/extras.js` | `c4b9267be24584ebc73833422f55cd3ec2caa69e0935b107334a5eecbd49a21b` |
| QtBase | `REUSE.toml` | `ac3a75d082888c1379902f53d42583322cdbe9bd9d2d49a5464a8c0e31837316` |

QtBase's pinned [REUSE metadata](https://github.com/qt/qtbase/blob/025bdad181de241e81bf853c8a2d7bf3d19261a9/REUSE.toml)
assigns both scripts `LicenseRef-Qt-Commercial OR GPL-3.0-only WITH
Qt-GPL-exception-1.0`; the examined CMake files identify BSD-3-Clause. Effective
QXmpp GPL/LGPL-3 conditions also require distribution review. These observations
are not complete legal clearance or a transitive license inventory.

Archive size and SHA-256 were measured locally; the listed witness SHA-256 values
were checked with `shasum -a 256`. The archive inventory and pinned source reads
were inspection only. A separate reviewer verified both archives, embedded commits and all eight witness
hashes before the owned source cache was removed. No upstream source remains in
the repository or that disposable cache.
OpenSSL, OMEMO, transitive dependencies, tools and runtime inputs were not fully
closed or approved. No native run, network run, corrected source, descriptor
acceptance, GCM red/green result or interoperability claim is made.

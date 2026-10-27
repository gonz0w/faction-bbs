# Attachment metadata proof evidence

This selected record belongs to [issue #252](https://github.com/gonz0w/faction-bbs/issues/252) and the [experimental descriptor contract](../../attachment-metadata.md). It preserves host observations and actual test-first history, not a fresh test run or product attachment delivery. The baseline is `7c94f51de4c9537d2db77fdc02b4ea6d1db3232e`.

## Recorded history

- `observations/red` retains the original missing-interface logs. Both commands exited 101 because the new attachment module was absent; this is not a defect in existing text mail. The original three test sources are resolved by `source-aliases.json`.
- The unformatted variant retains its inputs and the reported format-check failure. That console output was not captured as a full local log. The checkpoint fixture was corrected before its first runtime run: valid dot-host addresses and a self-owned portable temporary directory replaced invalid addresses and a required local environment variable. These were fixture corrections, not observed behavior REDs.
- `observations/run01` retains a lock-only setup failure. The existing Base64 package needed a version-qualified dependency edge; no behavior tests executed in that run.
- `observations/run02` records the first eight passing descriptor/native/checkpoint tests. `run03-characterization` strengthens missing/duplicate coverage and tests absent/wrong peer pins with the same recipient's private state before the ownership refactor.
- `observations/run04-final` retains the fixed-buffer Base64 variant and a Clippy needless-borrow rejection. `observations/run05-final` records all five commands passing after the borrow correction: five descriptor, two native, one checkpoint and six existing SCE tests, formatting and both-package native Clippy.
- `review` contains a separate reviewer's unchanged rerun record and logs: all 14 focused tests passed. It is a second examination, not independent human assurance or a modern-client interoperability test.

`source-aliases.json` maps original private-layout source paths to deduplicated `objects`. Original run records and logs are unchanged. Paths in those records describe the execution-time layout, not currently available private fixtures. `environment.json` identifies Rust 1.96.0, macOS 27.2 ARM64, Apple Clang 21 and the explicitly selected existing OpenSSL 3.6.3 library. No binaries, runtime vaults, native private snapshots, dependency trees or member data are included.

The separate `lead-regressions` record reports 49 feature-off passes and 154 native-mail passes across their Cargo test harnesses, with six pre-existing ignored helper entries in the native run. `lead-policy-checks` preserves 478 passing tooling tests and nine retained Temper hash checks. These are host/debug observations; the logs distinguish them from the focused producer and reviewer runs. `integration.json` records the reviewed dependency-edge reconciliation and its initial unavailable-cache outcome.

## Meaning and limits

The native tests authenticate before parsing, commit valid metadata, and abort authentic malformed metadata without changing committed snapshots. Same-recipient trust controls distinguish missing/wrong pins from a wrong recipient secret. The checkpoint test authenticates a candidate extension, rejects it through the current text-only receiver without changing canonical ciphertext/history/metadata, and then accepts ordinary text.

The descriptor validates assertions about size/hash; it does not compare them with a blob, prove key freshness, route numeric device IDs, fetch a URL or save a file. No blob encryption, streaming, private storage, persistence migration, two-home delivery, modern client profile interoperability or new NetBSD result is established. Earlier proofs remain separate. The saved owner playground stays stopped and intact.

`cleanup.json` records the actual removal of both marked owned fixtures after separate review, hash/ownership checks and process quiescence. No owner-playground operation, VM or listener was started. `metadata` retains the actual Git whitespace checks: the first check omitted globally ignored logs, the complete selection exposed final blank lines, and fifteen exact log-path attributes preserved those original bytes. No source/runtime/global whitespace exemption was added.

The existing generic `tools/check_recorded_files.py` can check this folder's `manifest.json` with its independently reviewed SHA-256 from the closing PR. That check verifies saved bytes only. Current native unit tests run through the existing hosted all-targets jobs; no new CI verifier or runtime is selected.

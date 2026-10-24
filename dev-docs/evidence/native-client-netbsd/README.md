# Native terminal client and file-link validation

Current source validation, 2026-10-21; sanitized [results](results.json).
Software NetBSD 11.0 ARM64, QEMU/HVF, two virtual CPUs and 1 GiB RAM.
Real native clients run on macOS and NetBSD against base SSH and the maintained
shared BBS/restricted SFTP service. Default eight sessions, credential limits and
owner process/thread limits are unchanged. No physical/capacity/peak-memory claim.

The maintained `tools/smoke_native_client_ssh.py` accepts private trusted argv/profile
fixtures and requires a marked disposable board. It exercises pending-member login,
terminal restoration, file-menu negotiation/arrows/hotkeys, upload/review/download,
local no-overwrite failure/cancel/retry, private room/chat links, recent-reference
selection and revoked access. The full run includes a NetBSD-native client and a
wrong-host-key profile. A separate stopped/drained full catalog/blob backup/restore
checks schema 5, retained public namespace and references, rotated private board
identity, empty transfer grants and exact published bytes. Unit tests cover migration,
policy, controls and local filesystem handling; results record actual counts.

All data and identities are synthetic. VM image/kernel provenance follows the
signed official artifacts and fingerprints in research 023. Endpoint/known-host/key
files, passwords, raw transcripts, disk and generated member payloads remain private
and are removed with the task VM. Results include source hashes and teardown state,
not private endpoints or credentials. The CI checker validates this record and
source identity; it does not rerun a VM. No XMPP/OMEMO, encrypted attachments, device
trust, home identity or roaming login is implemented by these tests.

Historical file-area result 13/90/90/101 is retained byte-for-byte through the sixth
baseline snapshot. It is not a measurement of this client increment. See
[ADR 0033](../../decisions/0033-native-terminal-files.md) and
[GitHub issue](https://github.com/gonz0w/faction-bbs/issues/87).

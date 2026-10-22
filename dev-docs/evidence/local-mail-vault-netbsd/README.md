# Native local mail vault evidence

[Research 0046](../../research/0046-native-local-mail-vault.md) describes the
maintained Rust implementation and exact software-only observation limits.
`results.json` records 105 host/native NetBSD workspace Rust tests, 13 real mail
PTY controls per platform, native build versions, source identities and cleanup.
The source checker does not rerun the VM or audit cryptography. This is offline
local persistence/UI evidence, **no network inbox/send, OMEMO device state,
roaming authentication, encrypted attachments, hardware or capacity claim**.

Reproduce with Rust 1.96.0: cargo test/build --workspace --locked --release,
then `python3 tools/probe_native_mail.py --client /absolute/faction-client`.
Use an unprivileged account and private disposable data on each native host. The
probe runs the actual client in PTYs, generates synthetic secrets, verifies
ciphertext/private files and recovery, and removes private fixture directories.
Never record transcripts/passwords/recovery secrets. A NetBSD ARM64 VM must use
verified official release inputs; the recorded fixture is 2 vCPUs/1024 MiB with
loopback-only SSH management. It proves no physical SBC or other-port result.

The pre-mail #89 native-client result and all 89 executed sources remain unchanged
under `mail-client-baseline-manifest.json` / `sources/pre90`. No hash was refreshed
to make an older network/client result describe these new mail sources.

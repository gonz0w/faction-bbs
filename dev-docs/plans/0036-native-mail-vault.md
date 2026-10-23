# 0036: Native local mail vault and trusted device-side UI

Issue: [#90](https://github.com/gonz0w/faction-bbs/issues/90), parent #84; roaming #91 and encrypted attachments #88.
Status: implemented and validated bounded increment, 2026-10-21; #90 remains open

## Goal and constraints

Ship the member-device persistence/unlock foundation in Rust without adopting the
research Python runtime or inventing an OMEMO ratchet. The first increment provides
an authenticated encrypted local vault, private drafts, explicit device pins,
create-only encrypted backup and recovery. Network inbox/send/reply, actual OMEMO
identity/session state, adapter interoperability and roaming admission remain #90/#91.
Do not call a local address claim an authenticated home identity or a local pin a
server/device enrollment. No server-rendered plaintext, remote unlock control,
BBS credential changes, new owner helpers or public service.

## Progress

- [x] Merge verified #93 and read current requirements.
- [x] Preserve 89 actual #89 execution sources and unchanged evidence before edits.
- [x] Review pinned RustCrypto dependency licenses/features/no-JS source graph.
- [x] Implement local authenticated storage, bounds and trusted UI.
- [x] Run host security/PTY checks and native NetBSD software validation.
- [x] Update product/status/decision docs; prepare issue-linked focused PR.

## Steps and commands

Use Rust 1.96.0: cargo +1.96.0 fmt/clippy/test/build. Run repository checks,
Python unit tests, dependency/evidence guards and tools/probe_native_mail.py.
Use a fresh private NetBSD 11 ARM64 two-core/1-GiB software VM for native build,
workspace tests and the same interactive mail probe. Remove fixture credentials,
private stores and VM after recording non-secret observations. No hardware or
capacity claim. Never refresh old measured digests to new client code.

## Discoveries and decisions

RustCrypto chacha20poly1305 0.11.0 (alloc/zeroize only) adds seven permissively
licensed packages; selected Linux/NetBSD graphs pass the full package filename
policy. Argon2 0.6.0 and zeroize were already dependencies. Local Argon2id uses
64 MiB, three passes, one lane; server Argon2 and credential permits are unchanged.
A random local data key is independently wrapped by the password-derived key and
a 256-bit offline member recovery secret. This is a local persistence format,
not an OMEMO protocol. See the accompanying ADR for review/portability limits.

Path hardening initially exposed macOS's symlinked /var temp prefix; use canonical
fixture paths and traverse every vault directory component with NOFOLLOW. Do not
weaken the checks to accommodate a symlinked example path.

## Validation and outcomes

Rust 1.96.0 workspace tests: 105 host, 105 native NetBSD; build/clippy/fmt pass.
The actual local client passes 13 disposable PTY controls on each platform;
Python repository suite has 104 passing tests. Native C frontend/daemon builds
also pass. Guest/host hashes match for all 93 executed inputs. The VM/stores and
synthetic keys are removed after evidence capture. No hardware/capacity claim,
new live BBS/SFTP run or native network mail is implied. See research 0046 and
local-mail-vault-netbsd/results.json.

#90/#84 must stay open: real native receive/send,
independent-client interoperability, device key enrollment/ratchet durability,
archive/reconnect/dedup/history recovery and integrated roaming are not implemented
by the local vault. Attachments stay in #88; no public file-area substitution.

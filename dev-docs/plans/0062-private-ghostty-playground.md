# 0062: A populated, persistent Ghostty playground

Issue: [#152](https://github.com/gonz0w/faction-bbs/issues/152), related #21/#71
Status: implemented; PR validation pending

## Goal and constraints

After merging checked #151, give the owner a real private SSH login to the current
NetBSD software preview. Batch the related lifecycle, sample-community and return-
visit work. Preserve the earlier saved playground, classic terminals, ordinary
approval/SSH/credential/session limits and device-only private-mail direction.
Hardening remains later. No public endpoint, image distribution or capacity claim.

## Progress

- [x] Verify #151 CI, merge its checked head, close completed #135.
- [x] Add an explicitly new-only demo seed using existing core operations.
- [x] Add private prepared-VM status/start/stop/connect commands with PID identity checks.
- [x] Clean-build the unchanged current runtime and tests inside NetBSD 11 ARM64.
- [x] Populate five rooms, local wall/pages/plans/chat and enable existing Lua demos.
- [x] Verify the exact supplied SSH profile, compact fallback and restart persistence.
- [ ] Run repository checks, update docs, open/attach PR and hand over live login.

## Steps and commands

Build `cargo build -p faction-core --example playground_seed --locked`, then run
`python3 tools/smoke_playground_seed.py target/debug/examples/playground_seed`.
Run `python3 -m unittest discover -s tools -p 'test_*.py'` and
`python3 tools/check_repository.py`. See [playground guide](../../docs/test-playground.md)
for private connection and lifecycle commands. Credentials, VM paths, SSH argv,
full logs and image checkpoints stay in ignored owner-private storage.

## Discoveries and decisions

2026-10-22: #151 merged as 268c466 after successful GitHub checks. The next useful
batch makes current social features reviewable in the owner's actual terminal.
It does not add another runtime feature or schema. Existing measured source bytes
and all 42 evidence guards stay unchanged; no historical hash refresh or new
measurement claim. The seed is a new example, outside the server entry points.

The first seed attempt rejected channel names without their required `#` prefix;
corrected the example to use the existing channel contract. Native owner execution
initially could not traverse the build account's private directory; copied the
fixture executable into the private preview parent, ran as the board owner and
removed that helper. Theme validation correctly required owner-owned packs;
prepared copies retain the existing factiond ownership rule. No runtime policy was
relaxed for these preparation fixes.

## Validation and outcomes

Final GitHub CI remains pending. Native clean build:
235 existing Rust tests, eight page PTYs, seven wall PTYs, seven full-screen PTYs,
thirteen writing PTYs and the existing room/browser/operator/directory/art checks
passed. New seed refusal/content checks passed on macOS and NetBSD. The initial
actual supplied-profile SSH journey passed using `TERM=xterm-ghostty` at 100x32:
Nightwire identity, rooms, wall, pages, chat, Lua menu/Neon Salvage and logout.
This is a software PTY check, not observation of Ghostty pixels, fonts or physical keys.

Final local checks: 235 Rust tests on macOS plus Clippy `-D warnings` and rustfmt
passed; 129 Python tests and all 52 offline policy/evidence/Python workflow commands
passed. All 42 saved-evidence guards and their historical inputs remain unchanged.
The new seed content/refusal check passed on both macOS and NetBSD; eleven new
lifecycle unit checks passed on both hosts. Installed faction/service hashes match
the clean native build. No existing runtime source or registry inventory changed.

Actual guest shutdown/start and the exact supplied pinned SSH config passed after
restart for approved ordinary `cam` at 100x32 and BBS `sysop` at 40x18. The populated
rooms/wall/pages/plans survived; chat and existing Lua menu/Neon Salvage returned.
An initial immediate-start port precheck falsely rejected TCP TIME_WAIT; matching
QEMU's SO_REUSEADDR behavior fixes that without allowing a live listener, covered
by a real bound-listener refusal test. The preview is deliberately left running in
ignored private storage for the owner; private credentials/ports/VM paths are not
published. Files/SFTP stays disabled in this fixture; wider product stories remain.

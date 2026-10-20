# Remaining transport memory (#58)

Separate software-only experiment using a separately versioned Rust shared-board candidate
from [055](../055-shared-board/README.md), existing base OpenSSH, the same native
descriptor bridge and independently salted fixed fixture. No service defaults,
core, normal frontend, schema or reviewed dependency graph are changed.

The variant copies 055 with one correctness fix: reset the chat request sequence
to one whenever opening a new core lease. Fresh unique delivery markers exposed
a repeat-chat-entry failure in 055; its frozen source/evidence are preserved.
Both allocator profiles use this identical corrected variant. Build/Clippy and
dependency review use this directory's standalone manifest/lockfile; the package
and dependency versions/features are identical to 055. No UI parity adoption.

`allocator.c` queries the actual NetBSD libc's versioned `__je_mallctl` interface.
It fails if any option cannot be read. It is a native research helper, not a
portable application API. Compile in the marked disposable guest with
`cc -std=c11 -O2 -Wall -Wextra -Werror allocator.c -o faction-allocator-probe`.

Compare a fresh-boot default allocator against the explicit guest-wide
`/etc/malloc.conf` symlink target
`narenas:1,tcache:false,dirty_decay_ms:0,muzzy_decay_ms:0,retain:false`.
The default readback must be eight arenas, tcache true, dirty decay 10000 ms;
the candidate must read one arena, tcache false, dirty decay zero. Actual image
version/options are recorded. The setting affects newly initialized guest
processes including SSH, administration and metric helpers: include all their
effects in the whole-VM comparison. Never apply it to a developer host or live
board, nor infer options from an environment variable or generic manual alone.
It trades allocation synchronization/page churn against resident retention.
No allocator library or interposition dependency is added.

`tools/check_transport_scaling.py --help` describes the marked-fixture driver.
Use existing 055/056 fixtures and native helpers. Reset the offline synthetic
board and reboot before each trial. Save exact source/native binary hashes.
The optional 32-transport attribution observes key-authenticated no-channel SSH
and then real handed-off password prompts. Actual distinct approved stages are
32/100/200/250 with menu/chat and a fresh unique delivery marker per stage.
All SSH helpers/frontends and VM anon/file/pool/free/CPU belong in the ledger.
No-channel sessions are attribution observations, never BBS members.

The original 128-MiB free plus clean-file-minus-wired guard, zero swap and kernel
process/file reserves remain. Save every decision and derived batch change;
checks between eight admissions cannot guarantee a continuously maintained
reserve. For high-load credentials, replace two approved transports while the
remaining cohort stays in chat; hidden passwords use the unchanged two-slot
Argon2 path. Use a persistent native sampler and 0/19/38-MiB touched controls.
Sampled maxima are lower bounds, not private memory or simultaneous KDF proof.

Full shared UI/theme/plain/resize/fairness adoption remains #55. Short load
trials do not establish full #28 active-mix/soak/appliance or physical SBC support.

## Minimal handoff frontend candidate

Two default trials stopped at 224 approved menus; two allocator-only compact
trials stopped at 248. Both profiles passed 200 useful members. The measured
32-transport prompt phase still adds anonymous and kernel memory. This justifies
a separately reviewed `minimal-frontend.c` candidate, not a wholesale language
change: the core/shared owner remains Rust and base OpenSSH is unchanged.

The C candidate sends the existing fixed 72-byte packet and real TTY descriptor
through the unchanged native `faction_send_fd` bridge. It exposes no database,
crypto, account identity/capability or owner API. Fixed stack arrays and capped
ASCII TERM/UID/path parsing avoid application heap allocation. Parent/socket
ownership/modes, no symlinks, distinct nonzero owner, TTYs and SSH command/environment
gates match the configured Rust frontend's contract. UID parsing is deliberately
decimal-only. One borrowed terminal duplicate serves handoff and restoration;
there is no extra restore duplicate. SIGPIPE is ignored as in Rust std.

Nonblocking connect/frame writes have two-second absolute deadlines; the single
status/EOF wait has a 1900-second absolute deadline. Terminal attributes restore
on normal success, backend status error/EOF and other returning errors. SIGKILL
cannot run cleanup in either frontend; actual SSH/backend failure oracles qualify
that path. No promise of arbitrary signal-safe cleanup is made.

Only the marked disposable guest may temporarily install this candidate at its
fixed forced-command path. Preserve the original root-owned Rust binary for
identity/rollback; never add this install to normal services. Compile natively:

```sh
cc -std=c11 -O2 -Wall -Wextra -Werror -fstack-protector-strong \
  -D_FORTIFY_SOURCE=2 -fPIE -pie -Wl,-z,relro,-z,now \
  experiments/058-transport-memory/minimal-frontend.c \
  crates/faction-unix/src/terminal_fd.c -o faction-ssh-minimal
cc -std=c11 -O2 -Wall -Wextra -Werror -fanalyzer -c \
  experiments/058-transport-memory/minimal-frontend.c -o analyzed.o
```

Native ELF readback verifies PIE, immediate binding, RELRO and separate RX/RW
LOAD segments. NetBSD's actual ELF does not contain a Linux-style GNU_STACK
header; do not invent a verified stack property from its absence. The linked
runtime is base libc only. C requires manual bounds/lifetime review in addition
to GCC analysis: path prefix is shorter than its capped socket path, TERM fits
packet[9..72], UID arithmetic checks overflow, IO lengths stay within the fixed
frame, and every owned FD is closed once. No pointers or buffers cross lifetimes.

`frontend_probe.py` runs only as root in the explicitly marked disposable guest.
Twenty direct oracles include actual descriptor/TTY ownership and packet bytes,
success/error/EOF/SIGPIPE restoration, hostile CLI/UID/TERM and unsafe endpoint
denials. Its PTY harness drains output before waiting for exit. Run the full real
SSH denial and shared approval/grant/failure suites before fresh-boot load tests.
Candidate-only source hashes and active/original frontend binary identities
must be distinct in the saved evidence. A favorable memory result does not adopt
C as the product frontend or waive #55 complete UI and #28 deployment gates.

Validated #58 load invocations use `--model shared` and the default 250 offer
ceiling. Inherited worker/smaller-ceiling harness options are not part of this
comparison's native evidence. Final minimal-frontend workloads reached 250 twice;
one retained the final guard and the repeat finished 4.10 MiB short. This is
functional progress, not repeatable capacity adoption; #58 remains open.

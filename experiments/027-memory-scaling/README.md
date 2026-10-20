# NetBSD memory/scaling experiment

Disposable software fixture only; no production deployment or physical SBC claim.
Read [report 0027](../../dev-docs/research/0027-memory-scaling.md),
[ADR 0014](../../dev-docs/decisions/0014-credential-budget-and-scaling-experiments.md)
and the prior [native fixture guide](../../dev-docs/research/0024-restricted-ssh-netbsd.md).

Use a fresh official signed/hash-verified NetBSD 11 ARM64 guest, two vCPUs/about
1 GiB RAM, private loopback host forwards, serial-verified host keys, separate
admin/member keys, native Rust 1.96/C/Python and the exact workspace source/vendor
inventory. Run native application/PTY/SSH/lifecycle checks before profiling. Root
guest marker `/etc/faction-disposable-test` and absent profiling database are
mandatory. Fixed fixture paths/accounts are intentionally incompatible with an
existing installation. Do not install the fixture example into member access.

Inside the guest build the example as the board owner and collector as root:

```sh
cargo build -p faction-core --example profile_seed --release --frozen -j 1
cc -std=c11 -O2 -Wall -Wextra -Werror experiments/027-memory-scaling/netbsd_metrics.c -o /usr/local/libexec/faction-netbsd-metrics
```

The example requires a NEW private `/var/db/faction-profile` directory owned by
the board owner, its `.profile-fixture` marker and a private generated password on
stdin. The driver creates/seeds this fixture; it never copies a live board.

From the developer host, pass private test connection arguments (placeholders
below, never actual endpoints/credentials committed):

```sh
python3 tools/check_memory_scaling.py --disposable-fixture --host PRIVATE_GUEST_HOST --port MEMBER_PORT --admin-port ADMIN_PORT --identity PRIVATE_MEMBER_KEY --admin-identity PRIVATE_ADMIN_KEY --known-hosts PRIVATE_PINNED_HOSTS --output PRIVATE_SANITIZED_RESULT
```

Optional `--max-sessions` bounds the highest offered stage. The harness explicitly
records experimental guest process/file ceilings and restores them on exit. Headroom
guards stop further admission; a stopped stage is not a capacity success. Normal
service remains eight workers. Delete the private guest/disks/keys/raw logs and
forwards after verifying final source and saving sanitized measurements.

Python stdlib-only accounting tests run in regular CI. Kernel metrics and actual
SSH scaling require native NetBSD; Linux/macOS output is not a substitute.

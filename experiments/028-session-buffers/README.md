# BBS-028 allocation ownership probe

Disposable C11 research; no server, production stack decision or third-party code.
Compare requested live allocations for identical synthetic delivery to 250 sessions:
copies versus an immutable shared pool and bounded reference array.

```sh
python3 tools/measure_buffer_probe.py --run
python3 tools/measure_buffer_probe.py --check
```

Run from repository root with Python 3.10+ and `cc`. Binaries compile in a temporary
directory and are removed. Five fresh processes per strategy verify delivery checksums.
Allocation totals exclude allocator overhead/RSS/runtime/crypto/OS/storage. There are no
real connections or users. The 64 KiB/session workload is a stress example exceeding
the proposed 8 KiB queue cap; it is not a recommended queue size. Reference release,
slow consumers and per-recipient permissions need a separate server prototype.

See [scope, actual results and validation plan](../../dev-docs/research/0028-small-host-budgets.md)
and [sanitized raw observations](../../dev-docs/research/results/028-buffer-probe.json).

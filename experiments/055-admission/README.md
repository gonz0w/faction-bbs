> Rejected for the 250-member memory target; retained for reproducibility.

# SSH admission accounting probes

Related [#55](https://github.com/gonz0w/faction-bbs/issues/55) and
[research 0036](../../dev-docs/research/0036-ssh-admission.md).

The separate native collectors preserve earlier measured collector bytes.
`netbsd_metrics.c` counts the admission listener and every base OpenSSH descendant
in the existing `ssh` transport role. `live_limits.c` records the listener
separately, queries live rlimits and explicitly counts completed config-sender
zombies; they remain in whole-host/process accounting. Kernel-reported zombie
RSS values can retain exit accounting. These sums are not physical/unique RAM.

The launcher itself is in `native/faction-admission/`; the builder and private
fixture drivers live in `tools/`. See [the operator contract](../../docs/experimental-admission.md).
No keys, endpoints, profile secrets, raw logs or real community data belong here.

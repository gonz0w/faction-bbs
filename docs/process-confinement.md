# Native process envelope and terminal peers

The NetBSD shared service and native `faction-tty` frontend apply irreversible
resource ceilings and authenticate both ends of the local terminal socket.
This is development hardening, with native software-VM evidence. The shared
owner remains trusted with the board database. Filesystem, network, exec and
same-UID hostile-code isolation remain open security gates.

| Kernel resource | Shared service | Native frontend |
| --- | --- | --- |
| Core-file bytes | 0 | 0 |
| Locked bytes | 0 | 0 |
| Fork/process limit | 0 | 0 |
| File descriptors | `2 * max_sessions + 48` | 8 |
| Additional NetBSD LWPs | 3 | 0 |

Both soft and hard ceilings are set and read back. A lower inherited limit causes
startup to fail; the application never raises it. The shared ceiling is 64 files
at the eight-session default and 548 in the explicit 250-session experiment.
The three helper threads start before the thread ceiling: one descriptor handoff
and two credential workers. NetBSD counts additional LWPs separately from each
process's initial thread. Native probes verify denied fork, extra thread, mlock,
excess files, hard-limit increases and actual crash without a core file.
These ceilings do not enforce a total application RAM budget. NetBSD's RSS limit
is not an implemented UVM memory boundary. Argon2 settings and its two-slot budget
are unchanged.

Use the root-managed [service lifecycle](service-lifecycle.md). Its shared launch
uses a minimal environment (`PATH`, `HOME`, `USER`, `LOGNAME`) before executing
Rust, with a non-root owner UID, primary group only, private state/log and working
directory `/`. The service rejects mismatched real/effective IDs, unexpected
supplementary groups and inherited descriptors above stderr before opening the
board. Standard descriptors are marked close-on-exec. The native C frontend
closes inherited descriptors above stderr and clears its environment after
copying the bounded validated TERM hint. This does not erase old environment
memory or undo dynamic-loader behavior before main; root-controlled SSH service
configuration remains part of the boundary.

The frontend requires stdin/stdout/stderr to refer to the same terminal owned by
its UID. It checks the protected socket path and the connected peer's kernel UID
before sending the TTY descriptor. The service checks the accepted peer's kernel
UID against `terminal_uid` before receiving the frame/descriptor. Merely chowning
a socket or passing a terminal-owned descriptor from another UID fails. Peer IDs
describe connection/bind-time credentials, not proof that a process is still
executing a particular binary. Compromised code under the permitted UID remains
outside this guarantee. SSH admission still grants no local BBS approval.

The control protocol remains the fixed terminal handoff followed by exactly one
completion status byte and EOF. The frontend rejects extra/malformed status and
requires EOF within two seconds of a success byte, preserving TTY restoration.
It provides no command, file, owner-tool or extension channel.

Defaults remain eight sessions and five-second token refill. Larger/faster
settings require explicit experimental configuration. No script installs allocator,
NTP, sysctl, chroot or global tracing policy. [Research 0034](../dev-docs/research/0034-process-confinement.md)
records native tests and full 250-member measurements. These short VM observations
are not a supported capacity or physical SBC claim.

Future Lua/doors must use a separate constrained launcher/UID and explicit
capabilities. Current member processes cannot fork; do not relax these ceilings
to launch extensions. Owner CLI export/recovery remains a separate invocation with its original inherited
limits. The worker backend and base OpenSSH retain their own original process
limits; worker rollback keeps the hardened native frontend envelope.

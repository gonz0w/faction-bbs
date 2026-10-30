# Optional durable provider policy

The `durable-policy` feature adds `Runtime::initialize_durable` for a new empty
private root and `Runtime::reopen_durable` for established state. The existing
memory-only runtime remains available. `policy_summary` returns bounded counts and
floors; `policy_matches_home` checks the service's home without exposing records.
The [operator guide](../docs/provider-operator.md) describes the explicit standalone
CLI. Default, native, network and device-client feature graphs do not select
`durable-policy` or `operator` automatically.

The offline operator `prepare` command checks a complete independently approved
proposal against a card and separately supplied client certificate without opening
the runtime or reading retained policy. It preserves all authority fields and makes
a candidate only; separate `init` or `apply` still enforces current floors and
history. See the [operator guide](../docs/provider-operator.md).

Durable and memory-only instances expose the same opaque `Control`. Durable
mutations validate, publish and install under the dispatch mutex, so there is no
in-memory control bypass. Invalid proposals preserve the old validated policy.
An I/O failure or uncertain publication stops the current runtime until an explicit
reopen; do not infer rollback. Stop or drop closes current authority and sessions,
but does not persist unavailability. Reopening creates fresh controls and sessions.

The versioned snapshot is at most 65536 bytes, with at most current and pending
snapshots and a permanent zero-length lock. The exclusive lock prevents concurrent
owners, including offline updates while serving. Private directories are owned
`0700`; files are owned regular `0600`, single-link and opened without following
symlinks. Missing, corrupt, unexpected or unsafe established state is refused,
without bootstrap fallback. The snapshot contains public policy, availability,
generation and reset floors and up to 32 lifetime certificate-to-exact-principal
mappings. It contains no local TLS or OMEMO private keys or session handles.

Revocation, reset and certificate history survive ordinary restart. Reset advances
the epoch and generation and clears all active bindings, leaving every device
unauthorized; retained certificate tombstones and history remain. Reset neither
issues nor recovers keys. To restore selected rightful devices, inspect the retained
floors and apply an explicitly preapproved proposal with a higher generation and a
reset epoch at least the retained floor. A retired certificate cannot be rebound to
a different principal; same-principal reapproval follows existing policy validation. This protects monotonic proposals against the
retained state, not replacement of the whole trusted root with an older valid copy.
There is no external rollback anchor, device enrollment, key recovery or implicit
membership. Retain and inspect uncertain state; do not delete or initialize over it.

The provider's one active connection, eight attempts, five-second TCP/TLS admission,
ten-second cycle, 8192-byte binary payload, 16384-byte IQ payload, 32-entry policy
and eight handles remain unchanged. Filesystem calls are cooperative; the existing
five-second drain is not a guarantee that a stalled filesystem can be preempted.

The separate default-off `delegation-records` feature uses transient local
retrieval policy and records. Its sender-home to audience mappings and records
are not persisted or restored on reopen. Existing registry homes retain their
current durable behavior. Installing durable policy invalidates prior records;
it does not create a remote read grant. Persistence of delegation service-client
registrations, certificates and invitation epochs remains future work under
[issue #300](https://github.com/gonz0w/faction-bbs/issues/300). See the
[retrieval intent contract](retrieval-intents.md).

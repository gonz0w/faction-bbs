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
generation and reset floors and typed lifetime certificate mappings for devices
and services. The aggregate limit is 32 active credentials and 32 history entries
across both kinds, with at most 32 homes. It contains no local TLS or OMEMO
private keys, sessions, observations, records or tokens.

Revocation, reset and certificate history survive ordinary restart. Reset advances
the epoch and generation and clears active device and service bindings, leaving
both unauthorized; retained certificate tombstones and history remain. Reset neither
issues nor recovers keys. To restore selected rightful devices, inspect the retained
floors and apply an explicitly preapproved proposal with a higher generation and a
reset epoch at least the retained floor. A retired certificate cannot be rebound
to a different typed identity; device identity is the exact principal and service
identity is the exact home and client ID. Reapproval of the same identity follows
existing policy validation. This protects monotonic proposals against the retained state, not replacement of the whole trusted root with an older valid copy.
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
it does not create a remote read grant. See the [retrieval intent contract](retrieval-intents.md).

With `delegation-services` explicitly enabled, a combined registry and service
policy update uses the same increasing generation and reset floors. A device-only
registry replacement disables service policy and retains typed certificate history.
Initialization writes version 1 device-only snapshots, and reopen reads them
without rewriting. Before adopting v3, an explicit successful observations-only
update in a service-capable build writes version 2, wrapping the version 1 device
state and retaining local home, service
registrations, invitations, typed history and floors. The version 2 marker remains
after device-only replacement or reset. Builds without `delegation-services`
refuse every version 2 snapshot, including tombstone-only state, without rewrite
or fallback. Observations and sessions never
survive reopen: a service must establish a fresh verified TLS connection. See the
[service observation contract](service-observations.md).

With `delegation-issuer` enabled, a deliberate complete higher-generation issuer
policy update publishes role-aware version 3. Version 1 and 2 reopen without
rewrite; version 2 imports observations only. Unsupported builds refuse active
and tombstone version 3 without fallback or rewrite. Reset or device-only
replacement disables issuer authority while retaining its format marker and typed
history. Live records, references and tokens are never persisted. See the
[issuer contract](delegation-issuer.md).

The optional `PublicationWireProfile` is external runtime configuration, not a
durable permission. It restricts the current installed Observation role by home
and client ID and cannot add a registration or authority. Restart requires the
embedder to supply it again. See the [publication carrier](service-observations.md#optional-publication-http-carrier).

With `delegation-verifier` and durable policy enabled, explicit version 4 stores
resource associations and remote generation/reset floor history in the same
atomic public-policy snapshot. Complete higher-local-generation proposals cannot
lower or forget either retained floor; a wire tuple must equal the configured
tuple, with no automatic adoption. Removal, reset or device-only replacement
disables active authority while retaining history, floors and the v4 marker. An
issuer-only replacement also retains the remote floors.
Versions 1–3 reopen without resource permission or automatic rewrite. Builds that
do not support v4 refuse active and tombstone v4 snapshots without fallback or
rewrite. The 65536-byte bound and whole-trusted-root rollback limitation remain.
No token, pending operation, key or PEM is persisted. V4 supports an issuer-only
or resource-only configuration, not both together; see the
[verifier contract](delegation-verifier.md).

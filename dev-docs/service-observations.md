# Service recipient observations

The default-off `delegation-services` feature gives a trusted embedder a local,
read-only check of recipient approval for an authenticated invited service. It
adds no client or operator enablement, new wire protocol or store access. The
existing device session and the separate service session have different authority:
a service does not become a device actor and cannot create a retrieval intent or
read attachment records.

## Trust and identity

Trusted validated policy names the local recipient home, invitations and service
registrations. A registration specifies the exact service home, client ID, DNS SAN,
leaf certificate pin, enabled state and validity. Configuration is not
authentication: an opaque service session arises only from an actual verified
mutual TLS connection whose observed leaf matches a current registration. A
caller-supplied service identity or certificate description is not a session.

The session shares the existing runtime's single active connection or handshake
lane with device sessions. TCP and TLS share the original five-second admission
budget, the cycle has ten seconds from acceptance and permits eight attempted
operations. Cancellation, stop and runtime drop retain the existing behavior; a
service session cannot retain the runtime or extend its deadline.

## Embedder interface

Enable `delegation-services` explicitly. Construct `services::ServicePolicy::new`
from a local home and validated `ServiceRegistration` entries. Each registration
contains `home`, `client_id`, `dns_san`, `certificate_sha256`, `enabled`,
`valid_from` and `valid_until`. Client IDs are at most 254 printable ASCII bytes
without `*`; DNS SANs use the home-name validation and are at most 189 bytes.
Install the complete registry and service policy together with
`Control::replace_delegation_policy`. Invalid proposals preserve prior policy.

`tls::Server::accept_service` authenticates the actual peer and returns a distinct
`tls::ServiceSession`. Call `observe_recipient(binding, action)` using a validated
binding and `services::PublicationAction`. The returned
`RecipientCurrentObservation` has read-only getters for the binding, action,
service home and client ID, service and recipient certificate digests, generation,
reset epoch and check time. Compare the complete returned values to the intended
operation; the observation is not store or device authority. A successful policy
installation invalidates earlier sessions, even when the policy data is unchanged.

The provider carrier remains TLS 1.3 only. This interface adds no HTTP, OAuth or
introspection endpoint and does not qualify the TLS 1.2 support required by
RFC 7662 for a future introspection backchannel.

## What a check means

An observation checks the complete exact attachment binding and one intended
operation: `Begin`, `PutChunk` with offset, length and ciphertext digest,
`Commit`, sender `Status`, or `Delete`. `GetChunk` is not an observation action.
The authenticated service's exact sender home must match the binding and current
invitations; the local recipient home must also be currently invited. The recipient must be local, with the exact account, device
and full identity currently approved against its registered certificate in the
recipient registry. A foreign sender does not need local device enrollment.

A successful observation reports approval at the time of the check. It does not
prove present possession of a recipient's encryption identity key, current sender
device authority, object commitment, or a remote grant. It conveys no actor,
read, upload or mutation authority. A later operation needs its own current
authority and exact checks; revocation cannot undo an already completed check.
A denial or unavailable policy must not trigger alternate routing.

## Policy and persistence

Device registry and service policy updates share the existing increasing
generation and reset floors. A device-only registry replacement disables service
policy, while retaining immutable typed certificate history. Across device and
service identities there are at most 32 active credentials and 32 lifetime
history entries, and at most 32 homes. Reset clears both active kinds and retains
history. Invalid proposals leave the previous state intact; uncertain durable
publication stops the runtime until explicit reopen.

Durable initialization writes version 1 device-only state, and reopen reads it
without a rewrite. Before adopting v3, an explicit successful observations-only
policy update in a service-capable build writes version 2, wrapping the version 1
device state with local home,
service registrations and typed history. Existing registry homes and floors
remain durable. The version 2 marker remains after device-only replacement or
reset. Builds without `delegation-services` refuse every version 2 state, including
tombstone-only state, without rewrite or fallback. The existing 65536-byte snapshot bound, current and pending snapshots,
permanent lock and private ownership rules remain. Sessions, observations,
retrieval records, tokens, private keys and transient retrieval audience mappings
are not persisted. Reopen requires fresh TLS. Monotonic checks protect retained
state, not rollback of the entire trusted root to an older valid copy.

This observation is not issuer, verifier or relay authority. The separately scoped
[issuer](delegation-issuer.md) uses explicit service roles; cross-home delivery
remains outside this primitive under [issue #300](https://github.com/gonz0w/faction-bbs/issues/300)
and [issue #301](https://github.com/gonz0w/faction-bbs/issues/301). See the
[provider integration](provider-integration.md), [durable policy](provider-policy.md)
and [delegation research](research/0063-invited-home-delegation.md) for their
respective boundaries.

## Software validation

The [source-bound paired evidence](evidence/service-observations/README.md) records
actual host/NetBSD policy and recovery controls, host client regressions, separate
build/runtime envelopes and retained failures. It establishes no remote grant or
supported hardware capacity.

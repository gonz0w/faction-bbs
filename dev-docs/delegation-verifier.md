# Delegated retrieval verifier for embedders

The default-off `delegation-verifier` feature adds a consuming library operation
for retrieval from already committed ciphertext. It starts no listener and enables
no client or operator. It does not authorize foreign publication, carry a response
on the incoming relay connection, or release mail. Device encryption keys and
plaintext remain on the device.

## Configure trusted policy

Construct `ServicePolicy::new_resource(local_home, entries, associations)` with
complete `(ServiceRegistration, ServiceUse)` entries and `ResourceAssociation`
values, then install the complete registry and policy through the existing control.
Each association names the recipient home, issuer, audience, relay client ID,
outbound introspection client ID, endpoint registration ID, fixed socket address,
HTTP host, CA DER SHA-256, and exact remote generation and reset. The local
resource home comes from policy; the endpoint registration supplies server name
and leaf pin. HTTP Host must equal the issuer, while recipient home, issuer/Host
and TLS server name may differ under explicit trusted configuration. The endpoint
ID is an internal registration identity, not a wire client ID. The v4 issuer and
HTTP host allow up to 253 bytes; local and recipient homes and the TLS server
name retain their existing home-name bounds. Legacy Observation client IDs retain
their existing grammar.

`ResourceClient` permits an authenticated foreign relay to prepare retrieval;
`OutboundIntrospection` is a local outgoing credential and `IssuerEndpoint` is
foreign server trust. The latter two cannot authenticate an incoming service
session. The external `HttpEndpoint` must match the configured address, host,
server name, pin, client certificate and CA before connection. Key material is
external and never persisted in policy. Device and service credentials share the
existing aggregate 32 active and 32 history limits; registry homes and retained
remote-floor homes share the 32-home limit.

Current policy supports a resource configuration with legacy Observation entries,
or an issuer configuration with its existing roles. It does not combine issuer
and resource configurations in one policy. Future integration must compose them
within the same existing runtime and limits; adding a second runtime is not a
workaround. See [durable policy](provider-policy.md) for persistence and floors.

## Prepare and execute

An actual authenticated `ServiceSession` with current `ResourceClient` permission
calls `prepare_retrieval(self, binding, request, token)` with a `Binding`,
`Request` and `UnverifiedToken`. It consumes and drops the
entire incoming TLS session before returning an opaque `PendingRetrieval`. Only
recipient `GetChunk` at the exact offset, including valid EOF, or recipient `Status`
can be prepared. The continuation is not a reusable grant and exposes no public
constructor, clone or bearer logging interface.

`PendingRetrieval::execute(self, &HttpEndpoint, &Mutex<Store>)` consumes it,
returns a `Result<Response, Error>`, and
performs fresh authenticated introspection. It checks the exact binding, action,
offset, principals, issuer, audience, lifetime, observed incoming relay certificate
and configured remote generation/reset, along with current local policy. The
response client ID identifies the incoming relay; the request client ID identifies
the separate outgoing introspector. Local sender approval is checked against the
current registry principal, digest and validity interval; this does not attest a
fresh sender TLS connection or private key.

No Authority or Store lock or exclusive Store borrow spans network I/O. Afterward,
the operation takes Store then Authority, rechecks current authority and privately
dispatches the exact immutable request against the actual stored binding. Only
committed-object recipient retrieval can return a bounded Store response to the
trusted embedder. Copied metadata supplies no public Store access. A retry requires
fresh introspection and remains within the original deadline and attempt accounting;
cancellation and stop still apply.

One delegated continuation may be outstanding. Its physical reservation counts
within the existing eight-slot aggregate and survives invalidation, expiry and
reset until the outbound operation and socket finish. A second preparation is Busy
while it remains held; an older release cannot free a newer reservation. Ordinary
provider admission retains its existing limits.

## Scope of component validation

The [host and NetBSD software record](evidence/delegation-verifier/README.md)
includes a two-home component test using a test-only, exact binding/sender/setup
Access to create an object through normal Begin, PutChunk and Commit, followed by
a Store reopen. It is not used for retrieval. The resource home has only its local sender
device; its ordinary session still denies foreign publication. Actual recipient
home issuance and fresh introspection then precede private retrieval. Scheduling
controls for held callbacks are not evidence of TLS kernel saturation. GitHub CI and merge remain separate workflow gates.

This component does not complete foreign-publication authorization, native relay,
production-console cross-home upload or mail delivery. Those remain under
[issue #300](https://github.com/gonz0w/faction-bbs/issues/300) and
[issue #301](https://github.com/gonz0w/faction-bbs/issues/301). Deployment,
independent interoperability and physical capacity are not established here.

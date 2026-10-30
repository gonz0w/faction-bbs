# Recipient retrieval intents

The optional `delegation-records` feature provides a transient, local record of
what an approved recipient device intends to retrieve. It is off by default. It
adds no client or operator enablement and does not authorize a store read. This
interface is for a trusted embedding application that owns the provider runtime
and configures its policy; it is not a wire protocol.

## Trust and scope

Configure a validated retrieval policy with one exact local recipient home and
at most 32 unique invited sender-home to exact service-audience mappings. Treat
this configuration as trusted local policy, not as authentication. The recipient
comes from the verified device TLS session, the sender home from the validated
binding, and the audience from the matching policy entry. Request text cannot
select an actor or audience. A foreign sender need not be enrolled in the local
recipient registry merely to designate an intent. Existing registry and store
access checks are unchanged.

Only a current, approved device TLS session can create an opaque record. Its
factory charges the session's existing operation attempt before validation. The
record binds the entire exact binding and either `GetChunk` with its exact offset
or `Status`, together with the derived recipient, certificate, policy identity,
generation and reset state. It exposes no session actor handle, stream or store access capability; the
checked binding does expose the recipient and sender metadata. A trusted runtime owner can check the
record and inspect its read-only intent data. Compare the complete returned
binding, action and offset, and derived audience to the intended operation; do
not treat an approximate match or request-supplied value as authority.

An intent does not prove that a foreign sender is currently approved or that an
object is committed. The future sender-side authority and store integration must
independently authenticate the issuer and audience and check both current policy
and the exact committed object and action. The record alone supplies no HTTP
endpoint, relay, callback or complete cross-home path. The separate default-off
[issuer](delegation-issuer.md) can provision a reference from the record after its
exact originating session ownership ends. See [the delegation research](research/0063-invited-home-delegation.md)
and [issue #300](https://github.com/gonz0w/faction-bbs/issues/300) and
[issue #301](https://github.com/gonz0w/faction-bbs/issues/301).

## Embedder interface

Enable `delegation-records` explicitly on `faction-provider`. Construct
`retrieval::RetrievalPolicy::new(local_home, sender_audiences)` from trusted
configuration, then install it with `Control::replace_retrieval_policy`. Each
mapping contains a sender home and its exact service audience. An audience is
bounded to 254 printable ASCII characters without whitespace, control characters
or `*`; home names use principal validation. Duplicate sender homes or invalid
configuration are refused.

On a verified `tls::Session`, call `retrieval_intent(binding, action)` with a
validated `private_blobs::Binding` and `retrieval::RetrievalAction::Status` or
`GetChunk { offset }`, where the offset is at most the binding size. Keep the opaque `RetrievalRecord` for the operation. Once
the originating session has closed, the runtime owner calls
`Control::check_retrieval(&record, &binding, action, audience)` with the exact
expected values. A successful `CheckedIntent` offers `binding()`, `action()` and
`audience()` getters for read-only inspection. Check every value against the
operation being considered, and handle an error as denial. Neither opaque type
has a public constructor or field access.

## Lifetime and failure

Dropping the originating session closes its socket, invalidates its handle and
releases the runtime's single admission lane. The separate record may survive
that normal closure, but only for the remaining original absolute ten-second
cycle and no later than certificate or binding expiry. It does not keep the
runtime alive, retain the connection or handle, or start a new cycle. Exact
repeated checks neither consume nor renew it.

At most eight live records exist in aggregate across sessions and cycles. Expired
slots are pruned; a ninth live record fails busy immediately, with no queue or
renewal. Capacity exhaustion returns `Error::Store(private_blobs::Error::Busy)`;
the existing session attempt limit returns `Error::Transport(tls::Stage::Limit)`.
Each check revalidates current recipient and certificate, invited homes,
policy identity, generation and reset, runtime state, the absolute monotonic
deadline and the whole-second wall-clock high-water mark. Expiry and detected
clock rollback are sticky:
recovery of the clock cannot revive an invalidated record. Registry, combined
service policy or retrieval policy replacement, unavailable state, reset, stop, runtime drop and durable installation
invalidate prior records. An invalid typed policy replacement preserves the
previous valid state. Records and the retrieval policy's sender-home to audience mappings are not
persisted in durable snapshots and cannot survive reopen or transfer to another
runtime. Existing registry homes retain their current durable behavior.

A denial, expiry, busy result or unavailable authority is not a grant and must
not trigger alternate routing. Existing one-connection, eight-attempt, admission,
cycle, policy and store limits continue to apply. See the
[provider integration](provider-integration.md) and
[durable policy](provider-policy.md) contracts for the surrounding lifecycle.

## Software validation

The [source-bound host and NetBSD record](evidence/retrieval-intents/README.md)
separates the local intent controls, ordinary regressions, build/runtime envelopes
and retained failures. It does not establish remote delivery or supported capacity.

# Delegation wire codecs

[`faction-delegation-codec`](../crates/faction-delegation-codec/README.md) is an
isolated, unpublished codec for a fixed Faction wire profile. Its separate Cargo
workspace and lockfile add no dependency to the root, provider or client product
graphs, including classic and feature-off configurations. It provides no listener,
token issuer or generator, registry, Access or Session conversion, store, network,
key handling or service deployment. Native end-to-end relay belongs to
[issue #301](https://github.com/gonz0w/faction-bbs/issues/301); the broader
authority work belongs to [issue #300](https://github.com/gonz0w/faction-bbs/issues/300).

## Trust boundary

Every decoded type is **unverified**. Parsing checks a bounded syntax and Faction
application schema, not TLS possession, certificate authentication, device approval,
registration, invitation, current policy, audience or resource authority. Public
claim fields remain editable and untrusted; encoders revalidate the schema. There
is no clock or registry check. `client_id` in a form is an assertion, not an
authenticated identity, and matching it to a registered TLS client remains a
separate responsibility.

`UnverifiedToken` and types containing it have no `Debug` or `Display` implementation.
Its explicit `expose()` borrows sensitive bearer text; the caller must protect it.
Encoded messages that contain it are sensitive too. The opaque operation reference in `scope` is sensitive too. Fixed
errors and refusal bodies do not echo input. `UnverifiedReference::parse` accepts
canonical unpadded base64url encoding of exactly 32 bytes; its use for
`cnf.x5t#S256` checks spelling only, not a certificate digest or possession.

## Wire profile

The exported `encode_*` and `decode_*` functions cover token and introspection
forms, token and introspection JSON responses, and complete HTTP request and
response messages. `encode_refusal_http` emits fixed refusal responses. RFC 6749
supplies the token form and response fields; this profile uses `client_credentials`
and maps `scope` to one opaque server-operation reference. RFC 7662 supplies
introspection fields and RFC 8705 defines the certificate-thumbprint confirmation
field. `faction_*` fields, binding shapes, action separation and all limits are
Faction application restrictions, not general OAuth requirements.

The current fixed source profile represents `iss` and `faction_issuer_home` as
canonical lowercase home DNS names and `aud` as a canonical home DNS name followed
by `/attachments`. Active introspection uses separate variants:

- `Retrieval` has the recipient account as subject and supports `GetChunk { offset }`
  and `Status`.
- `RecipientCurrent` has the asserted client ID as service subject and supports
  `Publish`, `Status` and `Delete` policy observations.
- `Inactive` is the distinct `{"active":false}` response, with no extra fields.

Neither active variant grants a capability. The schema also bounds claim lifetimes,
binding size and lifetime, and chunk offsets; see the [types](../crates/faction-delegation-codec/src/types.rs)
and [JSON codec](../crates/faction-delegation-codec/src/json.rs) for the precise
checks. Token response lifetimes are 1–10 seconds. The form and JSON decoders reject
unknown and duplicate fields; JSON rejects arrays, null and non-unsigned numeric
values. Limits are 8,192 bytes each for HTTP headers, form bodies and JSON bodies,
256 bytes for token text, 16 HTTP headers, 24 form fields or fields per JSON object,
and JSON nesting depth 4. These are syntax and allocation bounds, not a validation
of authority.

HTTP decoding takes one complete HTTP/1.1 message with CRLF framing, a required
Content-Length matching the entire body, and the expected content type. Requests
are POSTs to `/oauth/token` or `/oauth/introspect` with a canonical host; decoded
responses have status 200. The profile does not support chunking, compression,
redirects or pipelining. It is not a streaming HTTP implementation.

## Source qualification before build

Use Rust 1.96 and the current [documentation CI workflow](../.github/workflows/documentation.yml)
as the command authority. Before compiling, warm the locked metadata graphs in its
“Validate the isolated delegation wire codec” step for all three listed targets:
the codec, root `faction-client/device-provider`, and provider `operator` graphs.
Fresh caches need all of these, not only the codec graph. Metadata warming fetches
source; it does not authorize compiling it. Then run the offline source gate:

```sh
python3 tools/check_delegation_codec_dependencies.py
```

Only after the gate passes, run that step's `cargo fmt`, offline locked release
`cargo clippy` and `cargo test` commands, with two build jobs and a private
`CARGO_TARGET_DIR` (as in CI). Do not execute upstream tests or CI. The
[dependency inventory](dependencies/delegation-codec.json) and
[checker](../tools/check_delegation_codec_dependencies.py) qualify selected source,
licenses, features and no-JavaScript boundaries, including isolation from product
graphs. Inactive lockfile packages are distinct from the selected source graph.
This gate is an integrity check, not a security audit or proof of native execution.
The [qualification record](evidence/delegation-codecs/README.md) documents the
separate source, host and native codec checks and their limits.

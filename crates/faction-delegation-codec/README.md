# Faction delegation wire codec

This standalone, unpublished crate encodes and decodes a fixed Faction application
profile of OAuth token and introspection wire data. It has its own workspace and
lockfile; it is not a dependency of the root, provider or client product graphs.
It is not a general OAuth library. See the [developer contract](../../dev-docs/delegation-codecs.md)
for the supported shapes, limits and source qualification procedure.

**Every decoded value is unverified.** Syntax acceptance establishes no TLS or
certificate possession, device approval, registration, invitation, current policy,
audience or resource authority. This crate does not issue or generate tokens,
authenticate clients, consult a registry or store, handle keys, make network calls,
run a listener, deploy a service, or produce an Access or Session capability.

`UnverifiedToken` and types containing it deliberately omit `Debug` and `Display`.
Calling `expose()` borrows sensitive bearer text; the caller must protect it.
Encoded forms and HTTP messages containing it are sensitive too. The opaque scope reference is also sensitive.
Errors and fixed refusal bodies do not echo input.

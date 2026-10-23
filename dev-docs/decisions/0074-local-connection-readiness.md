# 0074: Local readiness before active connection diagnostics

Status: accepted bounded implementation
Date: 2026-10-23
Issue: [#158](https://github.com/gonz0w/faction-bbs/issues/158)

## Decision

Add a device-local connection desk to the existing native launcher and a redacted
plain-text CLI. Separate strict profile parsing from the readiness of its existing
referenced files, while preserving Profile::load's complete checks at connect.
Do not infer key unlock, live host identity, SSH admission or BBS approval from
file readability. Do not invent verified-contact ages or invitation state.

Use base OpenSSH's local ssh-keygen lookup for explicit stored-pin inspection,
including hashed entries, rather than implementing host-pattern/HMAC matching or
opening an actual SSH session as a "check". A checked file descriptor, fixed
arguments/environment, bounded output/deadline and guaranteed reap constrain this
one transient member-device helper. Render only canonical SHA256 fingerprints
and fixed messages; CA/revoked entries remain OpenSSH policy, not positive trust.
No new crate, server helper, network request or disk record is added.

The share preview/plain report omit all contact and key identifiers. Users choose
whether to copy it. The ANSI picker gets paged diagnostics while ordinary SSH and
a plain report remain useful. Keep #158 open for verified contact provenance,
route/invitation adapters and deliberate live probes with their own contracts.

## Alternatives and consequences

A live SSH probe could answer more but needs explicit remote-side effects,
credential/key-unlock semantics and cancellation/interface design. It is deferred.
A home-grown known-host matcher would duplicate OpenSSH trust policy and is
rejected. Static file checks alone cannot show stored expected fingerprints;
the optional base helper supplies local lookup without adopting trust authority.
The native NetBSD/base OpenSSH interface must be exercised, not inferred from
host OS behavior. No supported release/capacity/hardening claim follows.

The change is easy to reverse: no schema, dependencies or stored format change.
Implementation/validation evidence and limits are in plan 0079 and the user guide.

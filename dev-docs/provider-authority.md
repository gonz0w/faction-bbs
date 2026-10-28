# Experimental attachment provider authority

The isolated [provider-auth experiment](../experiments/270-provider-auth/README.md)
examines how a verified TLS peer can supply authority to the existing ciphertext
store. It is outside the product workspace and is not enabled by ordinary SSH or
native home-mail defaults. Current network mail retains its existing PLAIN profile;
there is no provider certificate enrollment here. The earlier
[boundary research](research/0060-provider-device-boundary.md) explains the choices.

## Authority and identity

A typed, offline synthetic registry binds the SHA-256 digest of exact certificate
DER to one validated `Principal`: account, nonzero device and full 32-byte public
identity. Approval, account/device status, validity interval, generation and invited
homes are explicit. The registry accepts at most 32 bindings and 32 homes. It rejects
malformed or ambiguous mappings. An invalid typed replacement leaves the last
validated snapshot intact; an unavailable external authority must be explicitly
marked unavailable. There is no registry file loader or production enrollment.

Each `Authority` retains at most 32 lifetime digest-to-principal tombstones. A
retired certificate cannot be remapped to another principal in that instance;
explicit reapproval of the same principal is distinct. A fresh instance loses this
in-memory history, so it provides no durable anti-rollback. Reset clears active
bindings and handles, not device keys. Password reset creates no binding and
recovers no keys.

The crate-private authentication entry point is reserved for a trusted TLS adapter.
This is API privacy, not a process sandbox. The adapter derives the actor from a
verified peer certificate, never from request text or a wire-selected principal.
Opaque handles belong to a connection and authority instance, with at most eight
live handles. Every store dispatch rechecks current generation, validity, approval,
account, device, full identity and relevant sender, recipient and invited-peer
policy. Revocation, rotation or reset affects the next operation. Copying expected
public metadata grants no authority. This mapping does not prove possession of an
OMEMO private identity key, change peer trust, or replace independent BBS approval.

## TLS fixture contract

The synthetic loopback fixture is designed for one active connection, a handshake
admission limit of five seconds including the wait for TCP and handshake, and an
absolute ten-second cycle from connection acceptance including the handshake. It
allows at most eight attempted requests and the existing binary codec capped at
8192 bytes.
It requires client certificate verification and checks the server CA, name and
exact leaf pin; leaf validity is checked on each dispatch. It disables 0-RTT,
tickets and resumption. Synthetic certificate keys are held in memory. Actual
runtime observations and their scope belong in the
[host evidence](evidence/provider-authority/README.md) and
[native NetBSD evidence](evidence/provider-authority-netbsd/README.md), not in this contract.
The deadline is cooperative at the filesystem boundary and cannot preempt a
blocked filesystem operation.

Denied requests do not mutate the store. Authorized status or expiry observations
can advance its durable high-water clock. A lost response after dispatch or
`PublicationUncertain` does not imply rollback: reopen and use the exact retry to
resolve the outcome. The existing [storage contract](private-attachment-storage.md)
retains its object, byte, chunk and retention limits.

This experiment does not implement a provider, own-home delegated transport,
independent client interoperability or hardware attestation. The native NetBSD
serial and default-parallel runs are bounded software evidence, not provider or production
support. Mail and attachment private keys and decryption remain on the member device. The ordinary classic, SSH and UI paths retain their existing limits.

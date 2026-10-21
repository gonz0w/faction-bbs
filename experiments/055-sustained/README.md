# Sustained workload fixture

Private software VM only; see [research 0037](../../dev-docs/research/0037-sustained-abuse.md).
The new reset variant recognizes only the existing synthetic UI rows and bounded
`sustained synthetic` posts (at most 100 additional total posts). It refuses
unknown rows, requires the stopped marked owner fixture, retains credential hashes
and checks exact baseline counts/integrity. Never use it on an actual board.
The older reset source/evidence remains unchanged.

The unsigned-key probe additionally uses a private developer-host venv with
Paramiko 4.0.0. Resolved dependency versions and installed Python/native-file hashes
are in result 038; this is not a product/server/CI dependency. No private key is
needed by that probe; its offered fixture public key is never published.

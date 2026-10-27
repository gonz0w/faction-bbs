# Experimental base-SSH key admission

Native NetBSD-only AuthorizedKeysCommand under a dedicated noninteractive UID.
Fixed private process-birth reservations and global token budget; no resident
process, socket/SSH parser or key-signature implementation. Base OpenSSH verifies
keys/options. Missing state fails closed; root initialization requires quiescence.

See [the operator contract](../../docs/experimental-admission.md),
[ADR 0022](../../dev-docs/decisions/0022-ssh-admission.md) and
[research/native evidence](../../dev-docs/research/0036-ssh-admission.md).
This bounds key-admission/authenticated monitors, not all TCP transports/forks.
Default eight; experimental 250 is a short software result, not production support.

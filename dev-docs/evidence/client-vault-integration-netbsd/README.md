# Current native client and local-vault integration

This record validates the reconciled native client and optional local vault entry
from the member-owned board picker on macOS ARM64 and NetBSD 11 ARM64 software.
See [plan 0063](../../plans/0063-current-client-and-local-vault.md) and
[ADR 0058](../../decisions/0058-local-vault-from-board-picker.md).

The preceding member-page record and original standalone vault record remain
unchanged. Their exact inputs are preserved in the two pre94 source snapshots;
this record has its own current input pins. It does not refresh historical hashes.

The new picker runs local unlock only with no active SSH child; its store comes
from the private member catalog. Drafts stay encrypted and explicitly unsent.
Initialization/recovery remain separate local commands. Existing vault format,
trust/pin rules, device-only secrets and local approval remain unchanged. The
NetBSD VM has two virtual CPUs and 1 GiB RAM, but no memory/capacity benchmark or
physical hardware claim follows from these functional checks.

The private operator playground remains stopped and retained. Native validation
uses a different marked copy-on-write VM and generated temporary credentials,
profiles and stores. Only that owned test VM/cache is removed at completion.
There is no public demo, real home enrollment, network inbox/send/OMEMO adapter,
roaming authentication, encrypted attachment implementation or new hardening claim.

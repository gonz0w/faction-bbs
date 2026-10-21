# Scoped extension contracts

Status: architecture requirements/proposed interfaces, not a shipped generic
plugin SDK or dynamic ABI. Owner direction 2026-10-21; #5/#10/#81/#83/#84.
Existing Lua and classic connectors provide specific interfaces already; these
requirements guide additions without widening those existing privileges.

| Capability | Input / result shape | Authority kept by the host |
| --- | --- | --- |
| Terminal transport | Admitted terminal/events and bounded rendered output | Session identity, approval and actions; transport never imports a role |
| Identity provider | Verified full principal and scoped expiring proof | Local account binding/approval, role, epoch and grants |
| File transfer | One grant: key/member/epoch/area/object/direction/bytes/expiry | Catalog/review/quota and current approval; adapter cannot choose arbitrary paths |
| Encrypted mailbox | Bounded encrypted envelopes and delivery states | Recipient policy/retention; member device alone holds decryption keys |
| Mail network | Addressed ciphertext, peer/domain validation and bounded outcomes | Explicit outbound consent, local membership and device trust |
| Shared chat | Explicit channel subscription and checked author/delivery context | Local-only channels and local roles/content |
| Door/game host | Declared terminal capability, bounded input/output and lifecycle | BBS credentials/state, resource ceilings and local return |
| Theme/Lua menu | Validated presentation/action proposals | Allowed actions and authorization; no general shell/DB/keys |

First add only contracts needed for SFTP and the selected mail proof. Do not build
an all-purpose plugin framework or native shared-library loader before useful
features exist. Keep semantic operations separate from network locators, storage
formats and terminal layouts. Rust interfaces can express local seams; versioned
bounded IPC with peer identity checks is required for separate privileged/native/
network helpers. Third-party native code must not run in the BBS owner process.

Each service protocol needs an explicit version/feature negotiation, length and
field caps, unknown-operation refusal, operation/request IDs, deadlines, bounded
queues/concurrency, cancellation, current-authority checks and terminal-safe
responses. Distinguish pending/committed/completed outcomes and idempotency scope;
never promise exactly-once external delivery from a request ID. On failure, fail
only the optional operation and keep useful local BBS functions available.

Capabilities are separate: verify-home-identity, operate-one-file-transfer,
route-addressed-ciphertext, participate-in-one-chat-channel, public publication,
payment request and spend/sign. Permission to route mail is not permission to
read local room/file data or private keys. Metadata and error feedback must also
respect scope. No wildcard provider-selected destination, ambient credential or
unreviewed callback into the database. Removing an adapter must leave readable
local state and an explicit backup/migration path.

Game hosts are **not DOS-only**. Preserve adapters for other original DOS titles,
Amiga or other emulators, native original servers and future supported runtimes.
Each must declare its input/output capability and have separate storage/identity,
startup/cleanup, legal artifact provenance and measured host costs. A title with
serial/terminal control is a plausible ANSI door. A GUI-only title needs an
additional interaction/rendering design or remains unsupported; an emulator does
not automatically convert it to a usable ANSI game. Do not bypass safe terminal
parsing or start arbitrary emulators under factiond/factionterm/Lua. #81 owns the
maintained host contract; concrete platforms/titles need independent reuse tests.

The first [native Files control](native-file-control.md) is a narrow versioned
terminal seam layered over the existing file grant/SFTP policy. It carries no
remote endpoints, local paths, mail keys or home identity. Stable local references
are independent of transfer adapters. The native client can later host a separate
mail presentation/keystore boundary, but #84/#88 must first prove actual encrypted
messages/attachments and scoped home trust; terminal control is not an auth protocol.


The [mail/identity proof](research/0045-encrypted-mail-and-roaming-proof.md) keeps
routing/archives, device cryptography/trust, trusted local presentation, online
issuer proof and local membership binding independent. Its Python/SCE reference
adapter is not a product extension runtime. Current terminal file-control frames
must never solicit home secrets, unlock a mail keystore or compose private mail.
Native mail needs an explicitly local trusted interaction (#90); verified home
identity needs a trusted connection-bound membership path (#91), with no role
or approval import. Keep attachments under #88's ciphertext/key boundary.

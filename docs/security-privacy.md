# Privacy and security requirements

Status: mandatory design constraints and open research. Narrow local authorization/credential behavior is implemented/tested in [ADR 0008](../dev-docs/decisions/0008-local-core-and-credentials.md); this is not a complete security or remote-deployment claim. Report sensitive findings through [SECURITY.md](../SECURITY.md).

## Membership and authority

- Authenticate users and require local sysop approval before every community action.
- Define registration, pending, approval, rejection, suspension/revocation, recovery, and operator bootstrap explicitly.
- Separate member, moderator, sysop, extension, and remote-channel privileges. A local directory entry grants no remote trust or access.
- Preserve local account and approval authority when shared chat is introduced. Do not federate account databases.

## Exposure and privacy

- A board must function without a public HTTP service, public listing, central service, or federation.
- An optional HTTP portal may be private or disabled; bind/listen/discovery behavior needs secure defaults and testing.
- Ordinary SSH client access with optional private-network mode is accepted direction; transport/network admission never substitutes for local BBS approval. Evaluate the SSH backend, registration/bootstrap, host identity and deny shell/exec/SFTP/forwarding escape paths. Compare any legacy Telnet compatibility separately. Treat the protocol choice as a security decision; no public Telnet deployment is selected or recommended by current docs.
- Minimize collected registration information, operational logs, retained addresses, and metadata. Telemetry is minimal and opt-in. Do not log message bodies, passwords, tokens, or private content by default.
- Define retention, export, deletion, backups, restore, and recovery as local operator responsibilities with useful safe defaults to validate.
- An underground/private atmosphere does not establish anonymity, confidentiality, encryption, or resistance to traffic analysis. Record actual guarantees and limitations only after validation.

## Optional extensions and external publication

- Optional helpers must not inherit unrestricted local-room/file, publication or spending access. The host rechecks current identity, approval, room scope and grants for every action; remote signatures/payments never grant local membership.
- A proposed public/permanent publishing room is separate from private local rooms and chat-only BBS federation. Require deliberate exact-content/destination disclosure and author intent; never silently export private history. Remote copies may outlive local moderation/deletion.
- Payment models remain under comparison, including external wallets and separate sysop services. Keep order intent, signing/custody, settlement and entitlements distinct. Current tests use synthetic data/fake providers/nonredeemable credits; no keys or funded operations.
- Lua worker processes/API restrictions are not an OS sandbox. Production doors require restricted OS privileges, bounded resources/concurrency, terminal-safe output and reliable termination/recovery.

## Isolation and untrusted content

- Treat terminal controls, user messages, filenames, uploads, imported records, directory submissions, and federated traffic as untrusted input.
- Prevent terminal escape injection, path traversal, resource exhaustion, unauthorized file access, and approval bypasses.
- Keep legacy doors and extensions optional and isolated, with bounded filesystem/network/process privileges, resource limits, and reliable session cleanup.
- Only participating shared chat channels may federate. Local-only channels, boards, files, mail, games, directories, and account/approval data stay local.
- No JavaScript/TypeScript code or dependencies, including browser-terminal assets and project tooling. Track dependency provenance, licenses, updates, and supply-chain risk for the eventual stack.

## Evidence required

Write the threat model and pass/fail tests before claiming a security baseline. Research must cover authorization across both access surfaces, interrupted and revoked sessions, terminal parsing, file handling, door isolation, default exposure, and shared-channel/local-data separation. No production deployment or supported version exists yet.

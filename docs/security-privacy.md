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

The [local chat baseline](local-chat.md) keeps private grants/joins/history floors
under current local approval/role/epoch checks. Bounded retained messages/receipts
are plaintext application data, not secure deletion or confidentiality from the
operator. A denied channel clears the selected draft; it is never redirected.

[Owner-only export/restore](export-restore.md) includes credential hashes and private
content in a standalone local plaintext snapshot. Private paths, exact schema/data
checks and fresh restored session binding are implemented; encryption, authenticity,
member-scoped disclosure and full installation backups are not. Review approvals
and password resets when restoring older policy. Never expose these owner tools
to a remote member entry point or treat an export as authority to publish data.

## Isolation and untrusted content

- Treat terminal controls, user messages, filenames, uploads, imported records, directory submissions, and federated traffic as untrusted input.
- Prevent terminal escape injection, path traversal, resource exhaustion, unauthorized file access, and approval bypasses.
- Keep legacy doors and extensions optional and isolated, with bounded filesystem/network/process privileges, resource limits, and reliable session cleanup.
- Only participating shared chat channels may federate. Local-only channels, boards, files, mail, games, directories, and account/approval data stay local.
- No JavaScript/TypeScript code or dependencies, including browser-terminal assets and project tooling. Track dependency provenance, licenses, updates, and supply-chain risk for the eventual stack.

The [restricted SSH development gateway](restricted-ssh.md) separates the terminal
UID from database ownership and keeps owner commands out of its remote entry point.
Native member/approval/TTY/escape-denial oracles pass. Socket permissions and TTY
ownership do not authenticate a BBS member; password/status/role/epoch checks do.
Trusted worker/root compromise and same-UID frontend compromise are not sandboxed.
The narrow NetBSD C/FFI bridge has an explicit ownership boundary and native positive
regression; do not describe the whole system as unsafe-free Rust. Keep the fixed
development caps until complete host/failure and remote hardening work is measured.

## Evidence required

Write the threat model and pass/fail tests before claiming a security baseline. Research must cover authorization across both access surfaces, interrupted and revoked sessions, terminal parsing, file handling, door isolation, default exposure, and shared-channel/local-data separation. No production deployment or supported version exists yet.

The [managed service path](service-lifecycle.md) now drains ordinary stops, reaps
workers and detects broker loss; native eight-session overload/fault checks passed.
Its pidfile lock must never be unlinked while live. Native signal handling extends
the explicit FFI exception; it is not a process sandbox or hard realtime guarantee.
Whole-host capacity, account/key fairness and durable failure remain open.

Credential derivation now has two nonblocking cross-process slots per board and
strict stored-cost checks before verification. Busy work is rejected, not queued
without bound; configured Argon2 work is unchanged. These advisory locks assume
trusted owner/root and cooperating workers. They are not hostile-code isolation,
per-account fairness or a whole-host memory guarantee. Higher session limits are
operator-only profiling controls; normal service keeps eight workers.

The [shared-owner experiment](../dev-docs/decisions/0015-shared-board-readiness-experiment.md)
keeps opaque sessions and chat leases inside their owning terminal connection;
member input cannot import an identity/role/capability or invoke owner tools.
Two fixed credential workers still use the kernel-slot budget. A shared backend
has a larger failure domain than separate member workers; completed prototype
checks do not replace full UI, fairness, confinement or production review.

The [allocator/transport experiment](../dev-docs/decisions/0016-native-allocator-comparison.md)
keeps base OpenSSH, private terminal/owner identities, current approval/epochs/
grants and unchanged two-slot Argon2. A short 250-member VM result is not a
production confinement/admission guarantee. The unchanged headroom predicate is
sampled between admissions; saved batch overshoot must not be represented as a
continuously maintained reserve. Guest-wide tuning requires scoped deployment
and broader workload review before adoption.

The [headroom follow-up](../dev-docs/research/0031-repeatable-headroom.md) retains
the unchanged SSH/UID/approval/credential gates. A disposable time-daemon paging
policy is host resource research, not new application confinement. Native tests
record zero swap and successful time-daemon/service presence; sustained clock
precision, per-service setup/rollback and full deployment security remain gates.

The [full shared UI candidate](shared-ui.md) keeps domain sessions and leases
inside their owning connection; cached menu-role presentation never grants an
action. Current core approval/epochs/grants still authorize every action. Native
denial/regrant/kick, forged control, cross-connection, backpressure and crash checks
pass. Two credential permits cover queued/active/result lifetime; output and work
are bounded without holding transactions behind terminal writes. These observations
do not establish filesystem/network/exec isolation or hostile admission safety.
Normal services remain eight; scoped adoption and broader workloads stay open.

## Current native process envelope

The maintained NetBSD shared service/native frontend now authenticate both Unix
socket peers before passing terminal descriptors, sanitize inherited authority,
and apply verified hard resource ceilings. See [the exact operator boundary](process-confinement.md)
and [native evidence](../dev-docs/research/0034-process-confinement.md). These checks
retain local approval and connection-bound authority. They do not isolate a
compromised shared owner from its database, restrict its filesystem/network/exec
syscalls, or isolate hostile code sharing an allowed UID. Stronger isolation,
authenticated SSH transport admission and production review remain open.

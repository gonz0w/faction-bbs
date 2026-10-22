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
- A proposed public/permanent publishing room is separate from private local rooms and the explicitly scoped chat/mail/identity federation. Require deliberate exact-content/destination disclosure and author intent; never silently export private history. Remote copies may outlive local moderation/deletion.
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
- Future external capabilities are explicit shared chat, addressed device-encrypted mail and scoped home-identity proofs (ADR 0031). Local-only channels, local rooms/community files, games, directories and account/approval stores stay local; home mailboxes are not copied into visited boards. #101 validates NNTP-first explicitly shared rooms; #103 plans Netnews article representation and #104 optional providers. Formatting grants no export authority; IRC/XMPP keep their formats. No production wire runtime or automatic publication is adopted.
- No JavaScript/TypeScript code or dependencies, including browser-terminal assets and project tooling. Track dependency provenance, licenses, updates, and supply-chain risk for the eventual stack.

The [restricted SSH development gateway](restricted-ssh.md) separates the terminal
UID from database ownership and keeps owner commands out of its remote entry point.
Native member/approval/TTY/escape-denial oracles pass. Socket permissions and TTY
ownership do not authenticate a BBS member; password/status/role/epoch checks do.
Trusted worker/root compromise and same-UID frontend compromise are not sandboxed.
The narrow NetBSD C/FFI bridge has an explicit ownership boundary and native positive
regression; do not describe the whole system as unsafe-free Rust. Keep the fixed
development caps until complete host/failure and remote hardening work is measured.

The optional [Lua arcade](lua-doors.md) retains BBS-local personal bests and run
counts. The host binds a one-shot run to the authenticated member/board/epoch and
rechecks approval at completion and score reads. Lua receives no identity/DB
handle. Scoreboards show approved members only; retained rows for revoked members
remain in private owner exports. Root-installed game code controls scoring and
can cheat, so scores grant no privileges, currency or payment entitlement.

The optional [classic game connector](classic-game-doors.md) opens only trusted
configuration literal endpoints after approved-member destination confirmation.
It sends no automatic account/session/credential data. Member-typed game traffic
crosses a legacy unencrypted hop, requiring a separately protected sysop-managed
path. The original game executes on its own server/compatibility host; bounded
Telnet/terminal parsing runs in the shared owner. This adds protocol surface,
not local native-game execution or a parser-process sandbox. Local escape, current
approval, terminal-control filtering, byte/queue/socket limits and cleanup remain
mandatory. External game saves/scores never mutate local board state.

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

The separate [experimental filesystem/exec and IP profile](experimental-isolation.md)
adds a minimal noexec root, permanent pre-main privilege drop and explicit NPF
group/user policy. Its frontend alone uses setuid root and needs independent
review before production adoption. Setup, host policy, loader/runtime trust and
rollback are explicit. Defaults/core authorization/credentials/resource ceilings
are unchanged; broader same-UID/process/all-family/admission threats stay open.

## Experimental SSH key admission

The [key-admission profile](experimental-admission.md) adds a separate trusted
helper UID and private fixed process-birth registry around base key lookup.
Missing/corrupt authority fails closed; runtime initialization must prove
quiescence, and listener restart cannot erase old authenticated reservations.
No key-file fallback, new SSH parser/crypto or BBS approval shortcut is added.
The default is eight reservations; base preauthentication controls remain separate.
Source penalties/NAT fairness, pre-fork/lookup CPU pressure, default process
visibility, independent review and complete active/soak budgets remain explicit
limits. Do not turn short 250-member VM passes into production capacity or privacy
sandbox claims. Never reuse the helper UID for doors or weaken member ceilings.

The [bounded lookup update](../dev-docs/decisions/0023-bounded-key-lookups.md)
charges four helper allowances per monitor birth, denies exhausted calls without
key/state IO or cohort scans, and requires explicit quiescent migration of legacy
state. Ordinary SSH signature/options and local BBS approval remain independent.
The finite cohort budget does not bound all new TCP/fork/KEX activity or guarantee
fresh-member availability behind a penalized source. Helper UID remains trusted,
private and unavailable to extensions; independent review/adoption gates remain.

The [global SSH admission candidate](experimental-global-admission.md) adds a
separately trusted, listener-only native library and persistent root-only budget.
It is an experiment requiring independent privileged-loader review; native source
penalties remain unchanged. Private tunnel keys/inner addresses are an optional
network-admission identity, separate from SSH invitation and local BBS approval.
No forwarded address claim or shared NAT establishes a member identity.

## Selected encrypted-mail and file-transfer requirements

The owner rejected server-readable private mail on 2026-10-21 and selected a
native local client. Device keys and plaintext stay on member devices; an SSH
server decrypting/rendering mail fails this requirement. Keep verified device
enrollment, local keystore unlock, account authentication and local-board approval
separate. Home password reset must not recover mail keys or silently add a trusted
device. No plaintext fallback. Necessary routing/time/size metadata remains visible;
new-device history/recovery and malicious-home key substitution require proof.
See [private mail](private-mail.md); no implementation claim.

SFTP file areas use a separate restricted service/UID and scoped grants, not a
new subsystem on the terminal-only BBS listener. Current local approval, epoch,
area, operation and client-key checks remain mandatory. Files stay local; partial
uploads remain quota-charged/private until review and atomic publication. Current
database exports include catalog policy but no file bytes and discard transfer
grants. Use the stopped/drained complete catalog/blob backup for file recovery. See [file areas](file-areas.md).

## Native client and file-reference boundary

The optional [native client](native-client.md) runs on the member device. A private
local profile selects SSH/SFTP destinations and verified known-host keys. Base
OpenSSH provides encryption/admission; local BBS login/approval remains separate.
Remote terminal output is untrusted: filter terminal controls, decode only bounded
file-control frames, and require local keyboard actions/path selection/confirmation
before transfer. No remote-selected local path, endpoint, shell or agent forwarding.
The Files session uses current connection authority, nonce and sequence, not a
member/role asserted by the client. Client-only PTY/C code is outside the BBS owner.

File references are public identifiers, not bearer grants. Sharing, per-reader
metadata rendering and actual download each enforce current approval and area
access. Arbitrary filenames/metadata typed as normal prose remain ordinary post
content; reference redaction cannot retract that prose or an earlier authorized
reader's copy. Public reference namespaces survive restore; private session identity
rotates and transfer grants are discarded. No link automatically fetches from a
remote BBS or publishes local files. Current file areas are readable by the sysop;
future device-encrypted mail attachments require a separate ciphertext/key boundary
under #88. The current client implements no mail encryption or roaming authentication.


The [encrypted-mail research proof](../dev-docs/research/0045-encrypted-mail-and-roaming-proof.md)
is separate from product security claims. Its reference devices keep keys in local
memory and pin peer keys, while two homes route/archive ciphertext. Native client
keystore/recovery/interoperability and verified home-principal-to-membership binding
remain #90/#91. The native signer failure under #92 blocks native issuance adoption.
No terminal file frame or remote ANSI prompt may unlock mail secrets or import a
home identity/role. Device-encrypted attachments remain #88; current file areas
remain readable by the sysop.

## New operator/modern-terminal story boundaries

The [operator and communications stories](modern-sysop-and-communications.md)
are requirements, not new deployed authority. Local OS login/operator privileges,
in-BBS sysop permissions and member SSH admission are distinct. Read-only status
uses bounded metadata/health; host/firewall writes require a separate narrow
host-authorized path with exact plans, validation, backups and recovery. Never
run the BBS owner as root or silently alter host policy. Show the actual observed
peer and NAT/shared-source limitations; do not trust supplied original-IP claims.

Capability negotiation grants rendering/input options only. Untrusted probe
replies or graphics data cannot become commands, credentials, terminal file/
network reads or clipboard operations. Existing terminal filtering remains until
a reviewed bounded feature path is implemented. Native XMPP unread/private views
retain local decryption/trust; a remote frame or theme cannot imitate key unlock.

IRC bridges must keep private XMPP/local-only content out of shared channels.
Optional forum federation research needs separate room/author/peer consent and
history/retained-copy policies; a remote peer never imports local account roles
or approval. Public conversations do not force anonymous/public-web deployment.

The [paged room reader](room-reading.md) re-fetches current approval/file visibility
for each screen and does not mark incomplete messages read. Shared read positions
advance only after successful output drain, not a failed close. The article codec
rejects unsupported/control headers and imports no authority; it is independent
of storage/publication and does not convert IRC or encrypted XMPP traffic.

The owner requested a substantial [later-phase hardening backlog](later-phase-hardening.md)
(#106–#111): signing, VM isolation, containment after component compromise, storage
and independent validation/recovery. No VM/chroot/jail/disk-encryption label alone
is an adopted end-to-end guarantee. Keep authority and residual risks explicit.

The [optional Kitty preview](kitty-graphics.md) uses only a fixed original asset.
Probes own their input and confer presentation capability only; native opt-in and
exact-command regeneration never permit arbitrary APC, local paths, clipboard or
file/mail/member authority. Image-specific cleanup is best effort on broken TTYs.

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
under #88. The client has an optional [encrypted local vault](local-mail-vault.md), but
no network mail encryption/delivery or roaming authentication.


The [encrypted-mail research proof](../dev-docs/research/0045-encrypted-mail-and-roaming-proof.md)
is separate from product security claims. Its reference devices keep keys in local
memory and pin peer keys, while two homes route/archive ciphertext. Local vault unlock/backup/recovery now runs on the member device with explicit
device pins; network OMEMO keys/interop/history recovery and verified
home-principal-to-membership binding remain #90/#91. The native signer failure under #92 blocks native issuance adoption.
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

## Explicit local-room draft checkpoints

[The room writer](writing-desk.md) stores only explicitly saved local-room
plaintext, readable by the sysop like local posts. Scope is board/member/epoch/
room/reply; revisions compare before overwrite/delete, and send clears its matching
checkpoint in the post transaction. Four rooms/member, 1024 body bytes/32 lines,
30-day recovery and lazy expiry are bounded application policy, not secure
erasure. Revocation/owner recovery clears affected checkpoints. Owner exports and
backups include them; old copies may remain. Full-screen input/graphics/theme
controls confer no member authority or local editor/path/unlock capability. Every
private-mail draft/key/decryption stays device-side; no server mail fallback.


## Local gatherings

[Gatherings](gatherings.md) are approved-member local plaintext, including RSVP rows
and hidden/cancelled event text in owner backups. Counts-only display is not host
confidentiality. Only the own response and permitted totals are exposed; no attendee
list. Revisions reject stale edits/responses; time/place changes clear attendance.
Current event/approval/private-channel checks apply on deliberate jumps. A title/
UTC-only optional login notice grants no destination access or notification export.
Schema-11 upgrades require stopped complete backup and fresh-path rollback.


## Local community polls

[Poll ballots](polls.md) are ordinary plaintext owner data in complete backups,
with no secret-ballot/election-grade claim. Only own selections and permitted totals
are exposed; no voter-name list or author/sysop display-policy bypass. First ballot
permanently locks wording; current revisions protect reviewed votes and close races.
Hidden/inactive polls cannot accept votes; revoked voters are excluded from totals.
No message publication, unread mutation, private-mail decrypt or federation path.
Schema-12 upgrades require stopped complete backup and fresh-path rollback.


## Explicit local catch-up

[Catch-up](catchup.md) reads current room/reply/file/event metadata under current
approval and source permissions. Previews never acknowledge read/download; only
explicit preference or visit saves persist own state. Saved UTC visits/favorites
are sysop-readable local data in owner backups, not mandatory login telemetry.
File publication counters are not exposed, and stale private-file grants are
rechecked on jumps. No server mail decryption or badge is introduced. Schema 13
requires stopped complete backup and fresh-path rollback.


## Local discussion discovery

[Find & save](discussion-search.md) applies current approval/source checks and
file-link projection before body matching and snippets. Exact source views and
bookmark/export actions never advance unread. Queries remain connection-local;
bookmarks store own references with current captions, not copied bodies or hidden
history. They remain plaintext operator-readable data in owner backups. Deleted
posts cascade references; logical deletion is not secure erasure. Paged member
reference exports contain public local identities and hex titles, never paths,
credentials or private-mail keys/content. No server-side private-mail index or
count is added. Schema-14 upgrades require stopped complete backup/fresh rollback.


## Curated gallery packs

See [the gallery guide](scene-gallery.md) for the optional owner-managed startup
pack, current-approval file links, canonical ANSI/text editions and rights review.
Gallery packs are external host configuration, preserved separately from database
exports/bundles. No new schema, public service or raw uploaded graphics authority.


## Device-local travel choices

[Travel settings](travel-modes.md) stay in the native process, never central history
or remote authority. Compact uses the same pinned SSH and local approval path;
per-file size deferral precedes upload reservation/download grant/SFTP. No route
fallback, server mail decryption, new schema or changed resource defaults.


## Curated library shelves

[Library shelves](library-shelves.md) layer owner-reviewed notes/collections over
current local file publication/grants. Hidden sources expose no captions, shelf
names or tokens. Transfers/discussion use existing authorized workflows; browsing
alone never advances unread. Packs are external config to preserve separately
from complete DB/blob backups. Schema remains 14; no new transfer authority.


The optional [magazine workshop](magazine-workshop.md) stores local plaintext
editorial work. Private drafts are author/sysop-readable; explicit submission
shares the proposed complete issue with selected contributors for review. This is
separate from device-encrypted mail. Current local identity/status and exact-issue
author approvals gate publication, not PGP signatures or resistance to a malicious
database owner. Edits invalidate consent; withdrawal hides ordinary reading but
cannot recall exports/downloads/backups. Related file publication/grants are
separate and need independent withdrawal. Schema 16 upgrade requires a stopped
complete backup; no new listener, privilege or relaxed runtime budget is added.

The optional [social lounge](social-lounge.md) advertises only explicit live-chat
availability. Both participants' current approval/epoch/lease and privacy controls
gate bodyless invitations and replies. No public hidden counts, private channel
labels or last-seen log is added. Existing channel /names remains independent of
lounge hiding. Mute, blocked names and UTC quiet-hour intent are private durable
settings; exports strip presence/signals/rate timestamps. No custom page body,
plaintext private mail, forced channel switch, privilege or delivery guarantee.
Take a stopped complete backup before schema 17. Broader hardening stays deferred.

The shared service's [outside-chat lounge](social-lounge.md) reuses one optional
connection lease with a silent heartbeat across UI states. Leaving chat removes
channel joins/send context while retaining explicit visit availability; disconnect
removes the lease. Menu notices are generic and only render at safe redraws.
The panel re-fetches all eligibility instead of replaying cached rosters/signals.
No new helper, database connection, private-message body, forced switch, hidden
channel metadata or live-lease export is added; schema 17 stays unchanged.
Local login/inherited workers now borrow the same Board for a visit owner and
silently heartbeat inside existing input loops. Logout/ordinary EOF/error closes
the lease; abrupt termination still uses stale expiry. No new helper/thread or
SQLite connection is added. Plain terminals retain the same controls.


Optional [idle page notifications](social-lounge.md) require /notify on for the
current visit and start off at every login. Current core approval/epoch/lease,
feature state and both-party eligibility are rechecked before a signal appears.
Only an empty idle main menu is eligible; partial input/control strings, drafts,
readers, games, transfers and probes are excluded. The bounded presentation state
retains at most eight signal IDs/states, with no names/text or deferred event queue.
No bell/desktop escape or private-message preview is emitted. Polling is limited
to once per two seconds while safe and opted in; no new helper, connection or
schema. Plain standalone input tracking applies only when opted in and retains
its existing limits. Hardening remains later.


The [Who caller list](who-and-presence.md) is separate from the Lounge: main-menu
Who and group `/who` show broad current activity plus availability/status, while
the Lounge retains its social/paging attributes. All three share the same current
visibility/availability/blocks. Schema 18 adds transient lease-bound activity;
no private resource names or retained activity history. Take a stopped complete
backup before upgrade; export/restore strips current activity and live authority.

The [device-local connection desk](connection-desk.md) separates file readiness,
stored pins and untested remote authority. Explicit local ssh-keygen lookups are
bounded/reaped and never connect; share reports omit contact/key identifiers.
No server/schema/dependency/default change or broader hardening claim.

The [local passport](local-passport.md) keeps explicitly saved plaintext notes
on the member device under private permissions, separate from encrypted mail.
No automatic visits/timestamps/beacons or server passport. Generic seals and local
profile hashes grant no verified identity or admission; private fields share only
when selected and public exports omit contact/key/route/binding identifiers.

# 0032: OpenSSH transport with a scoped single-object SFTP adapter

Status: accepted for the optional hobby preview; broader deployment not validated.
Date: 2026-10-21. Authority: owner authorized #83 implementation.
Issue: [#83](https://github.com/gonz0w/faction-bbs/issues/83).
Follows [ADR 0030](0030-file-areas-and-transfer-adapters.md).

## Question and alternatives

How can a member transfer one approved local file using an existing SFTP client,
while current BBS approval, area policy, exact size and quotas remain authoritative?

| Candidate | Useful reuse | Authorization/resource tradeoff | Disposition |
| --- | --- | --- | --- |
| Base internal-sftp with chroot/request filters | Mature full protocol/file operations | Host path access, one-time admission and chroot do not implement per-chunk BBS revocation, one-object grants or atomic quota reservations. Dynamic mounts/keys and broker mediation add privileged lifecycle work | Retain as future alternative; insufficient alone for current scope |
| Base OpenSSH plus bounded Rust virtual SFTP v3 | Reuses SSH crypto, signatures, host identity and standard clients; no HTTP/JS dependency | New parser/file lifecycle code requires real interoperability and negative tests; intentionally exposes only one object and limited operations | Select for this preview |
| Native Faction client / classic terminal transfers | Future smoother BBS UX | Would delay ordinary file sharing or require raw binary bypass of existing terminal codecs | Later adapters over the same core policy |

Qualitative decision, no numeric winner or unmeasured security score. Broad SFTP
features, hostile-native-code isolation, full platform capacities and all-client
compatibility are outside this preview's validation scope.

## Boundary and behavior

Schema 4 owns areas, explicit grants, catalog, transactional byte reservations and
short-lived one-use key-bound transfers. Canonical Ed25519 public-key blobs bind
exactly the key verified by base OpenSSH; private keys never enter the BBS. A
separate lookup UID can only request a ready-key match. The file UID can claim,
check, advance or complete one bounded operation through UID-checked policy IPC;
it gets no member session, credentials, TTY, room data, SQL or game authority.

The existing shared loop serves at most four bounded policy peers without adding
a process/thread. The separate SSH listener remains disabled/loopback by default.
Current member epoch, approval, area grant and object state are checked at lookup,
claim and each request/chunk. Only sysops may download completed pending bytes for
review; ordinary members receive published immutable bytes only.

Use 32-KiB chunks, two persistent kernel transfer slots, 16-MiB files, 256-MiB board
storage, two incomplete uploads/member and five-minute grants/helper wall bounds.
These are preview implementation caps, not measured peak memory or SSH preauth
budgets. Incomplete uploads remain hidden/charged; no resume. Explicit removal/GC
reclaims data only with both slots held, including the exact paired-hardlink crash
state. Unknown aliases fail closed. Review/publication is one SQLite transaction;
filesystem completion precedes pending catalog state and never publishes partials.

## Evidence, limits and reversal

See [plan 0033](../plans/0033-file-areas-sftp.md) and the native report linked there.
Host tests cover policy/epoch/key/replay/expiry/quota races, framing, path aliases,
menu capabilities and exports. The software NetBSD VM uses real OpenSSH SFTP for
upload, quarantine review, publication, checksummed download, path/alias/revocation
failure, GC, restart and complete catalog/blob restore. This is integration
confidence, not an independent security audit, physical SBC proof or 250-user claim.

Primary sources retrieved 2026-10-21: NetBSD [sshd_config(5)](https://man.netbsd.org/sshd_config.5),
[sftp-server(8)](https://man.netbsd.org/sftp-server.8), and the
[SFTP v3 working draft](https://www.ietf.org/archive/id/draft-ietf-secsh-filexfer-02.txt).
Online availability is distinct from actual NetBSD 11 option/client validation.

Moderate reversal cost: disable/drain the optional listener and remove both UID
keys; ordinary BBS service remains. Another adapter can use existing semantic
policy operations. Schema 4 is not backward-readable by old binaries: keep a
pre-upgrade backup. Database exports retain catalog but exclude blobs and remove
transfer authority. The separate complete private bundle restores fresh paths,
identity and exact bytes. Never overwrite live state or reset lock inodes.

Reopen for broader client/resume requirements, larger quota/stream limits, native
client integration, or independent parser/authority review. Keep mail encryption,
federation, classic doors and all-host network sandboxing separately scoped.

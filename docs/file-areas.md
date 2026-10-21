# File areas and SFTP

Status: selected next product increment, **not implemented yet**. Owner selected
SFTP through a separate restricted transfer service on 2026-10-21. Design #22;
implementation [#83](https://github.com/gonz0w/faction-bbs/issues/83).

The BBS will provide **[F]iles**: browse local areas, read descriptions, find files,
request uploads/downloads and review transfer status. Sysops manage areas, review
quarantined uploads and publish/remove files. Files stay on the local board;
addressed federated mail does not federate file areas.

SFTP supplies the first transfer adapter. Use a separate file-service identity,
private storage and restricted SSH service. The existing terminal-only BBS SSH
listener keeps denying shell, exec, SFTP and forwarding. An SSH key or file-service
connection alone never grants access: current BBS approval, epoch and area scope
must authorize the requested operation. No general host filesystem is exposed.

## Intended member flow

1. Log into the BBS and choose a permitted area/file or request an upload.
2. Receive a short-lived transfer grant bound to the exact operation and the
   transfer client's public SSH key. Private keys stay on the member's device.
3. Use an SFTP client against the separately configured transfer service. A future
   native Faction client can make this feel like one terminal workflow.
4. Downloads stream authorized immutable files. Uploads remain private/incomplete
   until length/hash validation and sysop review publish a complete object.

This is a design, not working ticket syntax or an installed SFTP endpoint. Compare
base OpenSSH SFTP reuse and a bounded native SFTP handler before selecting the
implementation. A chroot alone cannot enforce live BBS approval or area grants.

## Local policy and storage

Areas have a title/description, visibility/access policy, upload permission and
quotas. Approved-member and explicitly granted private areas remain distinct.
Member filenames/descriptions are untrusted metadata; disk objects use generated
IDs. No member-controlled path, symlink, overwrite, execution or archive extraction.
Listings are paginated and terminal-safe. Published content is immutable; replacing
it creates a new object and requires review.

Reserve byte/count quotas before writing so simultaneous uploads cannot overspend
one allowance. Bound active transfers, chunk buffers, request/frame sizes, idle/
wall deadlines and partial retention. Interruption never creates a visible partial
file. Resume must prove the same object/grant/owner and valid offset; it cannot
restore expired approval or bypass a quota. Proposed initial experiment bounds
are documented in [research 0043](../dev-docs/research/0043-file-areas-and-sftp.md),
not supported limits.

The first backup tool must include both the catalog and file objects with a
consistent manifest. **Current `faction export` does not back up file-area data.**
No automatic public download URL, HTTP service, mail attachment upload, anonymous
access or cross-board replication is implied. Classic in-session transfers can
be added through another adapter later without replacing area policy/storage.

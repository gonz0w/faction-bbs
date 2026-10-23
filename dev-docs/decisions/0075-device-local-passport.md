# 0075: A device-local passport with unverified souvenirs

Status: accepted bounded implementation
Date: 2026-10-23
Issue: [#161](https://github.com/gonz0w/faction-bbs/issues/161)

## Decision

Use an optional native notebook between board sessions for explicit personal
notes and original/licensed ASCII calling cards. Never infer or record a visit
from opening the launcher, connecting or collecting a file. Generic art is a
souvenir, not identity/admission/membership or a verified timestamp. Verified
contacts are not implemented; bind to a clearly labeled local configuration hash
and require explicit review/rebind for changed profiles/pins. Orphan entries stay
viewable/exportable/deletable without reviving a route or credentials.

The notebook is plaintext on the member's device, with private directory/file
ownership, no-follow descriptor-relative access, bounded parsing, atomic replace
and one lifetime lock. It is not private mail. Reusing the encrypted mail vault
would couple separate formats/unlock/recovery semantics; a separate encrypted
notebook can be designed later. Plaintext is disclosed before note entry and in
init output/docs. Keep private backups local and explain logical deletion limits.

Import only from explicitly chosen local reviewed files; reuse existing approved
file areas/SFTP for a sysop's optional generic pack distribution. No server-side
passport, automatic remote frame, path, unique visitor seal or network request.
Share previews omit notes/favorites unless selected, and always omit local contact
IDs, bindings, routes, keys and timestamps. Plain CLI export selects one souvenir.

## Consequences and alternatives

No server storage/capacity or runtime dependency change. A bounded ASCII art model
works on ordinary cells without trusting controls. Server passports/automatic
collection would introduce travel tracking and are rejected. Verified board-key
signatures, encrypted notebooks and integrated generic pack offers remain follow-on
work; seals do not fill those roles. No supported release/hardware/hardening claim.

# Native picker mail and outage compatibility

Issue [#214](https://github.com/gonz0w/faction-bbs/issues/214) adds explicit local
**E** home mail to the board picker. The [member guide](../../../docs/picker-home-mail.md)
describes configuration, locking, supported builds and classic/Unicode behavior.
The [pinned result](results.json) records software observations, not a deployment
or complete mailbox/roaming/private-chat claim.

## Actual checks

Host and native NetBSD runs each passed 17 picker controls and eight combined
journey controls. In the combined journey, one running picker opens encrypted
mail, encounters an actual refused home TCP connection, reads existing local
history, locks, then logs into an independently approved BBS and completes an
upload, sysop review/publication and checksummed restricted-SFTP download. It
returns to the same picker. Canonical encrypted bytes, authenticated history,
pending operation IDs and the full nonempty archive cursor/epoch remain exact.

The deliberately public BBS payload is separate from private mail. Both actual
visited-board transcript segments and board database/WAL, file store and service
logs were checked for private synthetic mail markers. No matches were found.
These checks do not prove absence of every possible leak or resistance to a
compromised member device. The software fixture places member and server roles
under separate identities in an owned VM; it is not a physical-device security
certification.

Client/OMEMO all-target release tests passed 142 per OS, with two intentional
subprocess helpers invoked by parents. Host PTY qualification uses a debug binary;
the actual combined host journey separately uses a release binary. Native tests
and journeys use the recorded native release artifacts. Default and native-only
builds each reject the configured online entry before opening a store. Existing
picker/vault/home/archive/checkpoint controls passed 8/13/9/6/23 per OS. All 307
current host Python tests passed; nine focused oracle tests also passed natively.
Required PR CI is separate from these saved observations.

## Source and resource scope

[The baseline manifest](../archive-mail-picker-baseline-manifest.json) binds all
644 #211 execution identities to their already-preserved exact objects and pins
the unchanged complete prior result. No earlier hashes or measurements were
refreshed. New native ledgers retain 677 server-build inputs, 679 client-build
inputs and 680 runtime inputs, including four explicit Python-only overrides.
Rust/C/product artifacts remain unchanged across those fixture corrections.
Unchanged objects are reused instead of copying another complete source tree.

The owned builder had two cores/2 GiB. The same overlay was rebooted at 1 GiB
(1,054,990,336 physical bytes) for runtime checks. A fresh complete 1 GiB build,
physical SBC, capacity, new RSS/throughput or production-support claim is not made.
Normal eight sessions/5000-ms admission, credential/kernel/helper limits, Argon2
and file-stream/chunk/grant/output bounds remain unchanged.

## Preserved corrections and cleanup

A rejected helper version could omit its inspector and still report all 17
controls. The actual incorrect pass is retained as rejected evidence. The API and
CLI now require the inspector, and state/refusal assertions cannot be skipped.

NetBSD times out on a bound, non-listening TCP socket while a closed port refuses
immediately. The final fixture selects then closes the reservation and requires
actual refusal before and after the journey. It still requires exact
`Transport(Connect)`; product deadlines and network policy were not changed.
Earlier heading/PATH/audit-script failures and executed source versions remain
identified separately. They are not relabeled as successful runs.

Only public source code, structured control/proof objects, counts and digests are
included. Raw terminal output, generated credentials, mail bodies, profiles and
opaque cursor values are excluded. Source publication checks distinguish already
published synthetic fixture constants from actual runtime private values.

[Verified teardown](teardown.json) stopped only current owned board/file services
and the marked VM/controller, closed the three loopback forwards and removed the
four owned fixture roots. The saved owner playground stayed stopped and unchanged.

Run `python3 tools/check_picker_mail_evidence.py` to verify saved record identity,
source bindings, exact journey/state/feature/privacy controls and teardown. This
checks saved evidence; it does not rerun a VM. The combined journey supplies the
remaining original compatibility proof for [#90](https://github.com/gonz0w/faction-bbs/issues/90).
The broader unread/in-session/private-chat story remains
[#99](https://github.com/gonz0w/faction-bbs/issues/99); roaming and attachments stay
[#91](https://github.com/gonz0w/faction-bbs/issues/91) and
[#88](https://github.com/gonz0w/faction-bbs/issues/88).

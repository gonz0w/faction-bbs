# Main-menu member-device mail handoff

The optional `home_mail` theme action parks an approved shared BBS main menu while
the native client runs its existing local encrypted home-mail console. This is a
narrow connection-bound UI handoff, not authentication, file authority, XMPP SSO
or a generic transport framework. [Member/sysop guide](../docs/main-menu-mail.md).

## Wire and state

A separate namespace preserves the existing `FACTION1`/`native-v1` Files contract.
Old native filters ignore the unknown bounded OSC and retain a readable plain
return prompt. No transfer key, file namespace/grant, home identity, mailbox
credentials or mail metadata appears in the mail control.

| Direction | Exact control |
| --- | --- |
| Board offer | `ESC ] 7770;FACTION-MAIL1;offer BEL` |
| Device park request | `@MAIL1 park NONCE` plus normal line termination |
| Board parked acknowledgement | `ESC ] 7770;FACTION-MAIL1;parked;NONCE BEL` |
| Device return request | `@MAIL1 back NONCE` plus normal line termination |
| Board return acknowledgement | `ESC ] 7770;FACTION-MAIL1;returned;NONCE BEL` |

NONCE is exactly 32 lowercase hexadecimal characters generated from 16 random
bytes on the device. The pure standard-library terminal codec validates bounded
payloads, exact fields/keywords and nonce shape. This framing adds no cryptographic
primitive; SSH and current local BBS authority remain separate prerequisites.

The initial offer uses the same advertisement gate as Files: ANSI presentation or
the existing explicit native compact hint. Ordinary dumb/plain sessions receive
the explanation and Enter/back return without OSC bytes. A plain menu action
retains its Menu context before dispatch, just like the ANSI hotkey path.

The shared state sequence is Menu → MailOffer → MailParked → Menu. Each transition
rechecks current approval, epoch and any roaming provenance through the real core
session. MailOffer accepts one park or a versioned back; plain Enter/back permits
classic recovery. A versioned back from MailOffer echoes its validated fresh nonce
without starting a private view. MailParked accepts only the stored nonce's back.
The return acknowledgement precedes the redraw of the same approved main menu.

A bounded per-connection history retains at most 64 consumed nonces. Reuse across
offers is rejected. Exhaustion refuses a new offer with an explanatory main-menu
return; it does not consume unbounded state or disable ordinary BBS access.
No nonce history is persisted or shared between board connections.

## Device boundary

Remote offers are data. They may only present the fixed local O Open / B Back /
N Local notices choice. Back is the initial Enter default. N changes only the
existing visit-local notice consent: discard typeahead, reset the decoder and
Back selection, then redraw locally. It does not access Remote, unlock a vault,
contact home or consume another park/back exchange.
Open requires deliberate local choice and fresh confirmation, then the matching
parked acknowledgement before the existing hidden device unlock. Missing optional
build/configuration explains and returns. Remote frames cannot supply local paths,
choose a store, unlock, acknowledge or start a home network cycle.

Keep the mail receive boundary ordered: discard every remote text byte preceding
an acknowledgement; retain only post-return board output after validation. Parse
all controls in the read, including malformed or unexpected frames after a valid
reply. A matching reply alone does not excuse an invalid trailing frame. Combined
existing frame/output limits and five-second exchange/drain deadlines still apply.

During the local console, do not pump/render remote data or forward mail keyboard
bytes. On every exit drop the vault, secrets and console terminal guard before
processing board output. Flush typeahead/decoder state and clear the viewport at
transitions. Scrollback is not erased. Keep the existing 120-second non-menu prompt
timeout, session lifetime and bounded PTY backpressure; loss/revocation is reported
without reconnect, retry or an indefinite-pause claim.

## Compatibility and limits

Standalone and inherited adapters explain the native/shared requirement and
return to their menu. They do not impersonate the native console. Default packs,
ordinary client builds and Files/native-v1 behavior remain unchanged. Broad
presence stays Menu. Existing opt-in notice snapshots remain local and stale or
unknown; no new background work or credential lease is introduced.

The main-menu entry and Files share one `VisitMail` for the board connection.
Consent starts off. Opt-in without a snapshot is locked/unknown; successful
explicit console lock may retain only unread, retained and pending counts.
Disabled tracking leaves unread unknown. A cancelled unlock or unsuccessful
console run invalidates the snapshot; declining the local Open confirmation
leaves it unchanged. A recoverable home-cycle error inside a healthy console can
still yield stale local counts on graceful lock. Opt-out clears it, re-enabling
cannot resurrect it, and reconnect resets consent. The
context selects fixed O (main-menu) or E (Files) hints, with inert lines bounded
to 39 cells at the existing 64-history/8-pending limits. L still drops the vault
and returns directly to the BBS main menu; selecting the entry again displays
its local notice. No server badge, protocol, schema, dependency or polling is
added, and Files need not be configured.

No schema, dependency, public listener, provider, file capability or BBS membership
is added. The member still chooses manual mail operations and explicit read
acknowledgements. Attachments, live private chat, broader active-screen access and
online roaming enrollment remain separate issues.

The [source-bound software observations](evidence/main-menu-mail/README.md) retain
actual RED/GREEN variants, host/NetBSD classic and Unicode controls, compatibility,
private I/O checks and the existing Files/SFTP regressions.

The [local notice observations](evidence/menu-mail-notices/README.md) cover the
fixed O/B/N choice and shared visit-local state, with preserved host/NetBSD
sources, controlled captures and explicit evidence limits.

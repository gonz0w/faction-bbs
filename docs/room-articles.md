# Authored subjects and local room articles

Status: bounded local implementation, 2026-10-21. Discussion rooms remain
chronological, local and useful with every network adapter disabled. No news
listener, feed, peer import or publication is enabled.

## Posting and replying

After choosing the room and optional parent post, enter a subject of up to 160
printable ASCII bytes. A blank new-post subject uses `Message #ID`. A blank reply
inherits the parent's subject with one `Re: ` prefix; the default is shown before
entry. Long inherited subjects are clipped to 160 bytes, removing trailing spaces.
You can enter a different reply subject. Existing post subjects are immutable.

The shared service and local/worker console use the same core operation. The
subject travels with the body through busy/retry state. A request ID with changed
subject, body, parent or room conflicts rather than overwriting content. Approval
and the current session epoch are checked in the posting transaction. Subject
controls, surrounding spaces, encoded-word syntax and non-ASCII subjects are
rejected explicitly; full internationalized subject entry is tracked in #114. UTF-8
bodies retain the current text limits and inert terminal presentation.

Schema remains 6; no second migration relabels old generated subjects as authored.
Use the [stopped upgrade/backup procedure](post-metadata.md) and update binaries
together. Schema-6 export/restore retains authored subjects, dates, IDs and reply
links. Unknown legacy dates stay unknown and legacy bodies remain readable.

## Core projection, without publication

`Board::room_article(session, post_id)` connects stored posts to `faction-articles`.
It rechecks approved local membership in one read transaction, along with current
permissions on every recognized file link. The returned article is a local
proto-article, not a grant to export or publish its room.

| Field | Current local projection |
| --- | --- |
| Message-ID | Persistent `<opaque-id@faction.invalid>` from schema 6. |
| From | Per-article `author-opaque-id@authors.faction.invalid`, with no handle, real mailbox, host or home identity. This alias is deliberately not a stable cross-post author identity. |
| Date | Recorded new-post UTC time, including seconds; no invented legacy date. |
| Subject | Authored text or the stored generated/reply default. |
| Newsgroups | `faction.local.rROOM_ID`, a local alias; no automatic subscription or real room-title disclosure. |
| Path | `not-for-mail`, no host route or injection assertion. |
| References | At most sixteen ordered IDs: root plus the nearest fifteen, traversing no more than 128 same-room ancestors. |
| Body | Exact stored supported text; no viewer-dependent rewrite under one ID. |

Projection refuses an unknown legacy date, inaccessible file link, invalid/cyclic
parent chain or more than 128 ancestors. Ordinary reading remains available;
projection never alters unread cursors, legacy records or bodies. The
[v2 text profile](article-profile.md) now encodes long logical lines with soft MIME
breaks, retaining exact content while bounding physical lines. Reply pruning keeps
the root and recent context without unbounded work.

Reserved `.invalid` aliases grant no identity, membership, mail delivery or
signature authority. Public IDs remain separate from private rotating session
identity and file namespace. Header data must never become approval or sysop roles.
#101 owns explicit room/author/peer/history policy and a real injection profile;
#104 owns provider connections. IRC and device-encrypted XMPP formats are unchanged.

## Codec and duplicate handling

The [bounded codec profile](discussion-articles-and-news.md) keeps ASCII headers,
LF-normalized UTF-8 text bodies, encoded UTF-8 subjects, plain-text MIME decoding,
CRLF/quoted-printable canonical output and bounded folded References.
Unsupported MIME, HTML, binary attachments, raw controls and unknown authority/
control headers fail closed. `compare_identity` compares validated normalized
articles: different IDs are distinct, identical fields/body are duplicates, and
any differing content under one ID is a conflict. Path changes also conflict in
this conservative local profile. Future adapters must reject/quarantine conflicts;
this API introduces no remote insertion, replacement, dedup database or cancel API.

The synthetic `emit_fixture` example and `tools/check_article_interop.py` exercise
a separate Python standard-library mail/MIME parser. This checks headers/date/
references/UTF-8 interpretation, not a news server or independent peer exchange.
The `profile_articles` example measures requested live allocation and elapsed time
for a 16-KiB synthetic body separately from encoded size. It excludes allocator
metadata, stacks, RSS, SQLite, SSH, board state and concurrent-member capacity.
See [recorded software evidence](../dev-docs/evidence/authored-articles-netbsd/README.md).

[Profile v2 and original INN checks](article-profile.md) complete the bounded local
article-format deliverable under #103. #101/#104 own live independent peers/providers
and consent/injection policy; #114 owns Unicode composer/storage compatibility.
The later [hardening phase](later-phase-hardening.md) remains queued separately.

# Reading discussion rooms

The local console, member worker and shared service use the same bounded message
presenter. Select **New** for unread posts or **History** to start at the beginning,
then enter the room ID. Messages stay chronological.

Each screen shows the room number, message number, author and reply-to context,
with separators and text wrapping. `y` at **More? [y/N]** continues the current
long message or opens the next message; `n` or Enter leaves the reader. ANSI
sessions clear the old screen and use the sysop theme's normal palette. Plain
sessions show the same text without ANSI controls. [Persistent metadata](post-metadata.md) shows generated subject labels and new-post
UTC times; older posts have explicit unknown dates.

Leaving midway through a long message keeps it unread. Only finishing its final
screen advances the read position; the shared service additionally waits for that
screen's output to drain. This is an output acknowledgment, not proof a person
has read it. Approval and per-viewer file visibility are checked again for each
screen. Account revocation ends access before another page is rendered.

Rendering escapes member-supplied controls and non-ASCII characters into inert
text, preserving the current conservative character policy. Quote markers and
paragraph breaks remain visible. Native Unicode glyph-width/graphics support is
still #96, and richer reply composition remains #21/#25.

Valid local file references stay contiguous in the byte stream so native-client
recent-file shortcuts and SSH copy/paste still work. A narrow terminal soft-wraps
these 78-byte references; their physical rows count toward the page budget. A
reference does not grant file access; current permissions are checked on download.

The current schema-6 metadata increment preserves local room IDs, post/reply data,
unread and file namespaces. New exports include persistent metadata; old exports
remain readable. Follow the [stopped-service migration](post-metadata.md) procedure. The separate
[article codec](discussion-articles-and-news.md) is a foundation for future
Netnews integration, not an automatic conversion or news feed.

Posting now offers an [authored subject](room-articles.md) after the optional
parent ID. Blank replies inherit a previewed subject with one Re: prefix; the
reader shows the stored subject on every page. Chronological order and unread
acknowledgment are unchanged.

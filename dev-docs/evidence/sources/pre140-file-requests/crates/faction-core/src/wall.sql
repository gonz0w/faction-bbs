CREATE TABLE wall_settings (
 singleton INTEGER PRIMARY KEY CHECK(singleton=1),
 enabled INTEGER NOT NULL CHECK(enabled IN (0,1)),
 name TEXT NOT NULL CHECK(length(name) BETWEEN 1 AND 24),
 max_notes INTEGER NOT NULL CHECK(max_notes BETWEEN 1 AND 128),
 note_bytes INTEGER NOT NULL CHECK(note_bytes BETWEEN 32 AND 240),
 retention_days INTEGER NOT NULL CHECK(retention_days BETWEEN 1 AND 365),
 cooldown_seconds INTEGER NOT NULL CHECK(cooldown_seconds BETWEEN 1 AND 3600),
 weekly INTEGER NOT NULL CHECK(weekly IN (0,1)),
 archive_editions INTEGER NOT NULL CHECK(archive_editions BETWEEN 0 AND 12),
 edition INTEGER NOT NULL CHECK(edition>=0),
 CHECK(weekly=1 OR archive_editions=0)
) STRICT;
INSERT INTO wall_settings VALUES(1,0,'Community wall',64,240,30,30,0,0,0);
CREATE TABLE wall_notes (
 id INTEGER PRIMARY KEY AUTOINCREMENT,
 edition INTEGER NOT NULL CHECK(edition>=0),
 member_id INTEGER NOT NULL REFERENCES members(id),
 request_id TEXT NOT NULL CHECK(length(request_id)=32),
 body TEXT NOT NULL CHECK(length(CAST(body AS BLOB)) BETWEEN 1 AND 240),
 posted_at INTEGER NOT NULL CHECK(posted_at>=0),
 edited_at INTEGER NOT NULL CHECK(edited_at>=0),
 UNIQUE(member_id,request_id)
) STRICT;
CREATE INDEX wall_edition ON wall_notes(edition,id);
CREATE TABLE wall_receipts (
 member_id INTEGER PRIMARY KEY REFERENCES members(id),
 request_id TEXT NOT NULL CHECK(length(request_id)=32),
 note_id INTEGER NOT NULL CHECK(note_id>0),
 last_at INTEGER NOT NULL CHECK(last_at>=0)
) STRICT;

CREATE TABLE page_settings (
 singleton INTEGER PRIMARY KEY CHECK(singleton=1),
 enabled INTEGER NOT NULL CHECK(enabled IN(0,1)),
 name TEXT NOT NULL CHECK(length(name) BETWEEN 1 AND 24),
 compact INTEGER NOT NULL CHECK(compact IN(0,1)),
 fields INTEGER NOT NULL CHECK(fields BETWEEN 0 AND 15),
 history INTEGER NOT NULL CHECK(history BETWEEN 1 AND 16),
 days INTEGER NOT NULL CHECK(days BETWEEN 7 AND 365),
 moderator_remove INTEGER NOT NULL CHECK(moderator_remove IN(0,1))
) STRICT;
INSERT INTO page_settings VALUES(1,0,'Member pages',0,15,8,90,0);
CREATE TABLE member_pages (
 member_id INTEGER PRIMARY KEY REFERENCES members(id),
 bio TEXT NOT NULL CHECK(length(CAST(bio AS BLOB))<=240),
 interests TEXT NOT NULL CHECK(length(CAST(interests AS BLOB))<=120),
 signature TEXT NOT NULL CHECK(length(CAST(signature AS BLOB))<=240),
 project TEXT NOT NULL CHECK(length(CAST(project AS BLOB))<=120),
 visible INTEGER NOT NULL CHECK(visible BETWEEN 0 AND 15),
 published INTEGER NOT NULL CHECK(published IN(0,1)),
 moderated INTEGER NOT NULL CHECK(moderated IN(0,1)),
 revision TEXT NOT NULL CHECK(length(revision)=32),
 updated_at INTEGER NOT NULL CHECK(updated_at>=0)
) STRICT;
CREATE TABLE member_plans (
 id INTEGER PRIMARY KEY AUTOINCREMENT,
 member_id INTEGER NOT NULL REFERENCES members(id),
 request_id TEXT NOT NULL CHECK(length(request_id)=32),
 body TEXT NOT NULL CHECK(length(CAST(body AS BLOB)) BETWEEN 1 AND 480),
 visible INTEGER NOT NULL CHECK(visible IN(0,1)),
 posted_at INTEGER NOT NULL CHECK(posted_at>=0),
 UNIQUE(member_id,request_id)
) STRICT;

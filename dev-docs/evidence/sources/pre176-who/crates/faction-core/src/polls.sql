CREATE TABLE poll_settings (
 singleton INTEGER PRIMARY KEY CHECK(singleton=1),
 enabled INTEGER NOT NULL CHECK(enabled IN(0,1)),
 name TEXT NOT NULL CHECK(length(name) BETWEEN 1 AND 24),
 compact INTEGER NOT NULL CHECK(compact IN(0,1)),
 member_creators INTEGER NOT NULL CHECK(member_creators IN(0,1))
) STRICT;
INSERT INTO poll_settings VALUES(1,0,'Polls',0,0);
CREATE TABLE polls (
 id INTEGER PRIMARY KEY AUTOINCREMENT,
 creator INTEGER NOT NULL REFERENCES members(id),
 request_id TEXT NOT NULL CHECK(length(request_id)=32),
 revision TEXT NOT NULL CHECK(length(revision)=32),
 question TEXT NOT NULL CHECK(length(CAST(question AS BLOB)) BETWEEN 1 AND 96),
 description TEXT NOT NULL CHECK(length(CAST(description AS BLOB))<=240),
 results INTEGER NOT NULL CHECK(results BETWEEN 0 AND 2),
 state INTEGER NOT NULL CHECK(state BETWEEN 0 AND 2),
 hidden INTEGER NOT NULL CHECK(hidden IN(0,1)),
 ever_voted INTEGER NOT NULL CHECK(ever_voted IN(0,1)),
 changed_at INTEGER NOT NULL CHECK(changed_at>=0),
 UNIQUE(creator,request_id)
) STRICT;
CREATE TABLE poll_choices (
 poll INTEGER NOT NULL REFERENCES polls(id) ON DELETE CASCADE,
 number INTEGER NOT NULL CHECK(number BETWEEN 1 AND 8),
 label TEXT NOT NULL CHECK(length(CAST(label AS BLOB)) BETWEEN 1 AND 48),
 PRIMARY KEY(poll,number)
) STRICT;
CREATE TABLE poll_ballots (
 poll INTEGER NOT NULL REFERENCES polls(id) ON DELETE CASCADE,
 member INTEGER NOT NULL REFERENCES members(id),
 choice INTEGER NOT NULL CHECK(choice BETWEEN 1 AND 8),
 PRIMARY KEY(poll,member),
 FOREIGN KEY(poll,choice) REFERENCES poll_choices(poll,number) ON DELETE CASCADE
) STRICT;

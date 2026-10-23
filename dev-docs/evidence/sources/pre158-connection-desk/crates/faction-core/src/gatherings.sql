CREATE TABLE gathering_settings (
 singleton INTEGER PRIMARY KEY CHECK(singleton=1),
 enabled INTEGER NOT NULL CHECK(enabled IN(0,1)),
 name TEXT NOT NULL CHECK(length(name) BETWEEN 1 AND 24),
 compact INTEGER NOT NULL CHECK(compact IN(0,1)),
 member_organizers INTEGER NOT NULL CHECK(member_organizers IN(0,1)),
 reminders INTEGER NOT NULL CHECK(reminders IN(0,1)),
 public_counts INTEGER NOT NULL CHECK(public_counts IN(0,1))
) STRICT;
INSERT INTO gathering_settings VALUES(1,0,'Gatherings',0,0,0,1);
CREATE TABLE gatherings (
 id INTEGER PRIMARY KEY AUTOINCREMENT,
 organizer INTEGER NOT NULL REFERENCES members(id),
 request_id TEXT NOT NULL CHECK(length(request_id)=32),
 revision TEXT NOT NULL CHECK(length(revision)=32),
 title TEXT NOT NULL CHECK(length(CAST(title AS BLOB)) BETWEEN 1 AND 64),
 body TEXT NOT NULL CHECK(length(CAST(body AS BLOB))<=480),
 starts_at INTEGER NOT NULL CHECK(starts_at BETWEEN 946684800 AND 4102444800),
 ends_at INTEGER NOT NULL CHECK(ends_at>starts_at AND ends_at<=starts_at+86400),
 place TEXT NOT NULL CHECK(length(place)<=40),
 cancelled INTEGER NOT NULL CHECK(cancelled IN(0,1)),
 hidden INTEGER NOT NULL CHECK(hidden IN(0,1)),
 changed_at INTEGER NOT NULL CHECK(changed_at>=0),
 UNIQUE(organizer,request_id)
) STRICT;
CREATE TABLE gathering_rsvps (
 gathering INTEGER NOT NULL REFERENCES gatherings(id) ON DELETE CASCADE,
 member INTEGER NOT NULL REFERENCES members(id),
 response INTEGER NOT NULL CHECK(response IN(1,2)),
 PRIMARY KEY(gathering,member)
) STRICT;

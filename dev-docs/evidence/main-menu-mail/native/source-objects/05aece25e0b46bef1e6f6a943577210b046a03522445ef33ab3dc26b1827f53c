CREATE TABLE social_settings (
 singleton INTEGER PRIMARY KEY CHECK(singleton=1),
 enabled INTEGER NOT NULL CHECK(enabled IN(0,1)),
 name TEXT NOT NULL CHECK(length(name) BETWEEN 1 AND 24)
) STRICT;
INSERT INTO social_settings VALUES(1,0,'Local lounge');
CREATE TABLE social_prefs (
 member INTEGER PRIMARY KEY REFERENCES members(id) ON DELETE CASCADE,
 muted INTEGER NOT NULL DEFAULT 0 CHECK(muted IN(0,1)),
 quiet_start INTEGER CHECK(quiet_start BETWEEN 0 AND 1439),
 quiet_end INTEGER CHECK(quiet_end BETWEEN 0 AND 1439),
 CHECK((quiet_start IS NULL AND quiet_end IS NULL) OR
       (quiet_start IS NOT NULL AND quiet_end IS NOT NULL AND quiet_start<>quiet_end))
) STRICT;
CREATE TABLE social_blocks (
 member INTEGER NOT NULL REFERENCES members(id) ON DELETE CASCADE,
 handle TEXT NOT NULL CHECK(length(handle) BETWEEN 1 AND 24),
 PRIMARY KEY(member,handle)
) STRICT;
CREATE TABLE social_presence (
 session BLOB PRIMARY KEY REFERENCES chat_sessions(id) ON DELETE CASCADE,
 member INTEGER NOT NULL UNIQUE REFERENCES members(id) ON DELETE CASCADE,
 mode TEXT NOT NULL CHECK(mode IN('available','busy','away')),
 status TEXT NOT NULL CHECK(length(status)<=48)
) STRICT;
CREATE TABLE social_pages (
 id INTEGER PRIMARY KEY AUTOINCREMENT,
 sender BLOB NOT NULL REFERENCES social_presence(session) ON DELETE CASCADE,
 recipient BLOB NOT NULL REFERENCES social_presence(session) ON DELETE CASCADE,
 nonce TEXT NOT NULL CHECK(length(nonce)=32),
 created INTEGER NOT NULL CHECK(created>=0),
 state TEXT NOT NULL CHECK(state IN('pending','accepted','declined')),
 UNIQUE(sender,nonce),CHECK(sender<>recipient)
) STRICT;
CREATE INDEX social_page_recipient ON social_pages(recipient,id);

CREATE TABLE social_cooldowns (
 member INTEGER PRIMARY KEY REFERENCES members(id) ON DELETE CASCADE,
 last_page INTEGER NOT NULL CHECK(last_page>=0)
) STRICT;

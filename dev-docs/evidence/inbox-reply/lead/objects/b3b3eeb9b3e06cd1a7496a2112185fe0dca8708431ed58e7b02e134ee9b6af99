CREATE TABLE catchup_settings (
 singleton INTEGER PRIMARY KEY CHECK(singleton=1),
 enabled INTEGER NOT NULL CHECK(enabled IN(0,1)),
 name TEXT NOT NULL CHECK(length(name) BETWEEN 1 AND 24),
 compact INTEGER NOT NULL CHECK(compact IN(0,1)),
 modules TEXT NOT NULL CHECK(length(modules)<=64),
 notice TEXT NOT NULL CHECK(length(CAST(notice AS BLOB))<=240)
) STRICT;
INSERT INTO catchup_settings VALUES(1,0,'Catch up',0,'notice,replies,rooms,files,gatherings','');
CREATE TABLE catchup_members (
 member INTEGER PRIMARY KEY REFERENCES members(id) ON DELETE CASCADE,
 file_cursor INTEGER NOT NULL CHECK(file_cursor>=0),
 saved_at INTEGER NOT NULL CHECK(saved_at>=0),
 revision TEXT NOT NULL CHECK(length(revision)=32),
 favorites_only INTEGER NOT NULL CHECK(favorites_only IN(0,1))
) STRICT;
CREATE TABLE catchup_favorites (
 member INTEGER NOT NULL REFERENCES catchup_members(member) ON DELETE CASCADE,
 room INTEGER NOT NULL REFERENCES rooms(id) ON DELETE CASCADE,
 PRIMARY KEY(member,room)
) STRICT;
CREATE TABLE catchup_publications (
 id INTEGER PRIMARY KEY AUTOINCREMENT,
 object TEXT NOT NULL UNIQUE REFERENCES file_objects(id) ON DELETE CASCADE,
 published_at INTEGER NOT NULL CHECK(published_at>=0)
) STRICT;
INSERT INTO catchup_publications(object,published_at) SELECT id,0 FROM file_objects WHERE state='published' ORDER BY rowid;
CREATE TRIGGER catchup_file_insert AFTER INSERT ON file_objects WHEN NEW.state='published' BEGIN
 INSERT INTO catchup_publications(object,published_at) VALUES(NEW.id,unixepoch());
END;
CREATE TRIGGER catchup_file_publish AFTER UPDATE OF state ON file_objects WHEN NEW.state='published' AND OLD.state<>'published' BEGIN
 INSERT OR IGNORE INTO catchup_publications(object,published_at) VALUES(NEW.id,unixepoch());
END;

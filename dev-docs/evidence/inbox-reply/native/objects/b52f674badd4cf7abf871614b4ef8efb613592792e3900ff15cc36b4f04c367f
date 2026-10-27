CREATE TABLE discussion_settings (
 singleton INTEGER PRIMARY KEY CHECK(singleton=1),
 enabled INTEGER NOT NULL CHECK(enabled IN(0,1)),
 name TEXT NOT NULL CHECK(length(name) BETWEEN 1 AND 24),
 compact INTEGER NOT NULL CHECK(compact IN(0,1))
) STRICT;
INSERT INTO discussion_settings VALUES(1,0,'Find & save',0);
CREATE INDEX discussion_author_posts ON posts(author_id,id);
CREATE TABLE discussion_bookmarks (
 member INTEGER NOT NULL REFERENCES members(id) ON DELETE CASCADE,
 post INTEGER NOT NULL REFERENCES posts(id) ON DELETE CASCADE,
 PRIMARY KEY(member,post)
) STRICT;

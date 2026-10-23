CREATE TABLE magazine_submissions (
 id INTEGER PRIMARY KEY AUTOINCREMENT,
 author INTEGER NOT NULL REFERENCES members(id),
 nonce TEXT NOT NULL CHECK(length(nonce)=32),
 revision TEXT NOT NULL CHECK(length(revision)=32),
 state TEXT NOT NULL CHECK(state IN('draft','submitted','withdrawn')),
 title TEXT NOT NULL CHECK(length(title) BETWEEN 1 AND 64),
 section TEXT NOT NULL CHECK(length(section) BETWEEN 1 AND 24),
 rights TEXT NOT NULL CHECK(length(rights) BETWEEN 1 AND 96),
 provenance TEXT NOT NULL CHECK(length(provenance) BETWEEN 1 AND 128),
 body TEXT NOT NULL CHECK(length(body) BETWEEN 1 AND 4096),
 UNIQUE(author,nonce)
) STRICT;
CREATE INDEX magazine_submission_author ON magazine_submissions(author,id);
CREATE TABLE magazine_issues (
 id INTEGER PRIMARY KEY AUTOINCREMENT,
 editor INTEGER NOT NULL REFERENCES members(id),
 nonce TEXT NOT NULL UNIQUE CHECK(length(nonce)=32),
 token TEXT NOT NULL CHECK(length(token)=32),
 number INTEGER NOT NULL CHECK(number BETWEEN 1 AND 9999),
 edition INTEGER NOT NULL CHECK(edition BETWEEN 1 AND 9999),
 name TEXT NOT NULL CHECK(length(name) BETWEEN 1 AND 24),
 title TEXT NOT NULL CHECK(length(title) BETWEEN 1 AND 64),
 released TEXT NOT NULL CHECK(length(released) BETWEEN 1 AND 32),
 schedule TEXT NOT NULL CHECK(length(schedule) BETWEEN 1 AND 64),
 state TEXT NOT NULL CHECK(state IN('draft','published','withdrawn')),
 object TEXT CHECK(object IS NULL OR length(object)=32),
 UNIQUE(number,edition)
) STRICT;
CREATE TABLE magazine_entries (
 issue INTEGER NOT NULL REFERENCES magazine_issues(id) ON DELETE CASCADE,
 slot INTEGER NOT NULL CHECK(slot BETWEEN 1 AND 8),
 submission INTEGER NOT NULL REFERENCES magazine_submissions(id),
 source_revision TEXT NOT NULL CHECK(length(source_revision)=32),
 approved TEXT CHECK(approved IS NULL OR length(approved)=32),
 title TEXT NOT NULL CHECK(length(title) BETWEEN 1 AND 64),
 section TEXT NOT NULL CHECK(length(section) BETWEEN 1 AND 24),
 rights TEXT NOT NULL CHECK(length(rights) BETWEEN 1 AND 96),
 provenance TEXT NOT NULL CHECK(length(provenance) BETWEEN 1 AND 128),
 body TEXT NOT NULL CHECK(length(body) BETWEEN 1 AND 4096),
 PRIMARY KEY(issue,slot),UNIQUE(issue,submission)
) STRICT;
CREATE INDEX magazine_entry_source ON magazine_entries(submission,issue);

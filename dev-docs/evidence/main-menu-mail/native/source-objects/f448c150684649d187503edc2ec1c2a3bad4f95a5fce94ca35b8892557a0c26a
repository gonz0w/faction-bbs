CREATE TABLE directory_submissions (
 id INTEGER PRIMARY KEY CHECK(id>0),
 author INTEGER NOT NULL REFERENCES members(id),
 nonce TEXT NOT NULL,
 revision TEXT NOT NULL,
 state TEXT NOT NULL CHECK(state IN('draft','pending','published','expired')),
 slug TEXT NOT NULL,
 name TEXT NOT NULL,
 summary TEXT NOT NULL,
 host TEXT NOT NULL,
 port INTEGER NOT NULL CHECK(port BETWEEN 1 AND 65535),
 login TEXT NOT NULL,
 fingerprint TEXT,
 reviewed TEXT,
 valid_until TEXT,
 UNIQUE(author,nonce),
 CHECK((state IN('draft','pending') AND reviewed IS NULL AND valid_until IS NULL) OR
       (state IN('published','expired') AND reviewed IS NOT NULL AND valid_until IS NOT NULL))
) STRICT;

CREATE TABLE file_requests (
 id INTEGER PRIMARY KEY AUTOINCREMENT,
 creator INTEGER NOT NULL REFERENCES members(id),
 nonce TEXT NOT NULL CHECK(length(nonce)=32),
 revision TEXT NOT NULL CHECK(length(revision)=32),
 title TEXT NOT NULL CHECK(length(title) BETWEEN 1 AND 64),
 description TEXT NOT NULL CHECK(length(description) BETWEEN 1 AND 128),
 state TEXT NOT NULL CHECK(state IN('open','fulfilled','closed')),
 fulfiller INTEGER REFERENCES members(id),
 object TEXT CHECK(object IS NULL OR length(object)=32),
 rights TEXT NOT NULL CHECK(length(rights)<=96),
 provenance TEXT NOT NULL CHECK(length(provenance)<=128),
 UNIQUE(creator,nonce),
 CHECK((object IS NULL AND fulfiller IS NULL AND rights='' AND provenance='') OR
       (object IS NOT NULL AND fulfiller IS NOT NULL AND length(rights)>0 AND length(provenance)>0)),
 CHECK(state<>'fulfilled' OR object IS NOT NULL),
 CHECK(state<>'open' OR object IS NULL)
) STRICT;
CREATE INDEX file_requests_creator ON file_requests(creator,id);

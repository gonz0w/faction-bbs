CREATE TABLE roaming_issuers (
 id INTEGER PRIMARY KEY CHECK(id>0),
 name TEXT NOT NULL UNIQUE,
 ca_fingerprint TEXT NOT NULL UNIQUE,
 generation INTEGER NOT NULL CHECK(generation>0),
 enabled INTEGER NOT NULL CHECK(enabled IN(0,1))
) STRICT;
CREATE TABLE roaming_bindings (
 id INTEGER PRIMARY KEY CHECK(id>0),
 issuer_id INTEGER NOT NULL REFERENCES roaming_issuers(id),
 subject TEXT NOT NULL,
 principal TEXT NOT NULL,
 member_id INTEGER NOT NULL REFERENCES members(id),
 uid INTEGER NOT NULL UNIQUE CHECK(uid BETWEEN 1 AND 4294967295),
 consent TEXT NOT NULL,
 generation INTEGER NOT NULL CHECK(generation>0),
 enabled INTEGER NOT NULL CHECK(enabled IN(0,1))
) STRICT;
CREATE UNIQUE INDEX roaming_active_subject ON roaming_bindings(issuer_id,subject) WHERE enabled=1;
CREATE UNIQUE INDEX roaming_active_principal ON roaming_bindings(principal) WHERE enabled=1;
CREATE TABLE roaming_chat (
 session BLOB PRIMARY KEY REFERENCES chat_sessions(id) ON DELETE CASCADE,
 binding_id INTEGER NOT NULL REFERENCES roaming_bindings(id),
 binding_generation INTEGER NOT NULL CHECK(binding_generation>0),
 issuer_id INTEGER NOT NULL REFERENCES roaming_issuers(id),
 issuer_generation INTEGER NOT NULL CHECK(issuer_generation>0)
) STRICT;
CREATE TABLE roaming_transfers (
 transfer TEXT PRIMARY KEY REFERENCES file_transfers(id) ON DELETE CASCADE,
 binding_id INTEGER NOT NULL REFERENCES roaming_bindings(id),
 binding_generation INTEGER NOT NULL CHECK(binding_generation>0),
 issuer_id INTEGER NOT NULL REFERENCES roaming_issuers(id),
 issuer_generation INTEGER NOT NULL CHECK(issuer_generation>0)
) STRICT;

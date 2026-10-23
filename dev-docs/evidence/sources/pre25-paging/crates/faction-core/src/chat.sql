CREATE TABLE chat_channels (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    private INTEGER NOT NULL CHECK(private IN (0,1)),
    sequence INTEGER NOT NULL DEFAULT 0 CHECK(sequence >= 0)
);
INSERT INTO chat_channels(name,private) VALUES('#lobby',0);
CREATE TABLE chat_grants (
    channel INTEGER NOT NULL REFERENCES chat_channels(id) ON DELETE CASCADE,
    member INTEGER NOT NULL REFERENCES members(id) ON DELETE CASCADE,
    PRIMARY KEY(channel,member)
);
CREATE TABLE chat_sessions (
    id BLOB PRIMARY KEY NOT NULL CHECK(length(id)=16),
    member INTEGER NOT NULL REFERENCES members(id) ON DELETE CASCADE,
    epoch INTEGER NOT NULL,
    touched INTEGER NOT NULL,
    last_request INTEGER NOT NULL DEFAULT 0 CHECK(last_request >= 0),
    last_channel INTEGER, last_sequence INTEGER, last_floor INTEGER, last_body TEXT, last_kind TEXT
);
CREATE TABLE chat_joins (
    session BLOB NOT NULL REFERENCES chat_sessions(id) ON DELETE CASCADE,
    channel INTEGER NOT NULL REFERENCES chat_channels(id) ON DELETE CASCADE,
    floor INTEGER NOT NULL CHECK(floor >= 0),
    PRIMARY KEY(session,channel)
);
CREATE TABLE chat_messages (
    channel INTEGER NOT NULL REFERENCES chat_channels(id) ON DELETE CASCADE,
    sequence INTEGER NOT NULL CHECK(sequence > 0),
    author INTEGER NOT NULL REFERENCES members(id),
    body TEXT NOT NULL CHECK(length(CAST(body AS BLOB)) BETWEEN 1 AND 512),
    kind TEXT NOT NULL CHECK(kind IN ('say','emote')),
    PRIMARY KEY(channel,sequence)
);
CREATE INDEX chat_member_sessions ON chat_sessions(member);

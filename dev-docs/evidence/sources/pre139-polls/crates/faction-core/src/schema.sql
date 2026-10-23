CREATE TABLE board (
    singleton INTEGER PRIMARY KEY CHECK(singleton=1),
    identity BLOB NOT NULL CHECK(length(identity)=32)
);
CREATE TABLE members (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    handle TEXT NOT NULL UNIQUE COLLATE NOCASE,
    password_hash TEXT NOT NULL,
    role TEXT NOT NULL CHECK(role IN ('sysop','member')),
    status TEXT NOT NULL CHECK(status IN ('pending','approved','revoked')),
    epoch INTEGER NOT NULL DEFAULT 1 CHECK(epoch > 0)
);
CREATE TABLE rooms (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    slug TEXT NOT NULL UNIQUE,
    title TEXT NOT NULL
);
CREATE TABLE posts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    room_id INTEGER NOT NULL REFERENCES rooms(id),
    author_id INTEGER NOT NULL REFERENCES members(id),
    request_id TEXT NOT NULL,
    body TEXT NOT NULL,
    reply_to INTEGER REFERENCES posts(id),
    UNIQUE(author_id,request_id)
);
CREATE INDEX room_posts ON posts(room_id,id);
CREATE TABLE unread (
    member_id INTEGER NOT NULL REFERENCES members(id),
    room_id INTEGER NOT NULL REFERENCES rooms(id),
    last_read INTEGER NOT NULL CHECK(last_read >= 0),
    PRIMARY KEY(member_id,room_id)
);

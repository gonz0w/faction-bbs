CREATE TABLE room_drafts (
    member_id INTEGER NOT NULL REFERENCES members(id),
    room_id INTEGER NOT NULL REFERENCES rooms(id),
    epoch INTEGER NOT NULL CHECK(epoch>0),
    request_id TEXT NOT NULL CHECK(length(request_id)=32),
    subject TEXT NOT NULL CHECK(length(CAST(subject AS BLOB))<=160),
    body TEXT NOT NULL CHECK(length(CAST(body AS BLOB))<=1024),
    reply_to INTEGER REFERENCES posts(id),
    updated_at INTEGER NOT NULL CHECK(updated_at BETWEEN 0 AND 253402300799),
    PRIMARY KEY(member_id,room_id),
    UNIQUE(member_id,request_id)
);

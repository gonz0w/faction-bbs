CREATE TABLE visit_activity (
 session BLOB PRIMARY KEY REFERENCES chat_sessions(id) ON DELETE CASCADE,
 activity TEXT NOT NULL CHECK(activity IN('Menu','Rooms','Chat','Doors','Files','Lounge','Who','Settings','Community','Gallery','Library','Directory'))
) STRICT;

CREATE TABLE file_areas (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    slug TEXT NOT NULL UNIQUE,
    title TEXT NOT NULL,
    private INTEGER NOT NULL CHECK(private IN (0,1)),
    quota INTEGER NOT NULL CHECK(quota BETWEEN 1 AND 268435456)
);
CREATE TABLE file_area_grants (
    area_id INTEGER NOT NULL REFERENCES file_areas(id),
    member_id INTEGER NOT NULL REFERENCES members(id),
    PRIMARY KEY(area_id,member_id)
);
CREATE TABLE file_objects (
    id TEXT PRIMARY KEY CHECK(length(id)=32),
    area_id INTEGER NOT NULL REFERENCES file_areas(id),
    uploader_id INTEGER NOT NULL REFERENCES members(id),
    filename TEXT NOT NULL,
    description TEXT NOT NULL,
    bytes INTEGER NOT NULL CHECK(bytes BETWEEN 0 AND 16777216),
    sha256 TEXT NOT NULL DEFAULT '',
    state TEXT NOT NULL CHECK(state IN ('reserved','pending','published','removed'))
);
CREATE INDEX files_by_area ON file_objects(area_id,id);
CREATE TABLE file_transfers (
    id TEXT PRIMARY KEY CHECK(length(id)=32),
    object_id TEXT NOT NULL REFERENCES file_objects(id) ON DELETE CASCADE,
    member_id INTEGER NOT NULL REFERENCES members(id),
    epoch INTEGER NOT NULL CHECK(epoch>0),
    ssh_key TEXT NOT NULL,
    direction TEXT NOT NULL CHECK(direction IN ('upload','download')),
    created INTEGER NOT NULL CHECK(created>0),
    expires INTEGER NOT NULL CHECK(expires>created),
    state TEXT NOT NULL CHECK(state IN ('ready','active','done','failed')),
    lease TEXT NOT NULL DEFAULT '',
    progress INTEGER NOT NULL DEFAULT 0 CHECK(progress>=0)
);
CREATE UNIQUE INDEX file_live_key ON file_transfers(ssh_key) WHERE state IN ('ready','active');

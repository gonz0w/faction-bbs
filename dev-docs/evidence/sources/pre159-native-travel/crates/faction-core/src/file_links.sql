CREATE TABLE file_namespace (
    singleton INTEGER PRIMARY KEY CHECK(singleton=1),
    id TEXT NOT NULL CHECK(length(id)=32)
);

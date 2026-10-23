CREATE TABLE post_metadata (
    post_id INTEGER PRIMARY KEY REFERENCES posts(id),
    article_id TEXT NOT NULL UNIQUE CHECK(length(article_id)=32 AND article_id NOT GLOB '*[^0-9a-f]*'),
    subject TEXT NOT NULL CHECK(length(subject) BETWEEN 1 AND 160),
    created_at INTEGER,
    origin TEXT NOT NULL CHECK(origin IN ('legacy','local')),
    CHECK((origin='legacy' AND created_at IS NULL) OR (origin='local' AND created_at BETWEEN 0 AND 253402300799 AND created_at IS NOT NULL))
);
INSERT INTO post_metadata(post_id,article_id,subject,created_at,origin)
    SELECT id,lower(hex(randomblob(16))),'Legacy message #' || id,NULL,'legacy' FROM posts;

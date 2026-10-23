//! Owner-only local data movement. Never delegate these APIs to a member adapter.
use crate::{APP_ID, Board, Error, Result, handle, private, request_id, text};
use rusqlite::{
    Connection, OpenFlags,
    backup::{Backup, StepResult},
    config::DbConfig,
    limits::Limit,
};
use std::{
    fs,
    io::Read,
    os::unix::fs::{DirBuilderExt, OpenOptionsExt},
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

/// Initial local tooling bound, not a supported board size or a memory allowance.
pub const SNAPSHOT_BYTES: u64 = 256 * 1024 * 1024;
const BUDGET: Duration = Duration::from_secs(30);
const INFO: &str = "CREATE TABLE faction_export (singleton INTEGER PRIMARY KEY CHECK(singleton=1), format INTEGER NOT NULL CHECK(format=1))";

#[derive(Debug, PartialEq, Eq)]
pub struct SnapshotSummary {
    pub members: i64,
    pub rooms: i64,
    pub posts: i64,
    pub channels: i64,
    pub chat_messages: i64,
    pub game_scores: i64,
}

fn harden(db: &Connection, deadline: Instant) -> Result<()> {
    db.busy_timeout(Duration::from_millis(100))?;
    db.set_db_config(DbConfig::SQLITE_DBCONFIG_DEFENSIVE, true)?;
    db.set_db_config(DbConfig::SQLITE_DBCONFIG_TRUSTED_SCHEMA, false)?;
    for (limit, value) in [
        (Limit::SQLITE_LIMIT_LENGTH, 65536),
        (Limit::SQLITE_LIMIT_SQL_LENGTH, 32768),
        (Limit::SQLITE_LIMIT_COLUMN, 64),
        (Limit::SQLITE_LIMIT_EXPR_DEPTH, 64),
        (Limit::SQLITE_LIMIT_COMPOUND_SELECT, 8),
        (Limit::SQLITE_LIMIT_TRIGGER_DEPTH, 8),
        (Limit::SQLITE_LIMIT_WORKER_THREADS, 0),
    ] {
        db.set_limit(limit, value)?;
    }
    db.progress_handler(1000, Some(move || Instant::now() >= deadline))?;
    db.execute_batch("PRAGMA foreign_keys=ON; PRAGMA cache_size=-2048; PRAGMA mmap_size=0; PRAGMA synchronous=FULL; PRAGMA secure_delete=ON;")?;
    Ok(())
}
fn budget(deadline: Instant) -> Result<()> {
    if Instant::now() >= deadline {
        Err(Error::Invalid(
            "snapshot work budget exceeded; retry offline or with a smaller store",
        ))
    } else {
        Ok(())
    }
}
fn size(db: &Connection) -> Result<()> {
    let pages: u32 = db.query_row("PRAGMA page_count", [], |r| r.get(0))?;
    let bytes: u32 = db.query_row("PRAGMA page_size", [], |r| r.get(0))?;
    if u64::from(pages)
        .checked_mul(u64::from(bytes))
        .is_none_or(|n| n > SNAPSHOT_BYTES)
    {
        return Err(Error::Invalid("snapshot exceeds the initial 256 MiB limit"));
    }
    Ok(())
}
fn copy(from: &Connection, to: &mut Connection, deadline: Instant) -> Result<()> {
    size(from)?;
    let page_size: u32 = from.query_row("PRAGMA page_size", [], |r| r.get(0))?;
    let backup = Backup::new(from, to)?;
    loop {
        budget(deadline)?;
        let step = backup.step(32)?;
        let count = backup.progress().pagecount;
        if count < 0 || (count as u64).saturating_mul(u64::from(page_size)) > SNAPSHOT_BYTES {
            return Err(Error::Invalid("snapshot exceeds the initial 256 MiB limit"));
        }
        match step {
            StepResult::Done => return Ok(()),
            StepResult::More => (),
            StepResult::Busy | StepResult::Locked => return Err(Error::Busy),
            _ => return Err(Error::Invalid("snapshot copy did not complete")),
        }
    }
}

type Schema = Vec<(String, String, String, Option<String>)>;
fn schema(db: &Connection) -> Result<Schema> {
    Ok(db
        .prepare("SELECT type,name,tbl_name,sql FROM sqlite_schema ORDER BY type,name LIMIT 65")?
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)))?
        .collect::<std::result::Result<_, _>>()?)
}
fn validate(db: &Connection, exported: bool, deadline: Instant) -> Result<SnapshotSummary> {
    budget(deadline)?;
    size(db)?;
    let app: i64 = db.query_row("PRAGMA application_id", [], |r| r.get(0))?;
    let version: i64 = db.query_row("PRAGMA user_version", [], |r| r.get(0))?;
    if app != APP_ID || !(2..=5).contains(&version) {
        return Err(Error::UnsupportedSchema);
    }
    // Never run an imported SQL script. Only this exact checked-in catalog
    // is eligible; reject extra views/triggers/virtual tables/unknown migrations.
    let reference = Connection::open_in_memory()?;
    reference.execute_batch(include_str!("schema.sql"))?;
    reference.execute_batch(include_str!("chat.sql"))?;
    if version >= 3 {
        reference.execute_batch(include_str!("scores.sql"))?;
    }
    if version >= 4 {
        reference.execute_batch(include_str!("files.sql"))?;
    }
    if version == 5 {
        reference.execute_batch(include_str!("file_links.sql"))?;
    }
    if exported {
        reference.execute_batch(INFO)?;
    }
    if schema(db)? != schema(&reference)? {
        return Err(Error::UnsupportedSchema);
    }
    let integrity: String = db.query_row("PRAGMA integrity_check(1)", [], |r| r.get(0))?;
    if integrity != "ok" || db.prepare("PRAGMA foreign_key_check")?.exists([])? {
        return Err(Error::Invalid(
            "snapshot integrity or foreign-key check failed",
        ));
    }
    if exported {
        let n: i64 = db.query_row(
            "SELECT count(*) FROM faction_export WHERE singleton=1 AND format=1",
            [],
            |r| r.get(0),
        )?;
        if n != 1
            || db
                .prepare("SELECT 1 FROM chat_sessions UNION ALL SELECT 1 FROM chat_joins LIMIT 1")?
                .exists([])?
        {
            return Err(Error::Invalid(
                "invalid snapshot format or live session data",
            ));
        }
    }
    let identity: Vec<u8> =
        db.query_row("SELECT identity FROM board WHERE singleton=1", [], |r| {
            r.get(0)
        })?;
    let board_count: i64 = db.query_row("SELECT count(*) FROM board", [], |r| r.get(0))?;
    if identity.len() != 32 || board_count != 1 {
        return Err(Error::UnsupportedSchema);
    }
    let mut stmt = db.prepare("SELECT id,handle,password_hash,role,status,epoch FROM members")?;
    let mut rows = stmt.query([])?;
    let mut sysop = false;
    while let Some(row) = rows.next()? {
        budget(deadline)?;
        let id: i64 = row.get(0)?;
        let name: String = row.get(1)?;
        let hash: String = row.get(2)?;
        let role: String = row.get(3)?;
        let status: String = row.get(4)?;
        let epoch: i64 = row.get(5)?;
        // Fixed production KDF parameters prevent imported hashes requesting arbitrary
        // memory/CPU on later login. Parse only, never run the password KDF here.
        let parsed = argon2::PasswordHash::new(&hash).map_err(|_| Error::PasswordHash)?;
        if id < 1
            || handle(&name)? != name
            || epoch < 1
            || parsed.algorithm.as_str() != "argon2id"
            || parsed.version != Some(19)
            || parsed.params.to_string() != "m=19456,t=2,p=1"
            || parsed.hash.is_none_or(|h| h.len() != 32)
            || parsed.salt.is_none_or(|s| s.len() != 16)
            || !["member", "sysop"].contains(&role.as_str())
            || !["pending", "approved", "revoked"].contains(&status.as_str())
        {
            return Err(Error::Invalid("snapshot contains an invalid account"));
        }
        sysop |= role == "sysop" && status == "approved";
    }
    if !sysop {
        return Err(Error::Invalid("snapshot needs an approved sysop"));
    }
    let mut stmt = db.prepare("SELECT id,slug,title FROM rooms")?;
    let mut rows = stmt.query([])?;
    while let Some(row) = rows.next()? {
        budget(deadline)?;
        let id: i64 = row.get(0)?;
        let slug: String = row.get(1)?;
        let title: String = row.get(2)?;
        if id < 1
            || slug.is_empty()
            || slug.len() > 32
            || !slug
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
        {
            return Err(Error::Invalid("invalid snapshot room"));
        }
        text(&title, 64, false)?;
    }
    let mut stmt = db.prepare("SELECT id,request_id,body FROM posts")?;
    let mut rows = stmt.query([])?;
    while let Some(row) = rows.next()? {
        budget(deadline)?;
        let id: i64 = row.get(0)?;
        let request: String = row.get(1)?;
        let body: String = row.get(2)?;
        if id < 1
            || !(16..=64).contains(&request.len())
            || !request
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
        {
            return Err(Error::Invalid("invalid snapshot post"));
        }
        text(&body, 1024, true)?;
    }
    for query in [
        "SELECT 1 FROM posts p JOIN posts r ON r.id=p.reply_to WHERE p.room_id!=r.room_id LIMIT 1",
        "SELECT 1 FROM unread u WHERE last_read!=0 AND NOT EXISTS(SELECT 1 FROM posts p WHERE p.id=u.last_read AND p.room_id=u.room_id) LIMIT 1",
        "SELECT 1 FROM chat_messages m JOIN chat_channels c ON c.id=m.channel WHERE m.sequence>c.sequence LIMIT 1",
        "SELECT 1 FROM chat_messages GROUP BY channel HAVING count(*)>64 LIMIT 1",
    ] {
        if db.prepare(query)?.exists([])? {
            return Err(Error::Invalid(
                "snapshot contains invalid record relationships",
            ));
        }
    }
    let mut stmt = db.prepare("SELECT name FROM chat_channels")?;
    let mut rows = stmt.query([])?;
    let mut channel_count = 0;
    let mut lobby = false;
    while let Some(row) = rows.next()? {
        let name: String = row.get(0)?;
        if !(2..=32).contains(&name.len())
            || !name.starts_with('#')
            || !name[1..]
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_' || b == b'-')
        {
            return Err(Error::Invalid("invalid snapshot channel"));
        }
        channel_count += 1;
        lobby |= name == "#lobby";
    }
    if channel_count > 32 || !lobby {
        return Err(Error::Invalid("snapshot channel limits exceeded"));
    }
    let mut stmt = db.prepare("SELECT body FROM chat_messages")?;
    let mut rows = stmt.query([])?;
    while let Some(row) = rows.next()? {
        text(&row.get::<_, String>(0)?, 512, false)?;
    }
    if version >= 3 {
        let mut stmt = db.prepare("SELECT member_id,game,best,plays FROM game_scores")?;
        let mut rows = stmt.query([])?;
        while let Some(row) = rows.next()? {
            budget(deadline)?;
            let member: i64 = row.get(0)?;
            crate::scores::game_slug(&row.get::<_, String>(1)?)?;
            let best: i64 = row.get(2)?;
            let plays: i64 = row.get(3)?;
            if member < 1
                || !(0..=i64::from(crate::SCORE_MAX)).contains(&best)
                || !(1..=i64::from(crate::SCORE_MAX)).contains(&plays)
            {
                return Err(Error::Invalid("invalid snapshot score"));
            }
        }
        if db
            .prepare("SELECT 1 FROM game_scores GROUP BY member_id HAVING count(*)>64 LIMIT 1")?
            .exists([])?
        {
            return Err(Error::Invalid("snapshot game limit exceeded"));
        }
    }
    if version >= 4 {
        crate::files::validate_catalog(db)?;
        if version == 5 {
            crate::file_links::validate_namespace(db)?;
        }
        if exported
            && db
                .prepare("SELECT 1 FROM file_transfers LIMIT 1")?
                .exists([])?
        {
            return Err(Error::Invalid("file grants must not survive export"));
        }
    }
    fn count(db: &Connection, sql: &str) -> Result<i64> {
        Ok(db.query_row(sql, [], |r| r.get(0))?)
    }
    budget(deadline)?;
    Ok(SnapshotSummary {
        members: count(db, "SELECT count(*) FROM members")?,
        rooms: count(db, "SELECT count(*) FROM rooms")?,
        posts: count(db, "SELECT count(*) FROM posts")?,
        channels: channel_count,
        chat_messages: count(db, "SELECT count(*) FROM chat_messages")?,
        game_scores: if version >= 3 {
            count(db, "SELECT count(*) FROM game_scores")?
        } else {
            0
        },
    })
}

/// Private same-filesystem staging. All parents must be trusted by the owner.
struct Stage(PathBuf);
impl Stage {
    fn new(parent: &Path) -> Result<Self> {
        private(parent, true)?;
        let dir = parent.join(format!(".faction-stage-{}", request_id()?));
        fs::DirBuilder::new().mode(0o700).create(&dir)?;
        Ok(Self(dir))
    }
    fn path(&self) -> PathBuf {
        self.0.join("snapshot.sqlite3")
    }
    fn connection(&self, deadline: Instant) -> Result<Connection> {
        fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(self.path())?;
        let db = Connection::open_with_flags(
            fs::canonicalize(&self.0)?.join("snapshot.sqlite3"),
            OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_NOFOLLOW,
        )?;
        harden(&db, deadline)?;
        Ok(db)
    }
}
impl Drop for Stage {
    fn drop(&mut self) {
        // No recursive deletion of arbitrary paths, even after an I/O failure.
        for suffix in ["", "-wal", "-shm", "-journal"] {
            let _ = fs::remove_file(self.0.join(format!("snapshot.sqlite3{suffix}")));
        }
        let _ = fs::remove_dir(&self.0);
    }
}
fn parent(path: &Path) -> Result<&Path> {
    if path.file_name().is_none() {
        return Err(Error::Invalid("a new named path is required"));
    }
    let p = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    private(p, true)?;
    Ok(p)
}
fn sync(path: &Path) -> Result<()> {
    fs::File::open(path)?.sync_all()?;
    Ok(())
}
fn finish(db: Connection, stage: &Stage, deadline: Instant) -> Result<()> {
    // Backup may copy a WAL-mode header; convert only the private destination.
    db.execute_batch("PRAGMA journal_mode=DELETE; VACUUM;")?;
    size(&db)?;
    budget(deadline)?;
    db.close().map_err(|(_, e)| Error::from(e))?;
    sync(&stage.path())?;
    Ok(())
}
fn source(path: &Path, deadline: Instant) -> Result<Connection> {
    parent(path)?;
    private(path, false)?;
    let mut file = fs::File::open(path)?;
    if file.metadata()?.len() > SNAPSHOT_BYTES {
        return Err(Error::Invalid("snapshot exceeds the initial 256 MiB limit"));
    }
    let mut header = [0; 100];
    file.read_exact(&mut header)?;
    if &header[..16] != b"SQLite format 3\0" || header[18] != 1 || header[19] != 1 {
        return Err(Error::Invalid(
            "snapshot must be a standalone rollback-journal SQLite file",
        ));
    }
    for suffix in ["-wal", "-shm", "-journal"] {
        let mut side = path.as_os_str().to_os_string();
        side.push(suffix);
        match fs::symlink_metadata(Path::new(&side)) {
            Ok(_) => {
                return Err(Error::Invalid(
                    "snapshot sidecars are forbidden; use an exported file",
                ));
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => (),
            Err(e) => return Err(e.into()),
        }
    }
    let db = Connection::open_with_flags(
        fs::canonicalize(parent(path)?)?.join(path.file_name().ok_or(Error::UnsafeState)?),
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NOFOLLOW,
    )?;
    harden(&db, deadline)?;
    db.execute_batch("BEGIN DEFERRED")?;
    Ok(db)
}

impl Board {
    /// Filesystem-owner operation: includes account hashes and all local private data.
    /// Does not authenticate a member, publish anything, or include external theme files.
    pub fn export_local(&self, output: &Path) -> Result<SnapshotSummary> {
        export_connection(&self.db, output, Instant::now() + BUDGET)
    }
}

/// Inspect a private export without modifying it or creating a board directory.
pub fn verify_export(input: &Path) -> Result<SnapshotSummary> {
    let deadline = Instant::now() + BUDGET;
    validate(&source(input, deadline)?, true, deadline)
}

/// Restore into a NEW directory only. Never replace/merge a running or existing board.
pub fn restore_local(input: &Path, destination: &Path) -> Result<SnapshotSummary> {
    let deadline = Instant::now() + BUDGET;
    let p = parent(destination)?;
    let source = source(input, deadline)?;
    let summary = validate(&source, true, deadline)?;
    let stage = Stage::new(p)?;
    let mut db = stage.connection(deadline)?;
    copy(&source, &mut db, deadline)?;
    validate(&db, true, deadline)?;
    let mut identity = [0; 32];
    getrandom::fill(&mut identity).map_err(|_| Error::Entropy)?;
    db.execute(
        "UPDATE board SET identity=?1 WHERE singleton=1",
        [identity.as_slice()],
    )?;
    db.execute_batch("DROP TABLE faction_export;")?;
    validate(&db, false, deadline)?;
    finish(db, &stage, deadline)?;
    // Reserve a new name only after validation. It is briefly an empty directory;
    // operators must not open it concurrently. A competing opener still cannot be
    // overwritten: publication is a no-clobber hard link to the complete database.
    fs::DirBuilder::new().mode(0o700).create(destination)?;
    if let Err(e) = fs::hard_link(stage.path(), destination.join("faction.sqlite3")) {
        let _ = fs::remove_dir(destination); // only succeeds when still empty
        return Err(e.into());
    }
    sync(destination)?;
    sync(p)?;
    Ok(summary)
}

/// Open an existing store read-only for an owner export; never initialize/migrate it.
pub fn export_local(directory: &Path, output: &Path) -> Result<SnapshotSummary> {
    private(directory, true)?;
    let path = directory.join("faction.sqlite3");
    private(&path, false)?;
    for suffix in ["-wal", "-shm", "-journal"] {
        let side = directory.join(format!("faction.sqlite3{suffix}"));
        match fs::symlink_metadata(&side) {
            Ok(_) => private(&side, false)?,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => (),
            Err(e) => return Err(e.into()),
        }
    }
    let deadline = Instant::now() + BUDGET;
    let db = Connection::open_with_flags(
        fs::canonicalize(directory)?.join("faction.sqlite3"),
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NOFOLLOW,
    )?;
    harden(&db, deadline)?;
    db.execute_batch("BEGIN DEFERRED")?;
    export_connection(&db, output, deadline)
}

fn export_connection(
    source: &Connection,
    output: &Path,
    deadline: Instant,
) -> Result<SnapshotSummary> {
    let p = parent(output)?;
    let stage = Stage::new(p)?;
    let mut db = stage.connection(deadline)?;
    copy(source, &mut db, deadline)?;
    // Validate the copied catalog before any writes that could fire imported SQL.
    validate(&db, false, deadline)?;
    db.execute_batch("DELETE FROM chat_sessions;")?;
    let version: i64 = db.query_row("PRAGMA user_version", [], |r| r.get(0))?;
    if version >= 4 {
        db.execute_batch("DELETE FROM file_transfers;")?;
    }
    db.execute_batch(INFO)?;
    db.execute("INSERT INTO faction_export VALUES(1,1)", [])?;
    let summary = validate(&db, true, deadline)?;
    finish(db, &stage, deadline)?;
    // hard_link publishes one complete file atomically and cannot overwrite a
    // regular file, symlink or race winner. Staging is on the same filesystem.
    fs::hard_link(stage.path(), output)?;
    sync(p)?;
    Ok(summary)
}

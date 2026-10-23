//! Local Faction identity and chronological rooms. No network/terminal authority.
#![forbid(unsafe_code)]
#[cfg(not(unix))]
compile_error!("The first local core supports Unix state ownership semantics only");
mod catchup;
mod chat;
pub use catchup::{
    CatchupConfig, CatchupItem, CatchupModule, CatchupPage, CatchupPrefs, CatchupTarget,
    CatchupWindow,
};
mod requests;
pub use requests::{FileRequest, FileRequestDraft, FileRequestOffer};
mod discussion;
pub use discussion::{
    DiscussionConfig, DiscussionField, DiscussionHit, DiscussionItem, DiscussionPage,
    DiscussionQuery,
};
mod credentials;
mod file_links;
mod files;
mod gatherings;
mod polls;
pub use polls::{Poll, PollConfig, PollDraft, PollResults, PollState};
mod member_pages;
pub use gatherings::{Gathering, GatheringConfig, GatheringDraft, GatheringPlace};
pub mod operator;
mod post_metadata;
mod room_article;
mod room_drafts;
mod wall;
pub use member_pages::{
    MemberPage, PAGE_FIELD_BYTES, PAGE_FIELDS, PageConfig, PlanUpdate, page_text,
};
pub use room_drafts::{ROOM_DRAFT_SECONDS, ROOM_DRAFTS, SavedRoomDraft};
pub use wall::{WallConfig, WallNote, WallPage, wall_text};
mod scores;
pub use files::{
    AreaFile, FILE_BOARD_BYTES, FILE_BYTES, FILE_CHUNK, FILE_ROWS, FileArea, TransferGrant,
    file_id, file_key, file_name,
};
pub use post_metadata::PostMetadata;
mod snapshot;
use argon2::{Algorithm, Argon2, Params, PasswordHasher, PasswordVerifier, Version};
pub use chat::{ChatChannel, ChatCommand, ChatKind, ChatLease, ChatMessage, ChatNames, ChatPage};
use rusqlite::{Connection, OptionalExtension, Transaction, TransactionBehavior, params};
pub use scores::{GameRun, GameScore, SCORE_GAMES, SCORE_MAX, SCORE_ROWS, ScoreUpdate};
pub use snapshot::{SNAPSHOT_BYTES, SnapshotSummary, export_local, restore_local, verify_export};
use std::{
    fmt, fs,
    os::unix::fs::{DirBuilderExt, MetadataExt, OpenOptionsExt},
    path::Path,
    time::Duration,
};

const APP_ID: i64 = 0x46414354;
/// Bounded posting input; empty subject selects the local default.
pub struct PostDraft<'a> {
    pub subject: &'a str,
    pub body: &'a str,
    pub reply_to: Option<i64>,
}
pub const POST_BYTES: usize = 1024;
pub const PAGE_BYTES: usize = 2048;
pub const PAGE_ROWS: usize = 20;
#[derive(Debug)]
pub enum Error {
    Invalid(&'static str),
    Denied,
    StaleSession,
    Credentials,
    NotFound,
    AlreadyInitialized,
    NotInitialized,
    RequestConflict,
    UnsafeState,
    UnsupportedSchema,
    Busy,
    CredentialBusy,
    Storage(rusqlite::Error),
    Io(std::io::Error),
    Entropy,
    PasswordHash,
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Invalid(reason) => reason,
            Self::Denied => "action denied",
            Self::StaleSession => "session expired; log in again",
            Self::Credentials => "invalid credentials",
            Self::NotFound => "not found",
            Self::AlreadyInitialized => "board already initialized",
            Self::NotInitialized => "sysop bootstrap required",
            Self::RequestConflict => "request ID already used for different content",
            Self::UnsafeState => {
                "state must be a private, owned directory and regular database file"
            }
            Self::UnsupportedSchema => "unsupported or foreign database schema",
            Self::Busy => "database busy; retry later",
            Self::CredentialBusy => "credential work busy; reconnect later",
            Self::Storage(_) => "database operation failed",
            Self::Io(_) => "state I/O failed",
            Self::Entropy => "OS entropy unavailable",
            Self::PasswordHash => "password hashing failed",
        })
    }
}
impl std::error::Error for Error {}
impl From<rusqlite::Error> for Error {
    fn from(e: rusqlite::Error) -> Self {
        if e.sqlite_error_code() == Some(rusqlite::ErrorCode::DatabaseBusy) {
            Self::Busy
        } else {
            Self::Storage(e)
        }
    }
}
impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}
pub type Result<T> = std::result::Result<T, Error>;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Pending,
    Approved,
    Revoked,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    Member,
    Sysop,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Member {
    pub handle: String,
    pub status: Status,
    pub role: Role,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Room {
    pub id: i64,
    pub slug: String,
    pub title: String,
    pub unread: i64,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Post {
    pub id: i64,
    pub room_id: i64,
    pub author: String,
    pub body: String,
    pub reply_to: Option<i64>,
}
/// Created only after password verification; cannot be serialized or transplanted between boards.
pub struct Session {
    board: [u8; 32],
    member: i64,
    epoch: i64,
}
impl fmt::Debug for Session {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Session { .. }")
    }
}
pub struct Board {
    db: Connection,
    identity: [u8; 32],
    credential_directory: std::path::PathBuf,
}

fn private(path: &Path, directory: bool) -> Result<()> {
    let m = fs::symlink_metadata(path)?;
    if m.uid() != rustix::process::geteuid().as_raw()
        || m.mode() & 0o077 != 0
        || if directory { !m.is_dir() } else { !m.is_file() }
    {
        return Err(Error::UnsafeState);
    }
    Ok(())
}
fn handle(value: &str) -> Result<String> {
    if !(3..=24).contains(&value.len())
        || !value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
    {
        return Err(Error::Invalid(
            "handle needs 3-24 ASCII letters, digits, underscores or hyphens",
        ));
    }
    Ok(value.to_ascii_lowercase())
}
fn text(value: &str, max: usize, multiline: bool) -> Result<()> {
    if value.trim().is_empty()
        || value.len() > max
        || value
            .chars()
            .any(|c| c.is_control() && !(multiline && c == '\n'))
    {
        return Err(Error::Invalid(
            "text is empty, too long or contains control characters",
        ));
    }
    Ok(())
}
fn password(value: &str) -> Result<()> {
    if value.chars().count() < 12 || value.len() > 128 || value.chars().any(char::is_control) {
        return Err(Error::Invalid(
            "password needs at least 12 characters, at most 128 bytes, and no controls",
        ));
    }
    Ok(())
}
fn hasher() -> Argon2<'static> {
    Argon2::new(
        Algorithm::Argon2id,
        Version::V0x13,
        Params::new(19456, 2, 1, Some(32)).expect("constant Argon2 parameters"),
    )
}
fn hash(value: &str) -> Result<String> {
    password(value)?;
    hasher()
        .hash_password(value.as_bytes())
        .map(|h| h.to_string())
        .map_err(|_| Error::PasswordHash)
}
fn status(s: &str) -> Result<Status> {
    match s {
        "pending" => Ok(Status::Pending),
        "approved" => Ok(Status::Approved),
        "revoked" => Ok(Status::Revoked),
        _ => Err(Error::UnsupportedSchema),
    }
}
fn role(s: &str) -> Result<Role> {
    match s {
        "member" => Ok(Role::Member),
        "sysop" => Ok(Role::Sysop),
        _ => Err(Error::UnsupportedSchema),
    }
}
fn current(
    tx: &Transaction<'_>,
    identity: &[u8; 32],
    session: &Session,
    approved: bool,
    sysop: bool,
) -> Result<Member> {
    if session.board != *identity {
        return Err(Error::Denied);
    }
    let row: Option<(String, String, String, i64)> = tx
        .query_row(
            "SELECT handle,status,role,epoch FROM members WHERE id=?1",
            [session.member],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
        )
        .optional()?;
    let (handle, s, r, epoch) = row.ok_or(Error::Denied)?;
    if epoch != session.epoch {
        return Err(Error::StaleSession);
    }
    let member = Member {
        handle,
        status: status(&s)?,
        role: role(&r)?,
    };
    if (approved && member.status != Status::Approved) || (sysop && member.role != Role::Sysop) {
        return Err(Error::Denied);
    }
    Ok(member)
}
fn post_row(r: &rusqlite::Row<'_>) -> rusqlite::Result<Post> {
    Ok(Post {
        id: r.get(0)?,
        room_id: r.get(1)?,
        author: r.get(2)?,
        body: r.get(3)?,
        reply_to: r.get(4)?,
    })
}

impl Board {
    /// Parent directories must already exist and be trusted by the local operator.
    pub fn open(directory: &Path) -> Result<Self> {
        match fs::DirBuilder::new().mode(0o700).create(directory) {
            Ok(()) => (),
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => (),
            Err(e) => return Err(e.into()),
        }
        private(directory, true)?;
        let path = directory.join("faction.sqlite3");
        match fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(&path)
        {
            Ok(_) => (),
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => (),
            Err(e) => return Err(e.into()),
        }
        private(&path, false)?;
        for suffix in ["-wal", "-shm", "-journal"] {
            let side = directory.join(format!("faction.sqlite3{suffix}"));
            match fs::symlink_metadata(&side) {
                Ok(_) => private(&side, false)?,
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => (),
                Err(e) => return Err(e.into()),
            }
        }
        let mut db = Connection::open_with_flags(
            &path,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_WRITE | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )?;
        db.busy_timeout(Duration::from_millis(100))?;
        let app: i64 = db.query_row("PRAGMA application_id", [], |r| r.get(0))?;
        let version: i64 = db.query_row("PRAGMA user_version", [], |r| r.get(0))?;
        if app == 0 && version == 0 {
            let tables: i64 =
                db.query_row("SELECT count(*) FROM sqlite_schema", [], |r| r.get(0))?;
            if tables != 0 {
                return Err(Error::UnsupportedSchema);
            }
            let mut identity = [0; 32];
            getrandom::fill(&mut identity).map_err(|_| Error::Entropy)?;
            let tx = db.transaction_with_behavior(TransactionBehavior::Immediate)?;
            // Another opener may have initialized while this connection waited.
            let actual: i64 = tx.query_row("PRAGMA user_version", [], |r| r.get(0))?;
            if actual == 0 {
                tx.execute_batch(include_str!("schema.sql"))?;
                tx.execute("INSERT INTO board VALUES(1,?1)", [identity.as_slice()])?;
                tx.pragma_update(None, "application_id", APP_ID)?;
                tx.pragma_update(None, "user_version", 1)?;
            }
            tx.commit()?;
        } else if app != APP_ID || !(1..=15).contains(&version) {
            return Err(Error::UnsupportedSchema);
        }
        let actual_app: i64 = db.query_row("PRAGMA application_id", [], |r| r.get(0))?;
        let actual_version: i64 = db.query_row("PRAGMA user_version", [], |r| r.get(0))?;
        if actual_app != APP_ID || !(1..=15).contains(&actual_version) {
            return Err(Error::UnsupportedSchema);
        }
        db.pragma_update(None, "foreign_keys", true)?;
        if actual_version == 1 {
            let tx = db.transaction_with_behavior(TransactionBehavior::Immediate)?;
            let v: i64 = tx.query_row("PRAGMA user_version", [], |r| r.get(0))?;
            if v == 1 {
                tx.execute_batch(include_str!("chat.sql"))?;
                tx.pragma_update(None, "user_version", 2)?;
            } else if !(2..=15).contains(&v) {
                return Err(Error::UnsupportedSchema);
            }
            tx.commit()?;
        }
        if actual_version < 3 {
            let tx = db.transaction_with_behavior(TransactionBehavior::Immediate)?;
            let v: i64 = tx.query_row("PRAGMA user_version", [], |r| r.get(0))?;
            if v == 2 {
                tx.execute_batch(include_str!("scores.sql"))?;
                tx.pragma_update(None, "user_version", 3)?;
            } else if !(3..=15).contains(&v) {
                return Err(Error::UnsupportedSchema);
            }
            tx.commit()?;
        }
        let tx = db.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let v: i64 = tx.query_row("PRAGMA user_version", [], |r| r.get(0))?;
        if v == 3 {
            tx.execute_batch(include_str!("files.sql"))?;
            tx.pragma_update(None, "user_version", 4)?;
        } else if !(4..=15).contains(&v) {
            return Err(Error::UnsupportedSchema);
        }
        tx.commit()?;
        let tx = db.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let v: i64 = tx.query_row("PRAGMA user_version", [], |r| r.get(0))?;
        if v == 4 {
            tx.execute_batch(include_str!("file_links.sql"))?;
            tx.execute("INSERT INTO file_namespace VALUES(1,?1)", [request_id()?])?;
            tx.pragma_update(None, "user_version", 5)?;
        } else if !(5..=15).contains(&v) {
            return Err(Error::UnsupportedSchema);
        }
        tx.commit()?;
        let tx = db.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let v: i64 = tx.query_row("PRAGMA user_version", [], |r| r.get(0))?;
        if v == 5 {
            tx.execute_batch(include_str!("post_metadata.sql"))?;
            tx.pragma_update(None, "user_version", 6)?;
        } else if !(6..=15).contains(&v) {
            return Err(Error::UnsupportedSchema);
        }
        tx.commit()?;
        let tx = db.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let v: i64 = tx.query_row("PRAGMA user_version", [], |r| r.get(0))?;
        if v == 6 {
            let expected = include_str!("post_metadata.sql")
                .split("INSERT INTO")
                .next()
                .ok_or(Error::UnsupportedSchema)?
                .trim()
                .trim_end_matches(';');
            let actual: String = tx.query_row(
                "SELECT sql FROM sqlite_schema WHERE type='table' AND name='post_metadata'",
                [],
                |r| r.get(0),
            )?;
            let entries: i64 = tx.query_row(
                "SELECT count(*) FROM sqlite_schema WHERE tbl_name='post_metadata'",
                [],
                |r| r.get(0),
            )?;
            if actual != expected || entries != 2 {
                return Err(Error::UnsupportedSchema);
            }
            // Validate the old contract before widening it. Never bless UTF-8
            // smuggled into schema 6, or replace existing public article IDs.
            post_metadata::validate(
                &tx,
                std::time::Instant::now() + std::time::Duration::from_secs(5),
                6,
            )?;
            tx.execute_batch(include_str!("post_metadata_v7.sql"))?;
            tx.pragma_update(None, "user_version", 7)?;
        } else if !(7..=15).contains(&v) {
            return Err(Error::UnsupportedSchema);
        }
        tx.commit()?;
        let tx = db.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let v: i64 = tx.query_row("PRAGMA user_version", [], |r| r.get(0))?;
        if v == 7 {
            tx.execute_batch(include_str!("room_drafts.sql"))?;
            tx.pragma_update(None, "user_version", 8)?;
        } else if !(8..=15).contains(&v) {
            return Err(Error::UnsupportedSchema);
        }
        tx.commit()?;
        let tx = db.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let v: i64 = tx.query_row("PRAGMA user_version", [], |r| r.get(0))?;
        if v == 8 {
            tx.execute_batch(include_str!("wall.sql"))?;
            tx.pragma_update(None, "user_version", 9)?;
        } else if !(9..=15).contains(&v) {
            return Err(Error::UnsupportedSchema);
        }
        tx.commit()?;
        let tx = db.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let v: i64 = tx.query_row("PRAGMA user_version", [], |r| r.get(0))?;
        if v == 9 {
            tx.execute_batch(include_str!("member_pages.sql"))?;
            tx.pragma_update(None, "user_version", 10)?;
        } else if !(10..=15).contains(&v) {
            return Err(Error::UnsupportedSchema);
        }
        tx.commit()?;
        let tx = db.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let v: i64 = tx.query_row("PRAGMA user_version", [], |r| r.get(0))?;
        if v == 10 {
            tx.execute_batch(include_str!("gatherings.sql"))?;
            tx.pragma_update(None, "user_version", 11)?;
        } else if !(11..=15).contains(&v) {
            return Err(Error::UnsupportedSchema);
        }
        tx.commit()?;
        let tx = db.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let v: i64 = tx.query_row("PRAGMA user_version", [], |r| r.get(0))?;
        if v == 11 {
            tx.execute_batch(include_str!("polls.sql"))?;
            tx.pragma_update(None, "user_version", 12)?;
        } else if !(12..=15).contains(&v) {
            return Err(Error::UnsupportedSchema);
        }
        tx.commit()?;
        let tx = db.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let v: i64 = tx.query_row("PRAGMA user_version", [], |r| r.get(0))?;
        if v == 12 {
            tx.execute_batch(include_str!("catchup.sql"))?;
            tx.pragma_update(None, "user_version", 13)?;
        } else if !(13..=15).contains(&v) {
            return Err(Error::UnsupportedSchema);
        }
        tx.commit()?;
        let tx = db.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let v: i64 = tx.query_row("PRAGMA user_version", [], |r| r.get(0))?;
        if v == 13 {
            tx.execute_batch(include_str!("discussion.sql"))?;
            tx.pragma_update(None, "user_version", 14)?;
        } else if !(14..=15).contains(&v) {
            return Err(Error::UnsupportedSchema);
        }
        tx.commit()?;
        let tx = db.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let v: i64 = tx.query_row("PRAGMA user_version", [], |r| r.get(0))?;
        if v == 14 {
            tx.execute_batch(include_str!("requests.sql"))?;
            tx.pragma_update(None, "user_version", 15)?;
        } else if v != 15 {
            return Err(Error::UnsupportedSchema);
        }
        tx.commit()?;
        db.execute_batch("PRAGMA foreign_keys=ON; PRAGMA journal_mode=WAL; PRAGMA synchronous=FULL; PRAGMA cache_size=-2048; PRAGMA mmap_size=0;")?;
        let identity: Vec<u8> =
            db.query_row("SELECT identity FROM board WHERE singleton=1", [], |r| {
                r.get(0)
            })?;
        Ok(Self {
            db,
            identity: identity.try_into().map_err(|_| Error::UnsupportedSchema)?,
            credential_directory: fs::canonicalize(directory)?,
        })
    }
    pub fn bootstrap(&mut self, name: &str, secret: &str) -> Result<()> {
        let name = handle(name)?;
        let hashed = {
            let _slot = credentials::acquire(&self.credential_directory)?;
            hash(secret)?
        };
        let tx = self
            .db
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let count: i64 = tx.query_row("SELECT count(*) FROM members", [], |r| r.get(0))?;
        if count != 0 {
            return Err(Error::AlreadyInitialized);
        }
        tx.execute("INSERT INTO members(handle,password_hash,role,status) VALUES(?1,?2,'sysop','approved')",params![name,hashed])?;
        tx.execute("INSERT INTO rooms(slug,title) VALUES('lobby','Lobby')", [])?;
        tx.commit()?;
        Ok(())
    }
    pub fn register(&mut self, name: &str, secret: &str) -> Result<()> {
        let name = handle(name)?;
        let hashed = {
            let _slot = credentials::acquire(&self.credential_directory)?;
            hash(secret)?
        };
        let tx = self
            .db
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let ready: i64 =
            tx.query_row("SELECT count(*) FROM members WHERE role='sysop'", [], |r| {
                r.get(0)
            })?;
        if ready == 0 {
            return Err(Error::NotInitialized);
        }
        if tx
            .query_row("SELECT id FROM members WHERE handle=?1", [&name], |r| {
                r.get::<_, i64>(0)
            })
            .optional()?
            .is_some()
        {
            return Err(Error::Invalid("handle unavailable"));
        }
        tx.execute("INSERT INTO members(handle,password_hash,role,status) VALUES(?1,?2,'member','pending')",params![name,hashed])?;
        tx.commit()?;
        Ok(())
    }
    pub fn authenticate(&mut self, name: &str, secret: &str) -> Result<Session> {
        let name = handle(name).map_err(|_| Error::Credentials)?;
        if secret.len() > 128 || secret.is_empty() {
            return Err(Error::Credentials);
        }
        let row: Option<(i64, String, i64)> = self
            .db
            .query_row(
                "SELECT id,password_hash,epoch FROM members WHERE handle=?1",
                [name],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .optional()?;
        let Some((member, stored, epoch)) = row else {
            return Err(Error::Credentials);
        };
        credentials::validate_record(&stored)?;
        {
            let _slot = credentials::acquire(&self.credential_directory)?;
            hasher()
                .verify_password(secret.as_bytes(), stored.as_str())
                .map_err(|_| Error::Credentials)?;
        }
        Ok(Session {
            board: self.identity,
            member,
            epoch,
        })
    }
    pub fn whoami(&mut self, session: &Session) -> Result<Member> {
        let tx = self.db.transaction()?;
        current(&tx, &self.identity, session, false, false)
    }
    pub fn members(&mut self, session: &Session) -> Result<Vec<Member>> {
        let tx = self.db.transaction()?;
        current(&tx, &self.identity, session, true, true)?;
        let mut stmt =
            tx.prepare("SELECT handle,status,role FROM members ORDER BY id LIMIT 100")?;
        let rows = stmt.query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
            ))
        })?;
        rows.map(|row| {
            let (handle, s, r) = row?;
            Ok(Member {
                handle,
                status: status(&s)?,
                role: role(&r)?,
            })
        })
        .collect()
    }
    pub fn set_status(&mut self, session: &Session, name: &str, value: Status) -> Result<()> {
        let name = handle(name)?;
        let tx = self
            .db
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        current(&tx, &self.identity, session, true, true)?;
        let target: Option<(i64, String)> = tx
            .query_row("SELECT id,role FROM members WHERE handle=?1", [name], |r| {
                Ok((r.get(0)?, r.get(1)?))
            })
            .optional()?;
        let (id, r) = target.ok_or(Error::NotFound)?;
        if r == "sysop" {
            return Err(Error::Invalid(
                "sysop status changes require a later multi-operator policy",
            ));
        }
        let value = match value {
            Status::Pending => "pending",
            Status::Approved => "approved",
            Status::Revoked => "revoked",
        };
        tx.execute(
            "UPDATE members SET status=?1,epoch=epoch+1 WHERE id=?2",
            params![value, id],
        )?;
        tx.execute("DELETE FROM room_drafts WHERE member_id=?1", [id])?;
        tx.commit()?;
        Ok(())
    }
    /// Local filesystem-owner recovery only. NEVER expose this through member transports.
    /// Possession of the private database already gives the operator equivalent authority.
    pub fn recover_local_sysop(&mut self, name: &str, new_secret: &str) -> Result<()> {
        let name = handle(name)?;
        let hashed = {
            let _slot = credentials::acquire(&self.credential_directory)?;
            hash(new_secret)?
        };
        let tx = self
            .db
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let changed=tx.execute("UPDATE members SET password_hash=?1,epoch=epoch+1,status='approved' WHERE handle=?2 AND role='sysop'",params![hashed,name])?;
        if changed != 1 {
            return Err(Error::NotFound);
        }
        tx.execute(
            "DELETE FROM room_drafts WHERE member_id IN (SELECT id FROM members WHERE handle=?1)",
            [name],
        )?;
        tx.commit()?;
        Ok(())
    }
    pub fn create_room(&mut self, session: &Session, slug: &str, title: &str) -> Result<i64> {
        if slug.is_empty()
            || slug.len() > 32
            || !slug
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
        {
            return Err(Error::Invalid(
                "room slug needs 1-32 lowercase letters, digits or hyphens",
            ));
        }
        text(title, 64, false)?;
        let tx = self
            .db
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        current(&tx, &self.identity, session, true, true)?;
        tx.execute(
            "INSERT INTO rooms(slug,title) VALUES(?1,?2)",
            params![slug, title],
        )?;
        let id = tx.last_insert_rowid();
        tx.commit()?;
        Ok(id)
    }
    pub fn rooms(&mut self, session: &Session) -> Result<Vec<Room>> {
        let tx = self.db.transaction()?;
        current(&tx, &self.identity, session, true, false)?;
        let mut stmt=tx.prepare("SELECT r.id,r.slug,r.title,(SELECT count(*) FROM posts p WHERE p.room_id=r.id AND p.id>coalesce(u.last_read,0)) FROM rooms r LEFT JOIN unread u ON u.room_id=r.id AND u.member_id=?1 ORDER BY r.id LIMIT 100")?;
        Ok(stmt
            .query_map([session.member], |r| {
                Ok(Room {
                    id: r.get(0)?,
                    slug: r.get(1)?,
                    title: r.get(2)?,
                    unread: r.get(3)?,
                })
            })?
            .collect::<rusqlite::Result<_>>()?)
    }
    pub fn posts(&mut self, session: &Session, room: i64, after: i64) -> Result<Vec<Post>> {
        if after < 0 {
            return Err(Error::Invalid("negative post cursor"));
        }
        let tx = self.db.transaction()?;
        current(&tx, &self.identity, session, true, false)?;
        if tx
            .query_row("SELECT id FROM rooms WHERE id=?1", [room], |r| {
                r.get::<_, i64>(0)
            })
            .optional()?
            .is_none()
        {
            return Err(Error::NotFound);
        }
        let mut stmt=tx.prepare("SELECT p.id,p.room_id,m.handle,p.body,p.reply_to FROM posts p JOIN members m ON m.id=p.author_id WHERE p.room_id=?1 AND p.id>?2 ORDER BY p.id LIMIT 20")?;
        let mut out = Vec::new();
        let mut bytes = 0;
        for row in stmt.query_map(params![room, after], post_row)? {
            let mut post = row?;
            if post.body.contains("faction-file:") {
                post.body = file_links::render_links(&tx, session.member, &post.body)?;
            }
            let charge = post.body.len() + post.author.len() + 64;
            if bytes + charge > PAGE_BYTES {
                break;
            }
            bytes += charge;
            out.push(post);
        }
        Ok(out)
    }
    pub fn post(
        &mut self,
        session: &Session,
        room: i64,
        request: &str,
        body: &str,
        reply: Option<i64>,
    ) -> Result<Post> {
        self.post_article(
            session,
            room,
            request,
            PostDraft {
                subject: "",
                body,
                reply_to: reply,
            },
        )
    }
    /// Empty subject selects a generated label or the parent's reply subject.
    pub fn post_article(
        &mut self,
        session: &Session,
        room: i64,
        request: &str,
        draft: PostDraft<'_>,
    ) -> Result<Post> {
        let PostDraft {
            subject,
            body,
            reply_to: reply,
        } = draft;
        post_metadata::subject(subject)?;
        text(body, POST_BYTES, true)?;
        if !(16..=64).contains(&request.len())
            || !request
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
        {
            return Err(Error::Invalid(
                "request ID needs 16-64 ASCII letters, digits, underscores or hyphens",
            ));
        }
        let tx = self
            .db
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let member = current(&tx, &self.identity, session, true, false)?;
        let existing:Option<Post>=tx.query_row("SELECT p.id,p.room_id,m.handle,p.body,p.reply_to FROM posts p JOIN members m ON m.id=p.author_id WHERE p.author_id=?1 AND p.request_id=?2",params![session.member,request],post_row).optional()?;
        if let Some(p) = existing {
            if p.room_id != room || p.body != body || p.reply_to != reply {
                return Err(Error::RequestConflict);
            }
            let wanted = post_metadata::resolve_subject(&tx, room, reply, subject, p.id)?;
            let stored: String = tx.query_row(
                "SELECT subject FROM post_metadata WHERE post_id=?1",
                [p.id],
                |r| r.get(0),
            )?;
            if stored != wanted {
                return Err(Error::RequestConflict);
            }
            tx.execute(
                "DELETE FROM room_drafts WHERE member_id=?1 AND room_id=?2 AND request_id=?3",
                params![session.member, room, request],
            )?;
            tx.commit()?;
            return Ok(p);
        }
        if tx
            .query_row("SELECT id FROM rooms WHERE id=?1", [room], |r| {
                r.get::<_, i64>(0)
            })
            .optional()?
            .is_none()
        {
            return Err(Error::NotFound);
        }
        if let Some(parent) = reply
            && tx
                .query_row(
                    "SELECT id FROM posts WHERE id=?1 AND room_id=?2",
                    params![parent, room],
                    |r| r.get::<_, i64>(0),
                )
                .optional()?
                .is_none()
        {
            return Err(Error::Invalid("reply must reference a post in this room"));
        }
        file_links::validate_links(&tx, session.member, body)?;
        tx.execute(
            "INSERT INTO posts(room_id,author_id,request_id,body,reply_to) VALUES(?1,?2,?3,?4,?5)",
            params![room, session.member, request, body, reply],
        )?;
        let post = Post {
            id: tx.last_insert_rowid(),
            room_id: room,
            author: member.handle,
            body: body.to_owned(),
            reply_to: reply,
        };
        let subject = post_metadata::resolve_subject(&tx, room, reply, subject, post.id)?;
        tx.execute("INSERT INTO post_metadata(post_id,article_id,subject,created_at,origin) VALUES(?1,lower(hex(randomblob(16))),?2,unixepoch(),'local')",params![post.id,subject])?;
        tx.execute(
            "DELETE FROM room_drafts WHERE member_id=?1 AND room_id=?2 AND request_id=?3",
            params![session.member, room, request],
        )?;
        tx.commit()?;
        Ok(post)
    }
    pub fn mark_read(&mut self, session: &Session, room: i64, through: i64) -> Result<()> {
        let tx = self
            .db
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        current(&tx, &self.identity, session, true, false)?;
        if tx
            .query_row(
                "SELECT id FROM posts WHERE id=?1 AND room_id=?2",
                params![through, room],
                |r| r.get::<_, i64>(0),
            )
            .optional()?
            .is_none()
        {
            return Err(Error::NotFound);
        }
        tx.execute("INSERT INTO unread VALUES(?1,?2,?3) ON CONFLICT(member_id,room_id) DO UPDATE SET last_read=max(last_read,excluded.last_read)",params![session.member,room,through])?;
        tx.commit()?;
        Ok(())
    }
    pub fn unread_cursor(&mut self, session: &Session, room: i64) -> Result<i64> {
        let tx = self.db.transaction()?;
        current(&tx, &self.identity, session, true, false)?;
        if tx
            .query_row("SELECT id FROM rooms WHERE id=?1", [room], |r| {
                r.get::<_, i64>(0)
            })
            .optional()?
            .is_none()
        {
            return Err(Error::NotFound);
        }
        Ok(tx
            .query_row(
                "SELECT last_read FROM unread WHERE member_id=?1 AND room_id=?2",
                params![session.member, room],
                |r| r.get(0),
            )
            .optional()?
            .unwrap_or(0))
    }
}
/// OS-random local request identifier; no timestamp, user identity or transport address.
pub fn request_id() -> Result<String> {
    let mut b = [0; 16];
    getrandom::fill(&mut b).map_err(|_| Error::Entropy)?;
    Ok(b.iter().map(|v| format!("{v:02x}")).collect())
}

//! Local ephemeral community notes, separate from rooms, chat and private mail.
use super::*;
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WallConfig {
    pub enabled: bool,
    pub name: String,
    pub max_notes: u16,
    pub note_bytes: u16,
    pub retention_days: u16,
    pub cooldown_seconds: u16,
    pub weekly: bool,
    pub archive_editions: u16,
}
impl WallConfig {
    pub fn validate(&self) -> Result<()> {
        text(&self.name, 24, false)?;
        if !self.name.bytes().all(|b| (32..=126).contains(&b))
            || !(1..=128).contains(&self.max_notes)
            || !(32..=240).contains(&self.note_bytes)
            || !(1..=365).contains(&self.retention_days)
            || !(1..=3600).contains(&self.cooldown_seconds)
            || self.archive_editions > 12
            || (!self.weekly && self.archive_editions != 0)
        {
            return Err(Error::Invalid("wall policy outside supported bounds"));
        }
        Ok(())
    }
}
#[derive(Debug, Clone)]
pub struct WallNote {
    pub id: i64,
    pub author: String,
    pub body: String,
    pub posted: String,
    pub edited: bool,
    pub mine: bool,
}
#[derive(Debug, Clone)]
pub struct WallPage {
    pub config: WallConfig,
    pub edition: i64,
    pub current: bool,
    pub count: usize,
    pub offset: usize,
    pub note: Option<WallNote>,
}
fn policy(db: &Connection) -> Result<(WallConfig, i64)> {
    Ok(db.query_row("SELECT enabled,name,max_notes,note_bytes,retention_days,cooldown_seconds,weekly,archive_editions,edition FROM wall_settings WHERE singleton=1",[],|r|Ok((WallConfig{enabled:r.get(0)?,name:r.get(1)?,max_notes:r.get(2)?,note_bytes:r.get(3)?,retention_days:r.get(4)?,cooldown_seconds:r.get(5)?,weekly:r.get(6)?,archive_editions:r.get(7)?},r.get(8)?)))?)
}
pub fn wall_text(body: &str, max: usize) -> Result<()> {
    if body.trim().is_empty()
        || body.len() > max
        || body.split('\n').count() > 4
        || body.split('\n').any(|l| l.len() > 72)
        || !body.bytes().all(|b| b == b'\n' || (32..=126).contains(&b))
    {
        return Err(Error::Invalid(
            "wall needs 1-4 printable ASCII lines, 72 columns, within its byte limit",
        ));
    }
    Ok(())
}
fn key(key: &str) -> Result<()> {
    if key.len() != 32
        || !key
            .bytes()
            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
    {
        return Err(Error::Invalid("invalid wall request ID"));
    }
    Ok(())
}
fn maintain(tx: &Transaction<'_>) -> Result<(WallConfig, i64)> {
    let (c, old) = policy(tx)?;
    let edition = if c.weekly {
        tx.query_row("SELECT max(?1,1+(unixepoch()+259200)/604800)", [old], |r| {
            r.get(0)
        })?
    } else {
        0
    };
    tx.execute(
        "UPDATE wall_settings SET edition=?1 WHERE singleton=1",
        [edition],
    )?;
    tx.execute(
        "DELETE FROM wall_notes WHERE posted_at<unixepoch()-?1",
        [i64::from(c.retention_days) * 86400],
    )?;
    tx.execute("DELETE FROM wall_notes WHERE edition!=?1 AND edition NOT IN (SELECT edition FROM wall_notes WHERE edition<?1 GROUP BY edition ORDER BY edition DESC LIMIT ?2)",params![edition,c.archive_editions])?;
    tx.execute("DELETE FROM wall_notes WHERE id IN (SELECT id FROM (SELECT id,row_number() OVER (PARTITION BY edition ORDER BY id DESC) AS n FROM wall_notes) WHERE n>?1)",[c.max_notes])?;
    Ok((c, edition))
}
fn enabled(tx: &Transaction<'_>) -> Result<(WallConfig, i64)> {
    let p = maintain(tx)?;
    if !p.0.enabled {
        return Err(Error::Denied);
    }
    Ok(p)
}
fn cooldown(tx: &Transaction<'_>, s: &Session, c: &WallConfig) -> Result<()> {
    if tx
        .prepare("SELECT 1 FROM wall_receipts WHERE member_id=?1 AND last_at>unixepoch()-?2")?
        .exists(params![s.member, c.cooldown_seconds])?
    {
        return Err(Error::Invalid(
            "wall cooldown; wait before posting or editing again",
        ));
    }
    Ok(())
}
impl Board {
    pub fn wall_config(&mut self, s: &Session) -> Result<WallConfig> {
        let tx = self.db.transaction()?;
        current(&tx, &self.identity, s, true, false)?;
        Ok(policy(&tx)?.0)
    }
    /// Changing weekly mode clears notes/archives; caller must make that explicit.
    pub fn configure_wall(&mut self, s: &Session, c: &WallConfig) -> Result<()> {
        c.validate()?;
        let tx = self
            .db
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        current(&tx, &self.identity, s, true, true)?;
        if policy(&tx)?.0.weekly != c.weekly {
            tx.execute("DELETE FROM wall_notes", [])?;
            tx.execute("UPDATE wall_settings SET edition=0", [])?;
        }
        tx.execute("UPDATE wall_settings SET enabled=?1,name=?2,max_notes=?3,note_bytes=?4,retention_days=?5,cooldown_seconds=?6,weekly=?7,archive_editions=?8",params![c.enabled,c.name,c.max_notes,c.note_bytes,c.retention_days,c.cooldown_seconds,c.weekly,c.archive_editions])?;
        // A smaller content limit removes notes that would no longer fit it.
        tx.execute(
            "DELETE FROM wall_notes WHERE length(CAST(body AS BLOB))>?1",
            [c.note_bytes],
        )?;
        maintain(&tx)?;
        tx.commit()?;
        Ok(())
    }
    pub fn wall_page(
        &mut self,
        s: &Session,
        edition: Option<i64>,
        offset: usize,
    ) -> Result<WallPage> {
        let tx = self
            .db
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        current(&tx, &self.identity, s, true, false)?;
        let (c, now) = enabled(&tx)?;
        let ed = edition.unwrap_or(now);
        let count: i64 = tx.query_row(
            "SELECT count(*) FROM wall_notes WHERE edition=?1",
            [ed],
            |r| r.get(0),
        )?;
        if ed != now && count == 0 {
            return Err(Error::NotFound);
        }
        let count = count as usize;
        let offset = offset.min(count.saturating_sub(1));
        let note=tx.query_row("SELECT n.id,m.handle,n.body,strftime('%Y-%m-%d %H:%M UTC',n.posted_at,'unixepoch'),n.edited_at!=0,n.member_id=?2 FROM wall_notes n JOIN members m ON m.id=n.member_id WHERE edition=?1 ORDER BY n.id LIMIT 1 OFFSET ?3",params![ed,s.member,offset as i64],|r|Ok(WallNote{id:r.get(0)?,author:r.get(1)?,body:r.get(2)?,posted:r.get(3)?,edited:r.get(4)?,mine:r.get(5)?})).optional()?;
        tx.commit()?;
        Ok(WallPage {
            config: c,
            edition: ed,
            current: ed == now,
            count,
            offset,
            note,
        })
    }
    pub fn wall_archives(&mut self, s: &Session) -> Result<Vec<(i64, String, i64)>> {
        let tx = self
            .db
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        current(&tx, &self.identity, s, true, false)?;
        let (_, now) = enabled(&tx)?;
        let rows=tx.prepare("SELECT edition,strftime('%Y-%m-%d',(edition-1)*604800-259200,'unixepoch'),count(*) FROM wall_notes WHERE edition<?1 GROUP BY edition ORDER BY edition DESC LIMIT 12")?.query_map([now],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?)))?.collect::<std::result::Result<Vec<_>,_>>()?;
        tx.commit()?;
        Ok(rows)
    }
    pub fn wall_post(
        &mut self,
        s: &Session,
        edition: i64,
        request: &str,
        body: &str,
    ) -> Result<i64> {
        key(request)?;
        let tx = self
            .db
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        current(&tx, &self.identity, s, true, false)?;
        let (c, now) = enabled(&tx)?;
        wall_text(body, c.note_bytes as usize)?;
        if let Some((id, old)) = tx
            .query_row(
                "SELECT id,body FROM wall_notes WHERE member_id=?1 AND request_id=?2",
                params![s.member, request],
                |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?)),
            )
            .optional()?
        {
            if old != body {
                return Err(Error::RequestConflict);
            }
            tx.commit()?;
            return Ok(id);
        }
        if edition != now {
            return Err(Error::Invalid("wall edition changed; reopen the wall"));
        }
        if tx
            .prepare("SELECT 1 FROM wall_receipts WHERE member_id=?1 AND request_id=?2")?
            .exists(params![s.member, request])?
        {
            return Err(Error::Invalid(
                "wall note removed; this request cannot recreate it",
            ));
        }
        cooldown(&tx, s, &c)?;
        tx.execute("INSERT INTO wall_notes(edition,member_id,request_id,body,posted_at,edited_at) VALUES(?1,?2,?3,?4,unixepoch(),0)",params![now,s.member,request,body])?;
        let id = tx.last_insert_rowid();
        tx.execute("INSERT INTO wall_receipts VALUES(?1,?2,?3,unixepoch()) ON CONFLICT(member_id) DO UPDATE SET request_id=excluded.request_id,note_id=excluded.note_id,last_at=excluded.last_at",params![s.member,request,id])?;
        maintain(&tx)?;
        tx.commit()?;
        Ok(id)
    }
    pub fn wall_edit(&mut self, s: &Session, id: i64, expected: &str, body: &str) -> Result<()> {
        let tx = self
            .db
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        current(&tx, &self.identity, s, true, false)?;
        let (c, now) = enabled(&tx)?;
        wall_text(body, c.note_bytes as usize)?;
        let (old, ed, member): (String, i64, i64) = tx
            .query_row(
                "SELECT body,edition,member_id FROM wall_notes WHERE id=?1",
                [id],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .optional()?
            .ok_or(Error::NotFound)?;
        if member != s.member || ed != now {
            return Err(Error::Denied);
        }
        if old != expected {
            return Err(Error::RequestConflict);
        }
        if old != body {
            cooldown(&tx, s, &c)?;
            tx.execute(
                "UPDATE wall_notes SET body=?1,edited_at=unixepoch() WHERE id=?2",
                params![body, id],
            )?;
            tx.execute(
                "UPDATE wall_receipts SET last_at=unixepoch() WHERE member_id=?1",
                [s.member],
            )?;
        }
        tx.commit()?;
        Ok(())
    }
    pub fn wall_delete(&mut self, s: &Session, id: i64) -> Result<()> {
        let tx = self
            .db
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let m = current(&tx, &self.identity, s, true, false)?;
        enabled(&tx)?;
        let n = tx.execute(
            "DELETE FROM wall_notes WHERE id=?1 AND (member_id=?2 OR ?3)",
            params![id, s.member, m.role == Role::Sysop],
        )?;
        if n != 1 {
            return Err(Error::Denied);
        }
        tx.commit()?;
        Ok(())
    }
}
pub(crate) fn validate(db: &Connection, deadline: std::time::Instant) -> Result<()> {
    let (c, ed) = policy(db)?;
    c.validate()?;
    if (!c.weekly && ed != 0)
        || db.query_row("SELECT count(*) FROM wall_settings", [], |r| {
            r.get::<_, i64>(0)
        })? != 1
    {
        return Err(Error::Invalid("snapshot wall policy"));
    }
    let mut q = db.prepare("SELECT request_id,body FROM wall_notes")?;
    let mut rows = q.query([])?;
    let mut count = 0;
    while let Some(r) = rows.next()? {
        if std::time::Instant::now() >= deadline {
            return Err(Error::Invalid("snapshot wall deadline"));
        }
        key(&r.get::<_, String>(0)?)?;
        wall_text(&r.get::<_, String>(1)?, c.note_bytes as usize)?;
        count += 1;
    }
    if count > 128 * 13
        || db
            .prepare("SELECT 1 FROM wall_notes GROUP BY edition HAVING count(*)>?1")?
            .exists([c.max_notes])?
        || db.query_row("SELECT count(DISTINCT edition) FROM wall_notes", [], |r| {
            r.get::<_, i64>(0)
        })? > i64::from(c.archive_editions) + 1
    {
        return Err(Error::Invalid("snapshot wall quota"));
    }
    let mut q = db.prepare("SELECT request_id FROM wall_receipts")?;
    let mut rows = q.query([])?;
    while let Some(r) = rows.next()? {
        if std::time::Instant::now() >= deadline {
            return Err(Error::Invalid("snapshot wall deadline"));
        }
        key(&r.get::<_, String>(0)?)?;
    }
    Ok(())
}

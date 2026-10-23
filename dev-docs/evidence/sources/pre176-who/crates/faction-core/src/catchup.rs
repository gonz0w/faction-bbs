//! Explicit personal catch-up checkpoints; existing unread remains the only post ledger.
use super::*;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CatchupModule {
    Notice,
    Replies,
    Rooms,
    Files,
    Gatherings,
}
impl CatchupModule {
    pub fn key(self) -> &'static str {
        match self {
            Self::Notice => "notice",
            Self::Replies => "replies",
            Self::Rooms => "rooms",
            Self::Files => "files",
            Self::Gatherings => "gatherings",
        }
    }
    pub fn parse(s: &str) -> Result<Self> {
        match s {
            "notice" => Ok(Self::Notice),
            "replies" => Ok(Self::Replies),
            "rooms" => Ok(Self::Rooms),
            "files" => Ok(Self::Files),
            "gatherings" => Ok(Self::Gatherings),
            _ => Err(Error::Invalid("unknown catch-up section")),
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatchupConfig {
    pub enabled: bool,
    pub name: String,
    pub compact: bool,
    pub modules: Vec<CatchupModule>,
    pub notice: String,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatchupWindow {
    pub after: i64,
    pub through: i64,
    pub revision: String,
    pub last_saved: Option<String>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatchupPrefs {
    pub favorites_only: bool,
    pub favorites: Vec<i64>,
}
#[derive(Debug, Clone)]
pub struct CatchupItem {
    pub id: i64,
    pub key: String,
    pub title: String,
    pub summary: String,
}
#[derive(Debug, Clone)]
pub struct CatchupPage {
    pub items: Vec<CatchupItem>,
    pub next: Option<i64>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CatchupTarget {
    Room(i64),
    File(String),
    Gathering(i64),
}
fn ascii(s: &str, bytes: usize, lines: usize, empty: bool) -> Result<()> {
    if s.len() > bytes
        || (!empty && s.trim().is_empty())
        || s.split('\n').count() > lines
        || s.split('\n').any(|l| l.len() > 72)
        || !s.bytes().all(|b| b == b'\n' || (32..=126).contains(&b))
    {
        return Err(Error::Invalid(
            "catch-up text exceeds printable ASCII/line bounds",
        ));
    }
    Ok(())
}
fn modules(s: &str) -> Result<Vec<CatchupModule>> {
    if s.is_empty() {
        return Ok(vec![]);
    }
    let mut out = vec![];
    for key in s.split(',') {
        let m = CatchupModule::parse(key)?;
        if out.contains(&m) {
            return Err(Error::Invalid("duplicate catch-up section"));
        }
        out.push(m);
    }
    Ok(out)
}
impl CatchupConfig {
    pub fn validate(&self) -> Result<()> {
        ascii(&self.name, 24, 1, false)?;
        ascii(&self.notice, 240, 4, true)?;
        let raw = self
            .modules
            .iter()
            .map(|m| m.key())
            .collect::<Vec<_>>()
            .join(",");
        modules(&raw)?;
        Ok(())
    }
}
fn policy(db: &Connection) -> Result<CatchupConfig> {
    let (enabled, name, compact, raw, notice): (bool, String, bool, String, String) = db
        .query_row(
            "SELECT enabled,name,compact,modules,notice FROM catchup_settings WHERE singleton=1",
            [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)),
        )?;
    Ok(CatchupConfig {
        enabled,
        name,
        compact,
        modules: modules(&raw)?,
        notice,
    })
}
fn enabled(db: &Connection, module: Option<CatchupModule>) -> Result<CatchupConfig> {
    let c = policy(db)?;
    if !c.enabled || module.is_some_and(|m| !c.modules.contains(&m)) {
        return Err(Error::Invalid("catch-up section is disabled"));
    }
    Ok(c)
}
fn high_water(db: &Connection) -> Result<i64> {
    Ok(db.query_row(
        "SELECT coalesce((SELECT seq FROM sqlite_sequence WHERE name='catchup_publications'),0)",
        [],
        |r| r.get(0),
    )?)
}
fn window(db: &Connection, member: i64) -> Result<CatchupWindow> {
    let row=db.query_row("SELECT file_cursor,revision,CASE WHEN saved_at=0 THEN NULL ELSE strftime('%Y-%m-%d %H:%M',saved_at,'unixepoch') END FROM catchup_members WHERE member=?1",[member],|r|Ok((r.get::<_,i64>(0)?,r.get::<_,String>(1)?,r.get::<_,Option<String>>(2)?))).optional()?;
    let (after, revision, last_saved) = row.unwrap_or((0, String::new(), None));
    Ok(CatchupWindow {
        after,
        through: high_water(db)?,
        revision,
        last_saved,
    })
}
fn check_window(db: &Connection, w: &CatchupWindow) -> Result<()> {
    if w.after < 0 || w.through < w.after || w.through > high_water(db)? {
        return Err(Error::Invalid("catch-up file window outside bounds"));
    }
    Ok(())
}
fn preference(db: &Connection, member: i64) -> Result<CatchupPrefs> {
    let only = db
        .query_row(
            "SELECT favorites_only FROM catchup_members WHERE member=?1",
            [member],
            |r| r.get::<_, bool>(0),
        )
        .optional()?
        .unwrap_or(false);
    let favorites = db
        .prepare("SELECT room FROM catchup_favorites WHERE member=?1 ORDER BY room")?
        .query_map([member], |r| r.get(0))?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    Ok(CatchupPrefs {
        favorites_only: only,
        favorites,
    })
}
fn revision(db: &Connection, member: i64, expected: &str) -> Result<()> {
    if window(db, member)?.revision != expected {
        return Err(Error::RequestConflict);
    }
    Ok(())
}
impl Board {
    pub fn catchup_config(&mut self, s: &Session) -> Result<CatchupConfig> {
        let tx = self.db.transaction()?;
        let m = current(&tx, &self.identity, s, true, false)?;
        let mut c = policy(&tx)?;
        if m.role != Role::Sysop && (!c.enabled || !c.modules.contains(&CatchupModule::Notice)) {
            c.notice.clear();
        }
        Ok(c)
    }
    pub fn configure_catchup(&mut self, s: &Session, c: &CatchupConfig) -> Result<()> {
        c.validate()?;
        let tx = self
            .db
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        current(&tx, &self.identity, s, true, true)?;
        tx.execute(
            "UPDATE catchup_settings SET enabled=?1,name=?2,compact=?3,modules=?4,notice=?5",
            params![
                c.enabled,
                c.name,
                c.compact,
                c.modules
                    .iter()
                    .map(|m| m.key())
                    .collect::<Vec<_>>()
                    .join(","),
                c.notice
            ],
        )?;
        tx.commit()?;
        Ok(())
    }
    pub fn catchup_window(&mut self, s: &Session) -> Result<CatchupWindow> {
        let tx = self.db.transaction()?;
        current(&tx, &self.identity, s, true, false)?;
        enabled(&tx, None)?;
        window(&tx, s.member)
    }
    pub fn catchup_preferences(&mut self, s: &Session) -> Result<CatchupPrefs> {
        let tx = self.db.transaction()?;
        current(&tx, &self.identity, s, true, false)?;
        enabled(&tx, None)?;
        preference(&tx, s.member)
    }
    /// An explicit saved visit is not a post read acknowledgment or download receipt.
    pub fn catchup_checkpoint(&mut self, s: &Session, w: &CatchupWindow) -> Result<()> {
        let tx = self
            .db
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        current(&tx, &self.identity, s, true, false)?;
        enabled(&tx, None)?;
        check_window(&tx, w)?;
        revision(&tx, s.member, &w.revision)?;
        tx.execute("INSERT INTO catchup_members VALUES(?1,?2,unixepoch(),?3,0) ON CONFLICT(member) DO UPDATE SET file_cursor=max(file_cursor,excluded.file_cursor),saved_at=excluded.saved_at,revision=excluded.revision",params![s.member,w.through,request_id()?])?;
        tx.commit()?;
        Ok(())
    }
    /// '+ID', '-ID', all, fav, none. No account-wide last-seen telemetry is created.
    pub fn catchup_subscribe(&mut self, s: &Session, expected: &str, command: &str) -> Result<()> {
        let tx = self
            .db
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        current(&tx, &self.identity, s, true, false)?;
        enabled(&tx, None)?;
        revision(&tx, s.member, expected)?;
        let mut p = preference(&tx, s.member)?;
        if let Some(n) = command
            .strip_prefix('+')
            .or_else(|| command.strip_prefix('-'))
        {
            let room = n
                .parse::<i64>()
                .ok()
                .filter(|n| *n > 0)
                .ok_or(Error::Invalid("use +room-ID or -room-ID"))?;
            if !tx
                .prepare("SELECT 1 FROM rooms WHERE id=?1")?
                .exists([room])?
            {
                return Err(Error::NotFound);
            }
            if command.starts_with('+') {
                if !p.favorites.contains(&room) {
                    if p.favorites.len() >= 16 {
                        return Err(Error::Invalid("sixteen favorite rooms per member"));
                    }
                    p.favorites.push(room);
                }
                p.favorites_only = true;
            } else {
                p.favorites.retain(|id| *id != room);
            }
        } else {
            match command {
                "all" => p.favorites_only = false,
                "fav" => p.favorites_only = true,
                "none" => {
                    p.favorites_only = true;
                    p.favorites.clear();
                }
                _ => return Err(Error::Invalid("use +ID, -ID, all, fav or none")),
            }
        }
        tx.execute("INSERT INTO catchup_members VALUES(?1,0,0,?2,?3) ON CONFLICT(member) DO UPDATE SET revision=excluded.revision,favorites_only=excluded.favorites_only",params![s.member,request_id()?,p.favorites_only])?;
        tx.execute("DELETE FROM catchup_favorites WHERE member=?1", [s.member])?;
        for id in p.favorites {
            tx.execute(
                "INSERT INTO catchup_favorites VALUES(?1,?2)",
                params![s.member, id],
            )?;
        }
        tx.commit()?;
        Ok(())
    }
    pub fn catchup_page(
        &mut self,
        s: &Session,
        module: CatchupModule,
        after: i64,
        w: &CatchupWindow,
    ) -> Result<CatchupPage> {
        if after < 0 {
            return Err(Error::Invalid("negative catch-up cursor"));
        }
        let tx = self.db.transaction()?;
        let m = current(&tx, &self.identity, s, true, false)?;
        enabled(&tx, Some(module))?;
        let mut items = vec![];
        match module {
            CatchupModule::Notice => {}
            CatchupModule::Rooms | CatchupModule::Replies => {
                let prefs = preference(&tx, s.member)?;
                let rows=tx.prepare("SELECT r.id,r.title,coalesce(u.last_read,0) FROM rooms r LEFT JOIN unread u ON u.room_id=r.id AND u.member_id=?1 WHERE r.id>?2 AND (?3=0 OR EXISTS(SELECT 1 FROM catchup_favorites f WHERE f.member=?1 AND f.room=r.id)) ORDER BY r.id LIMIT 5")?.query_map(params![s.member,after,prefs.favorites_only],|r|Ok((r.get::<_,i64>(0)?,r.get::<_,String>(1)?,r.get::<_,i64>(2)?)))?.collect::<std::result::Result<Vec<_>,_>>()?;
                for (id, title, last) in rows {
                    let posts=tx.prepare("SELECT p.author_id,EXISTS(SELECT 1 FROM posts parent WHERE parent.id=p.reply_to AND parent.author_id=?3) FROM posts p WHERE p.room_id=?1 AND p.id>?2 ORDER BY p.id LIMIT 101")?.query_map(params![id,last,s.member],|r|Ok((r.get::<_,i64>(0)?,r.get::<_,bool>(1)?)))?.collect::<std::result::Result<Vec<_>,_>>()?;
                    let replies = posts
                        .iter()
                        .take(100)
                        .filter(|(author, reply)| *author != s.member && *reply)
                        .count();
                    let summary = if module == CatchupModule::Replies {
                        format!("{replies} direct replies in next 100 unread")
                    } else {
                        format!(
                            "{}{} unread",
                            posts.len().min(100),
                            if posts.len() > 100 { "+" } else { "" }
                        )
                    };
                    items.push(CatchupItem {
                        id,
                        key: id.to_string(),
                        title,
                        summary,
                    });
                }
            }
            CatchupModule::Files => {
                check_window(&tx, w)?;
                items=tx.prepare("SELECT p.id,o.filename,o.bytes,p.published_at,p.object FROM catchup_publications p JOIN file_objects o ON o.id=p.object JOIN file_areas a ON a.id=o.area_id WHERE p.id>?1 AND p.id<=?2 AND o.state='published' AND (a.private=0 OR ?3=1 OR EXISTS(SELECT 1 FROM file_area_grants g WHERE g.area_id=a.id AND g.member_id=?4)) ORDER BY p.id LIMIT 3")?.query_map(params![after.max(w.after),w.through,m.role==Role::Sysop,s.member],|r|{let at:i64=r.get(3)?;Ok(CatchupItem{id:r.get(0)?,key:r.get(4)?,title:r.get(1)?,summary:format!("{} bytes / {}",r.get::<_,i64>(2)?,if at==0{"legacy publication date unknown"}else{"published locally"})})})?.collect::<std::result::Result<Vec<_>,_>>()?;
            }
            CatchupModule::Gatherings => {
                let g_enabled: bool = tx.query_row(
                    "SELECT enabled FROM gathering_settings WHERE singleton=1",
                    [],
                    |r| r.get(0),
                )?;
                if g_enabled {
                    items=tx.prepare("SELECT g.id,g.title,strftime('%Y-%m-%d %H:%M',g.starts_at,'unixepoch') FROM gatherings g JOIN members m ON m.id=g.organizer WHERE g.id>?1 AND g.hidden=0 AND g.cancelled=0 AND g.ends_at>unixepoch() AND m.status='approved' ORDER BY g.id LIMIT 5")?.query_map([after],|r|Ok(CatchupItem{id:r.get(0)?,key:r.get::<_,i64>(0)?.to_string(),title:r.get(1)?,summary:format!("{} UTC",r.get::<_,String>(2)?)}))?.collect::<std::result::Result<Vec<_>,_>>()?;
                }
            }
        }
        let cap = if module == CatchupModule::Files { 2 } else { 4 };
        let next = if items.len() > cap {
            items.truncate(cap);
            items.last().map(|r| r.id)
        } else {
            None
        };
        Ok(CatchupPage { items, next })
    }
    pub fn catchup_target(
        &mut self,
        s: &Session,
        module: CatchupModule,
        key: &str,
        w: &CatchupWindow,
    ) -> Result<CatchupTarget> {
        let id = if module == CatchupModule::Files {
            0
        } else {
            key.parse::<i64>()
                .ok()
                .filter(|id| *id > 0)
                .ok_or(Error::Invalid("positive source ID required"))?
        };
        {
            let tx = self.db.transaction()?;
            current(&tx, &self.identity, s, true, false)?;
            enabled(&tx, Some(module))?;
        }
        match module {
            CatchupModule::Rooms | CatchupModule::Replies => {
                self.unread_cursor(s, id)?;
                Ok(CatchupTarget::Room(id))
            }
            CatchupModule::Gatherings => {
                let g = self.gathering(s, id)?;
                if g.hidden || g.cancelled || g.ended || !g.organizer_approved {
                    return Err(Error::NotFound);
                }
                Ok(CatchupTarget::Gathering(id))
            }
            CatchupModule::Files => {
                let object = {
                    let tx = self.db.transaction()?;
                    current(&tx, &self.identity, s, true, false)?;
                    check_window(&tx, w)?;
                    tx.query_row("SELECT p.object FROM catchup_publications p JOIN file_objects o ON o.id=p.object WHERE p.object=?1 AND p.id>?2 AND p.id<=?3 AND o.state='published'",params![key,w.after,w.through],|r|r.get::<_,String>(0)).optional()?.ok_or(Error::NotFound)?
                };
                self.file_info(s, &object).map_err(|e| {
                    if matches!(e, Error::Denied | Error::NotFound) {
                        Error::NotFound
                    } else {
                        e
                    }
                })?;
                Ok(CatchupTarget::File(object))
            }
            CatchupModule::Notice => Err(Error::Invalid("the notice is already displayed")),
        }
    }
}
pub(crate) fn validate(db: &Connection, deadline: std::time::Instant) -> Result<()> {
    policy(db)?.validate()?;
    if db.query_row("SELECT count(*) FROM catchup_settings", [], |r| {
        r.get::<_, i64>(0)
    })? != 1
    {
        return Err(Error::Invalid("snapshot catch-up policy"));
    }
    if db
        .prepare("SELECT 1 FROM catchup_favorites GROUP BY member HAVING count(*)>16")?
        .exists([])?
    {
        return Err(Error::Invalid("snapshot favorite quota"));
    }
    let high = high_water(db)?;
    let mut q = db.prepare("SELECT file_cursor,revision,saved_at FROM catchup_members")?;
    let mut rows = q.query([])?;
    while let Some(r) = rows.next()? {
        if std::time::Instant::now() >= deadline {
            return Err(Error::Invalid("snapshot catch-up budget"));
        }
        let rev: String = r.get(1)?;
        let cursor: i64 = r.get(0)?;
        let saved: i64 = r.get(2)?;
        if rev.len() != 32
            || !rev
                .bytes()
                .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
            || cursor > high
            || (saved == 0 && cursor != 0)
        {
            return Err(Error::Invalid("snapshot catch-up checkpoint"));
        }
    }
    if db.prepare("SELECT 1 FROM file_objects o WHERE state='published' AND NOT EXISTS(SELECT 1 FROM catchup_publications p WHERE p.object=o.id)")?.exists([])?{return Err(Error::Invalid("snapshot missing file publication"));}
    Ok(())
}

//! Local one-off gatherings. No external calendars, attendee lists or scheduler.
use super::*;
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GatheringConfig {
    pub enabled: bool,
    pub name: String,
    pub compact: bool,
    pub member_organizers: bool,
    pub reminders: bool,
    pub public_counts: bool,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GatheringDraft {
    pub title: String,
    pub body: String,
    pub starts_utc: String,
    pub minutes: u16,
    /// Empty, room:ID, or an exact local #channel. Never an endpoint or command.
    pub place: String,
}
#[derive(Debug, Clone)]
pub struct Gathering {
    pub id: i64,
    pub organizer: String,
    pub revision: String,
    pub draft: GatheringDraft,
    pub ends_utc: String,
    pub cancelled: bool,
    pub hidden: bool,
    pub ended: bool,
    pub mine: bool,
    pub organizer_approved: bool,
    pub counts: Option<(u16, u16)>,
    pub response: u8,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GatheringPlace {
    Room(i64),
    Channel(String),
}
impl GatheringConfig {
    pub fn validate(&self) -> Result<()> {
        ascii(&self.name, 24, 1, false)
    }
}
fn ascii(value: &str, bytes: usize, lines: usize, empty: bool) -> Result<()> {
    if (!empty && value.trim().is_empty())
        || value.len() > bytes
        || value.split('\n').count() > lines
        || value.split('\n').any(|l| l.len() > 72)
        || !value.bytes().all(|b| b == b'\n' || (32..=126).contains(&b))
    {
        return Err(Error::Invalid(
            "gathering text exceeds printable ASCII/line bounds",
        ));
    }
    Ok(())
}
fn key(value: &str) -> Result<()> {
    if value.len() != 32
        || !value
            .bytes()
            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
    {
        return Err(Error::Invalid("invalid gathering revision/request ID"));
    }
    Ok(())
}
fn policy(db: &Connection) -> Result<GatheringConfig> {
    Ok(db.query_row("SELECT enabled,name,compact,member_organizers,reminders,public_counts FROM gathering_settings WHERE singleton=1",[],|r|Ok(GatheringConfig{enabled:r.get(0)?,name:r.get(1)?,compact:r.get(2)?,member_organizers:r.get(3)?,reminders:r.get(4)?,public_counts:r.get(5)?}))?)
}
fn maintain(tx: &Transaction<'_>) -> Result<GatheringConfig> {
    // A reader may encounter another WAL writer. Cleanup is opportunistic;
    // age predicates below still hide expired rows while a writer holds the DB.
    if let Err(e) = tx.execute(
        "DELETE FROM gatherings WHERE ends_at<unixepoch()-2592000",
        [],
    ) && e.sqlite_error_code() != Some(rusqlite::ErrorCode::DatabaseBusy)
    {
        return Err(e.into());
    }
    policy(tx)
}
fn enabled(tx: &Transaction<'_>) -> Result<GatheringConfig> {
    let c = maintain(tx)?;
    if !c.enabled {
        return Err(Error::Invalid("gatherings are disabled"));
    }
    Ok(c)
}
/// Strict UTC calendar text, round-tripped through epoch seconds to reject e.g. Feb 30.
fn utc(db: &Connection, value: &str) -> Result<i64> {
    let b = value.as_bytes();
    if b.len() != 16
        || b[4] != b'-'
        || b[7] != b'-'
        || b[10] != b' '
        || b[13] != b':'
        || b.iter()
            .enumerate()
            .any(|(i, b)| ![4, 7, 10, 13].contains(&i) && !b.is_ascii_digit())
    {
        return Err(Error::Invalid("use UTC YYYY-MM-DD HH:MM"));
    }
    let epoch: Option<i64> = db.query_row("SELECT unixepoch(?1||':00')", [value], |r| r.get(0))?;
    let epoch = epoch.ok_or(Error::Invalid("invalid UTC calendar time"))?;
    let canonical: Option<String> = db.query_row(
        "SELECT strftime('%Y-%m-%d %H:%M',?1,'unixepoch')",
        [epoch],
        |r| r.get(0),
    )?;
    if canonical.as_deref() != Some(value) || !(946684800..=4102444800).contains(&epoch) {
        return Err(Error::Invalid("invalid UTC calendar time"));
    }
    Ok(epoch)
}
fn place(db: &Connection, s: &Session, role: Role, value: &str) -> Result<Option<GatheringPlace>> {
    if value.is_empty() {
        return Ok(None);
    }
    if let Some(id) = value.strip_prefix("room:") {
        let id = id
            .parse::<i64>()
            .ok()
            .filter(|n| *n > 0)
            .ok_or(Error::Invalid("use room:ID or #channel"))?;
        if value != format!("room:{id}")
            || !db
                .prepare("SELECT 1 FROM rooms WHERE id=?1")?
                .exists([id])?
        {
            return Err(Error::Invalid("local destination unavailable"));
        }
        return Ok(Some(GatheringPlace::Room(id)));
    }
    if value.len() > 32
        || !value.starts_with('#')
        || value.len() < 2
        || !value[1..]
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_' || b == b'-')
    {
        return Err(Error::Invalid("use room:ID or an exact lowercase #channel"));
    }
    if !db.prepare("SELECT 1 FROM chat_channels c WHERE name=?1 AND (private=0 OR ?2=1 OR EXISTS(SELECT 1 FROM chat_grants g WHERE g.channel=c.id AND g.member=?3))")?.exists(params![value,role==Role::Sysop,s.member])? {
        return Err(Error::Invalid("local destination unavailable"));
    }
    Ok(Some(GatheringPlace::Channel(value.into())))
}
fn draft(db: &Connection, d: &GatheringDraft) -> Result<(i64, i64)> {
    ascii(&d.title, 64, 1, false)?;
    ascii(&d.body, 480, 8, true)?;
    if !(1..=1440).contains(&d.minutes) {
        return Err(Error::Invalid("duration needs 1-1440 minutes"));
    }
    let start = utc(db, &d.starts_utc)?;
    Ok((start, start + i64::from(d.minutes) * 60))
}
fn row(
    tx: &Transaction<'_>,
    s: &Session,
    m: &Member,
    c: &GatheringConfig,
    id: i64,
) -> Result<Gathering> {
    let found=tx.query_row("SELECT g.id,m.handle,g.revision,g.title,g.body,g.starts_at,g.ends_at,g.place,g.cancelled,g.hidden,g.organizer=?2,m.status='approved' FROM gatherings g JOIN members m ON m.id=g.organizer WHERE g.id=?1 AND g.ends_at>=unixepoch()-2592000 AND (?3=1 OR (m.status='approved' AND (g.hidden=0 OR g.organizer=?2)))",params![id,s.member,m.role==Role::Sysop],|r|Ok((r.get::<_,i64>(0)?,r.get::<_,String>(1)?,r.get::<_,String>(2)?,r.get::<_,String>(3)?,r.get::<_,String>(4)?,r.get::<_,i64>(5)?,r.get::<_,i64>(6)?,r.get::<_,String>(7)?,r.get::<_,bool>(8)?,r.get::<_,bool>(9)?,r.get::<_,bool>(10)?,r.get::<_,bool>(11)?))).optional()?.ok_or(Error::NotFound)?;
    let (
        id,
        organizer,
        revision,
        title,
        body,
        start,
        end,
        p,
        cancelled,
        hidden,
        mine,
        organizer_approved,
    ) = found;
    let (starts_utc,ends_utc,ended):(String,String,bool)=tx.query_row("SELECT strftime('%Y-%m-%d %H:%M',?1,'unixepoch'),strftime('%Y-%m-%d %H:%M',?2,'unixepoch'),?2<=unixepoch()",params![start,end],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?)))?;
    let counts = if c.public_counts || mine || m.role == Role::Sysop {
        Some(tx.query_row("SELECT coalesce(sum(response=1),0),coalesce(sum(response=2),0) FROM gathering_rsvps r JOIN members m ON m.id=r.member WHERE gathering=?1 AND m.status='approved'",[id],|r|Ok((r.get(0)?,r.get(1)?)))?)
    } else {
        None
    };
    let response = tx
        .query_row(
            "SELECT response FROM gathering_rsvps WHERE gathering=?1 AND member=?2",
            params![id, s.member],
            |r| r.get(0),
        )
        .optional()?
        .unwrap_or(0);
    let p = if mine || m.role == Role::Sysop || place(tx, s, m.role, &p).is_ok() {
        p
    } else {
        String::new()
    };
    Ok(Gathering {
        id,
        organizer,
        revision,
        draft: GatheringDraft {
            title,
            body,
            starts_utc,
            minutes: ((end - start) / 60) as u16,
            place: p,
        },
        ends_utc,
        cancelled,
        hidden,
        ended,
        mine,
        organizer_approved,
        counts,
        response,
    })
}
impl Board {
    pub fn gathering_config(&mut self, s: &Session) -> Result<GatheringConfig> {
        let tx = self.db.transaction()?;
        current(&tx, &self.identity, s, true, false)?;
        policy(&tx)
    }
    pub fn configure_gatherings(&mut self, s: &Session, c: &GatheringConfig) -> Result<()> {
        c.validate()?;
        let tx = self
            .db
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        current(&tx, &self.identity, s, true, true)?;
        tx.execute("UPDATE gathering_settings SET enabled=?1,name=?2,compact=?3,member_organizers=?4,reminders=?5,public_counts=?6",params![c.enabled,c.name,c.compact,c.member_organizers,c.reminders,c.public_counts])?;
        tx.commit()?;
        Ok(())
    }
    pub fn gatherings(&mut self, s: &Session, past: bool, offset: usize) -> Result<Vec<Gathering>> {
        if offset > 64 {
            return Err(Error::Invalid("gathering page outside bounds"));
        }
        let tx = self.db.transaction()?;
        let m = current(&tx, &self.identity, s, true, false)?;
        let c = enabled(&tx)?;
        let ids=tx.prepare("SELECT g.id FROM gatherings g JOIN members m ON m.id=g.organizer WHERE g.ends_at>=unixepoch()-2592000 AND (?1=1 OR (m.status='approved' AND (hidden=0 OR organizer=?2))) AND ((?3=0 AND ends_at>unixepoch()) OR (?3=1 AND ends_at<=unixepoch())) ORDER BY g.starts_at,g.id LIMIT 4 OFFSET ?4")?.query_map(params![m.role==Role::Sysop,s.member,past,offset as i64],|r|r.get::<_,i64>(0))?.collect::<std::result::Result<Vec<_>,_>>()?;
        let out = ids
            .into_iter()
            .map(|id| row(&tx, s, &m, &c, id))
            .collect::<Result<Vec<_>>>()?;
        tx.commit()?;
        Ok(out)
    }
    pub fn gathering(&mut self, s: &Session, id: i64) -> Result<Gathering> {
        let tx = self.db.transaction()?;
        let m = current(&tx, &self.identity, s, true, false)?;
        let c = enabled(&tx)?;
        let out = row(&tx, s, &m, &c, id)?;
        tx.commit()?;
        Ok(out)
    }
    pub fn save_gathering(
        &mut self,
        s: &Session,
        id: Option<i64>,
        expected: &str,
        d: &GatheringDraft,
    ) -> Result<i64> {
        key(expected)?;
        let tx = self
            .db
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let m = current(&tx, &self.identity, s, true, false)?;
        let c = enabled(&tx)?;
        if m.role != Role::Sysop && !c.member_organizers {
            return Err(Error::Denied);
        }
        let (start, end) = draft(&tx, d)?;
        place(&tx, s, m.role, &d.place)?;
        let now: i64 = tx.query_row("SELECT unixepoch()", [], |r| r.get(0))?;
        if start > now + 366 * 86400 || end <= now {
            return Err(Error::Invalid(
                "event must end in the future and start within 366 days",
            ));
        }
        let id = if let Some(id) = id {
            let old = row(&tx, s, &m, &c, id)?;
            if !old.mine {
                return Err(Error::Denied);
            }
            if old.revision != expected {
                return Err(Error::RequestConflict);
            }
            if old.cancelled || old.ended {
                return Err(Error::Invalid("cancelled/ended event cannot be edited"));
            }
            if start < now && old.draft.starts_utc != d.starts_utc {
                return Err(Error::Invalid("new start must be in the future"));
            }
            if old.draft.starts_utc != d.starts_utc
                || old.draft.minutes != d.minutes
                || old.draft.place != d.place
            {
                tx.execute("DELETE FROM gathering_rsvps WHERE gathering=?1", [id])?;
            }
            tx.execute("UPDATE gatherings SET title=?1,body=?2,starts_at=?3,ends_at=?4,place=?5,revision=?6,changed_at=unixepoch() WHERE id=?7",params![d.title,d.body,start,end,d.place,request_id()?,id])?;
            id
        } else {
            if m.role != Role::Sysop && !c.member_organizers {
                return Err(Error::Denied);
            }
            if start < now {
                return Err(Error::Invalid("new start must be in the future"));
            }
            if let Some(id) = tx
                .query_row(
                    "SELECT id FROM gatherings WHERE organizer=?1 AND request_id=?2",
                    params![s.member, expected],
                    |r| r.get::<_, i64>(0),
                )
                .optional()?
            {
                let old = row(&tx, s, &m, &c, id)?;
                if old.draft != *d {
                    return Err(Error::RequestConflict);
                }
                tx.commit()?;
                return Ok(id);
            }
            if tx.query_row("SELECT count(*) FROM gatherings", [], |r| {
                r.get::<_, i64>(0)
            })? >= 64
            {
                return Err(Error::Invalid(
                    "64 retained gatherings; remove or allow old events to expire",
                ));
            }
            if m.role != Role::Sysop
                && tx.query_row(
                    "SELECT count(*) FROM gatherings WHERE organizer=?1 AND ends_at>unixepoch()",
                    [s.member],
                    |r| r.get::<_, i64>(0),
                )? >= 8
            {
                return Err(Error::Invalid("eight future events per member organizer"));
            }
            tx.execute("INSERT INTO gatherings(organizer,request_id,revision,title,body,starts_at,ends_at,place,cancelled,hidden,changed_at) VALUES(?1,?2,?2,?3,?4,?5,?6,?7,0,0,unixepoch())",params![s.member,expected,d.title,d.body,start,end,d.place])?;
            tx.last_insert_rowid()
        };
        tx.commit()?;
        Ok(id)
    }
    pub fn cancel_gathering(&mut self, s: &Session, id: i64, expected: &str) -> Result<()> {
        let tx = self
            .db
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let m = current(&tx, &self.identity, s, true, false)?;
        let c = enabled(&tx)?;
        let old = row(&tx, s, &m, &c, id)?;
        if !old.mine {
            return Err(Error::Denied);
        }
        if old.revision != expected {
            return Err(Error::RequestConflict);
        }
        tx.execute(
            "UPDATE gatherings SET cancelled=1,revision=?1,changed_at=unixepoch() WHERE id=?2",
            params![request_id()?, id],
        )?;
        tx.commit()?;
        Ok(())
    }
    pub fn gathering_rsvp(
        &mut self,
        s: &Session,
        id: i64,
        expected: &str,
        response: u8,
    ) -> Result<()> {
        if response > 2 {
            return Err(Error::Invalid("RSVP needs 0:clear, 1:going or 2:maybe"));
        }
        let tx = self
            .db
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let m = current(&tx, &self.identity, s, true, false)?;
        let c = enabled(&tx)?;
        let event = row(&tx, s, &m, &c, id)?;
        if event.revision != expected {
            return Err(Error::RequestConflict);
        }
        if response == 0 {
            tx.execute(
                "DELETE FROM gathering_rsvps WHERE gathering=?1 AND member=?2",
                params![id, s.member],
            )?;
        } else {
            if event.cancelled || event.hidden || event.ended || !event.organizer_approved {
                return Err(Error::Invalid("event is cancelled, hidden or ended"));
            }
            let old = tx
                .prepare("SELECT 1 FROM gathering_rsvps WHERE gathering=?1 AND member=?2")?
                .exists(params![id, s.member])?;
            if !old
                && tx.query_row(
                    "SELECT count(*) FROM gathering_rsvps WHERE gathering=?1",
                    [id],
                    |r| r.get::<_, i64>(0),
                )? >= 250
            {
                return Err(Error::Invalid("250 responses per gathering"));
            }
            tx.execute("INSERT INTO gathering_rsvps VALUES(?1,?2,?3) ON CONFLICT(gathering,member) DO UPDATE SET response=excluded.response",params![id,s.member,response])?;
        }
        tx.commit()?;
        Ok(())
    }
    pub fn moderate_gathering(
        &mut self,
        s: &Session,
        id: i64,
        expected: &str,
        action: &str,
    ) -> Result<()> {
        let tx = self
            .db
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let m = current(&tx, &self.identity, s, true, true)?;
        let c = enabled(&tx)?;
        let old = row(&tx, s, &m, &c, id)?;
        if old.revision != expected {
            return Err(Error::RequestConflict);
        }
        match action {
            "hide" | "show" => {
                tx.execute("UPDATE gatherings SET hidden=?1,revision=?2,changed_at=unixepoch() WHERE id=?3",params![action=="hide",request_id()?,id])?;
            }
            "remove" => {
                tx.execute("DELETE FROM gatherings WHERE id=?1", [id])?;
            }
            _ => return Err(Error::Invalid("use hide, show or remove")),
        }
        tx.commit()?;
        Ok(())
    }
    pub fn gathering_destination(&mut self, s: &Session, id: i64) -> Result<GatheringPlace> {
        let tx = self
            .db
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let m = current(&tx, &self.identity, s, true, false)?;
        let c = enabled(&tx)?;
        let event = row(&tx, s, &m, &c, id)?;
        if event.cancelled || event.hidden || event.ended || !event.organizer_approved {
            return Err(Error::Invalid("event is cancelled, hidden or ended"));
        }
        let p = place(&tx, s, m.role, &event.draft.place)?
            .ok_or(Error::Invalid("no accessible local destination"))?;
        tx.commit()?;
        Ok(p)
    }
    pub fn gathering_reminder(&mut self, s: &Session) -> Result<Option<String>> {
        let tx = self.db.transaction()?;
        let m = current(&tx, &self.identity, s, true, false)?;
        let c = policy(&tx)?;
        if !c.enabled || !c.reminders {
            return Ok(None);
        }
        let id=tx.query_row("SELECT g.id FROM gatherings g JOIN members m ON m.id=g.organizer WHERE m.status='approved' AND hidden=0 AND cancelled=0 AND ends_at>unixepoch() AND starts_at<=unixepoch()+86400 ORDER BY g.starts_at,g.id LIMIT 1",[],|r|r.get::<_,i64>(0)).optional()?;
        let out = if let Some(id) = id {
            let g = row(&tx, s, &m, &c, id)?;
            Some(format!(
                "Gathering #{}: {}\n{} UTC. Open Gatherings from the menu.",
                g.id, g.draft.title, g.draft.starts_utc
            ))
        } else {
            None
        };
        tx.commit()?;
        Ok(out)
    }
}
pub(crate) fn validate(db: &Connection, deadline: std::time::Instant) -> Result<()> {
    policy(db)?.validate()?;
    if db.query_row("SELECT count(*) FROM gathering_settings", [], |r| {
        r.get::<_, i64>(0)
    })? != 1
        || db.query_row("SELECT count(*) FROM gatherings", [], |r| {
            r.get::<_, i64>(0)
        })? > 64
    {
        return Err(Error::Invalid("snapshot gathering quota/policy"));
    }
    let mut q = db
        .prepare("SELECT request_id,revision,title,body,starts_at,ends_at,place FROM gatherings")?;
    let mut rows = q.query([])?;
    while let Some(r) = rows.next()? {
        if std::time::Instant::now() >= deadline {
            return Err(Error::Invalid("snapshot gathering budget"));
        }
        key(&r.get::<_, String>(0)?)?;
        key(&r.get::<_, String>(1)?)?;
        ascii(&r.get::<_, String>(2)?, 64, 1, false)?;
        ascii(&r.get::<_, String>(3)?, 480, 8, true)?;
        let start: i64 = r.get(4)?;
        let end: i64 = r.get(5)?;
        if start % 60 != 0 || (end - start) % 60 != 0 {
            return Err(Error::Invalid("snapshot gathering time"));
        }
        let p: String = r.get(6)?;
        // Structural reference validation only: imported private-channel grants are independent.
        let dummy = Session {
            board: [0; 32],
            member: 0,
            epoch: 0,
        };
        place(db, &dummy, Role::Sysop, &p)?;
    }
    if db
        .prepare("SELECT 1 FROM gathering_rsvps GROUP BY gathering HAVING count(*)>250")?
        .exists([])?
    {
        return Err(Error::Invalid("snapshot RSVP quota"));
    }
    Ok(())
}

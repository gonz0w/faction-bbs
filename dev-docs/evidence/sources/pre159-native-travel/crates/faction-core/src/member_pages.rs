//! Local opt-in social pages; no home directories, host files or identity proofs.
use super::*;
pub const PAGE_FIELDS: [&str; 4] = ["Bio", "Interests", "ASCII signature", "Current project"];
pub const PAGE_FIELD_BYTES: [usize; 4] = [240, 120, 240, 120];
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PageConfig {
    pub enabled: bool,
    pub name: String,
    pub compact: bool,
    pub fields: u8,
    pub history: u8,
    pub days: u16,
    pub moderator_remove: bool,
}
impl PageConfig {
    pub fn validate(&self) -> Result<()> {
        text(&self.name, 24, false)?;
        if !self.name.bytes().all(|b| (32..=126).contains(&b))
            || self.fields > 15
            || !(1..=16).contains(&self.history)
            || !(7..=365).contains(&self.days)
        {
            return Err(Error::Invalid("member-page policy outside bounds"));
        }
        Ok(())
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemberPage {
    pub handle: String,
    pub fields: [String; 4],
    pub visible: u8,
    pub published: bool,
    pub moderated: bool,
    pub revision: Option<String>,
    pub updated: String,
}
#[derive(Debug, Clone)]
pub struct PlanUpdate {
    pub id: i64,
    pub body: String,
    pub visible: bool,
    pub posted: String,
}
pub fn page_text(value: &str, max: usize, lines: usize) -> Result<()> {
    if value.len() > max
        || value.split('\n').count() > lines
        || value.split('\n').any(|l| l.len() > 72)
        || !value.bytes().all(|b| b == b'\n' || (32..=126).contains(&b))
    {
        return Err(Error::Invalid(
            "page text exceeds ASCII byte/line/72-column bounds",
        ));
    }
    Ok(())
}
fn revision(k: &str) -> Result<()> {
    if k.len() != 32
        || !k
            .bytes()
            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
    {
        return Err(Error::Invalid("invalid page revision"));
    }
    Ok(())
}
fn policy(db: &Connection) -> Result<PageConfig> {
    Ok(db.query_row("SELECT enabled,name,compact,fields,history,days,moderator_remove FROM page_settings WHERE singleton=1",[],|r|Ok(PageConfig{enabled:r.get(0)?,name:r.get(1)?,compact:r.get(2)?,fields:r.get(3)?,history:r.get(4)?,days:r.get(5)?,moderator_remove:r.get(6)?}))?)
}
fn maintain(tx: &Transaction<'_>) -> Result<PageConfig> {
    let c = policy(tx)?;
    tx.execute(
        "DELETE FROM member_plans WHERE posted_at<unixepoch()-?1",
        [i64::from(c.days) * 86400],
    )?;
    tx.execute("DELETE FROM member_plans WHERE id IN (SELECT id FROM (SELECT id,row_number() OVER (PARTITION BY member_id ORDER BY id DESC) n FROM member_plans) WHERE n>?1)",[c.history])?;
    Ok(c)
}
fn enabled(tx: &Transaction<'_>) -> Result<PageConfig> {
    let c = policy(tx)?;
    if !c.enabled {
        return Err(Error::Denied);
    }
    Ok(c)
}
fn target(tx: &Transaction<'_>, s: &Session, name: &str, own: bool) -> Result<i64> {
    let id: Option<i64> = tx
        .query_row(
            "SELECT id FROM members WHERE handle=?1 AND status='approved'",
            [name],
            |r| r.get(0),
        )
        .optional()?;
    let id = id.ok_or(Error::NotFound)?;
    if own && id != s.member {
        return Err(Error::Denied);
    }
    Ok(id)
}
fn blank(name: String) -> MemberPage {
    MemberPage {
        handle: name,
        fields: std::array::from_fn(|_| String::new()),
        visible: 15,
        published: true,
        moderated: false,
        revision: None,
        updated: String::new(),
    }
}
impl Board {
    pub fn page_config(&mut self, s: &Session) -> Result<PageConfig> {
        let tx = self.db.transaction()?;
        current(&tx, &self.identity, s, true, false)?;
        policy(&tx)
    }
    pub fn configure_pages(&mut self, s: &Session, c: &PageConfig) -> Result<()> {
        c.validate()?;
        let tx = self
            .db
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        current(&tx, &self.identity, s, true, true)?;
        tx.execute("UPDATE page_settings SET enabled=?1,name=?2,compact=?3,fields=?4,history=?5,days=?6,moderator_remove=?7",params![c.enabled,c.name,c.compact,c.fields,c.history,c.days,c.moderator_remove])?;
        maintain(&tx)?;
        tx.commit()?;
        Ok(())
    }
    pub fn page_handles(&mut self, s: &Session, offset: usize) -> Result<Vec<String>> {
        let tx = self.db.transaction()?;
        current(&tx, &self.identity, s, true, false)?;
        enabled(&tx)?;
        if offset >= 100 {
            return Ok(Vec::new());
        }
        let list=tx.prepare("SELECT m.handle FROM member_pages p JOIN members m ON m.id=p.member_id WHERE m.status='approved' AND p.published=1 AND p.moderated=0 ORDER BY m.handle LIMIT ?1 OFFSET ?2")?.query_map(params![8.min(100-offset) as i64,offset as i64],|r|r.get(0))?.collect::<std::result::Result<Vec<_>,_>>()?;
        Ok(list)
    }
    pub fn member_page(&mut self, s: &Session, name: &str) -> Result<MemberPage> {
        let name = handle(name)?;
        let tx = self.db.transaction()?;
        let me = current(&tx, &self.identity, s, true, false)?;
        let c = enabled(&tx)?;
        let id = target(&tx, s, &name, false)?;
        let row=tx.query_row("SELECT bio,interests,signature,project,visible,published,moderated,revision,strftime('%Y-%m-%d %H:%M UTC',updated_at,'unixepoch') FROM member_pages WHERE member_id=?1",[id],|r|Ok(MemberPage{handle:name.clone(),fields:[r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?],visible:r.get(4)?,published:r.get(5)?,moderated:r.get(6)?,revision:Some(r.get(7)?),updated:r.get(8)?})).optional()?;
        let mut p = match row {
            Some(p) => p,
            None if id == s.member => blank(name),
            None => return Err(Error::NotFound),
        };
        if id != s.member {
            if !p.published || p.moderated {
                return Err(Error::NotFound);
            }
            for (i, f) in p.fields.iter_mut().enumerate() {
                if p.visible & c.fields & (1 << i) == 0 {
                    f.clear()
                }
            }
            p.visible &= c.fields;
            p.revision = None;
        } else {
            debug_assert_eq!(p.handle, me.handle);
        }
        Ok(p)
    }
    pub fn save_member_page(&mut self, s: &Session, page: &MemberPage) -> Result<String> {
        let name = handle(&page.handle)?;
        if page.visible > 15 {
            return Err(Error::Invalid("invalid page visibility"));
        }
        for (i, f) in page.fields.iter().enumerate() {
            page_text(f, PAGE_FIELD_BYTES[i], if i == 0 || i == 2 { 4 } else { 2 })?;
        }
        let key = request_id()?;
        let tx = self
            .db
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        current(&tx, &self.identity, s, true, false)?;
        let c = enabled(&tx)?;
        let id = target(&tx, s, &name, true)?;
        let old:Option<(String,[String;4])>=tx.query_row("SELECT revision,bio,interests,signature,project FROM member_pages WHERE member_id=?1",[id],|r|Ok((r.get(0)?,[r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?]))).optional()?;
        if old.as_ref().map(|o| o.0.as_str()) != page.revision.as_deref() {
            return Err(Error::RequestConflict);
        }
        for (i, f) in page.fields.iter().enumerate() {
            let before = old.as_ref().map_or("", |o| o.1[i].as_str());
            if c.fields & (1 << i) == 0 && !f.is_empty() && f != before {
                return Err(Error::Denied);
            }
        }
        tx.execute("INSERT INTO member_pages VALUES(?1,?2,?3,?4,?5,?6,?7,0,?8,unixepoch()) ON CONFLICT(member_id) DO UPDATE SET bio=excluded.bio,interests=excluded.interests,signature=excluded.signature,project=excluded.project,visible=excluded.visible,published=excluded.published,revision=excluded.revision,updated_at=excluded.updated_at",params![id,page.fields[0],page.fields[1],page.fields[2],page.fields[3],page.visible,page.published,key])?;
        tx.commit()?;
        Ok(key)
    }
    pub fn delete_member_page(&mut self, s: &Session, expected: Option<&str>) -> Result<()> {
        let tx = self
            .db
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        current(&tx, &self.identity, s, true, false)?;
        let old: Option<(String, bool)> = tx
            .query_row(
                "SELECT revision,moderated FROM member_pages WHERE member_id=?1",
                [s.member],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .optional()?;
        if old.as_ref().map(|r| r.0.as_str()) != expected {
            return Err(Error::RequestConflict);
        }
        tx.execute("DELETE FROM member_plans WHERE member_id=?1", [s.member])?;
        if old.as_ref().is_some_and(|r| r.1) {
            // Erase every authored field/update without letting an author erase
            // the sysop's independent publication gate by deleting/recreating.
            tx.execute("UPDATE member_pages SET bio='',interests='',signature='',project='',visible=0,published=0,revision=?1,updated_at=unixepoch() WHERE member_id=?2",params![request_id()?,s.member])?;
        } else {
            tx.execute("DELETE FROM member_pages WHERE member_id=?1", [s.member])?;
        }
        tx.commit()?;
        Ok(())
    }
    pub fn member_plans(&mut self, s: &Session, name: &str) -> Result<Vec<PlanUpdate>> {
        let name = handle(name)?;
        let tx = self
            .db
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        current(&tx, &self.identity, s, true, false)?;
        enabled(&tx)?;
        maintain(&tx)?;
        let id = target(&tx, s, &name, false)?;
        if id != s.member
            && !tx
                .prepare(
                    "SELECT 1 FROM member_pages WHERE member_id=?1 AND published=1 AND moderated=0",
                )?
                .exists([id])?
        {
            return Err(Error::NotFound);
        }
        let rows=tx.prepare("SELECT id,body,visible,strftime('%Y-%m-%d %H:%M UTC',posted_at,'unixepoch') FROM member_plans WHERE member_id=?1 AND (visible=1 OR member_id=?2) ORDER BY id DESC LIMIT 16")?.query_map(params![id,s.member],|r|Ok(PlanUpdate{id:r.get(0)?,body:r.get(1)?,visible:r.get(2)?,posted:r.get(3)?}))?.collect::<std::result::Result<Vec<_>,_>>()?;
        tx.commit()?;
        Ok(rows)
    }
    pub fn add_member_plan(
        &mut self,
        s: &Session,
        key: &str,
        body: &str,
        visible: bool,
    ) -> Result<i64> {
        revision(key)?;
        page_text(body, 480, 8)?;
        if body.trim().is_empty() {
            return Err(Error::Invalid("empty plan update"));
        }
        let tx = self
            .db
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        current(&tx, &self.identity, s, true, false)?;
        enabled(&tx)?;
        maintain(&tx)?;
        if !tx
            .prepare("SELECT 1 FROM member_pages WHERE member_id=?1")?
            .exists([s.member])?
        {
            return Err(Error::Invalid("save a member page before adding a plan"));
        }
        if let Some((id, old, v)) = tx
            .query_row(
                "SELECT id,body,visible FROM member_plans WHERE member_id=?1 AND request_id=?2",
                params![s.member, key],
                |r| {
                    Ok((
                        r.get::<_, i64>(0)?,
                        r.get::<_, String>(1)?,
                        r.get::<_, bool>(2)?,
                    ))
                },
            )
            .optional()?
        {
            if old != body || v != visible {
                return Err(Error::RequestConflict);
            }
            tx.commit()?;
            return Ok(id);
        }
        tx.execute("INSERT INTO member_plans(member_id,request_id,body,visible,posted_at) VALUES(?1,?2,?3,?4,unixepoch())",params![s.member,key,body,visible])?;
        let id = tx.last_insert_rowid();
        maintain(&tx)?;
        tx.commit()?;
        Ok(id)
    }
    pub fn remove_member_plan(&mut self, s: &Session, id: i64) -> Result<()> {
        let tx = self
            .db
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        current(&tx, &self.identity, s, true, false)?;
        if tx.execute(
            "DELETE FROM member_plans WHERE id=?1 AND member_id=?2",
            params![id, s.member],
        )? != 1
        {
            return Err(Error::Denied);
        }
        tx.commit()?;
        Ok(())
    }
    /// Hide/show preserves author data; removing a page requires explicit sysop policy.
    pub fn moderate_member_page(&mut self, s: &Session, name: &str, action: &str) -> Result<()> {
        let name = handle(name)?;
        let tx = self
            .db
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        current(&tx, &self.identity, s, true, true)?;
        let c = enabled(&tx)?;
        let id = target(&tx, s, &name, false)?;
        match action {
            "hide" | "show" => {
                if tx.execute(
                    "UPDATE member_pages SET moderated=?1,revision=?2 WHERE member_id=?3",
                    params![action == "hide", request_id()?, id],
                )? != 1
                {
                    return Err(Error::NotFound);
                }
            }
            "remove" if c.moderator_remove => {
                tx.execute("DELETE FROM member_plans WHERE member_id=?1", [id])?;
                tx.execute("DELETE FROM member_pages WHERE member_id=?1", [id])?;
            }
            _ => return Err(Error::Denied),
        }
        tx.commit()?;
        Ok(())
    }
}
pub(crate) fn validate(db: &Connection, deadline: std::time::Instant) -> Result<()> {
    let c = policy(db)?;
    c.validate()?;
    if db.query_row("SELECT count(*) FROM page_settings", [], |r| {
        r.get::<_, i64>(0)
    })? != 1
    {
        return Err(Error::Invalid("snapshot page policy"));
    }
    let mut q = db.prepare("SELECT bio,interests,signature,project,revision FROM member_pages")?;
    let mut rows = q.query([])?;
    while let Some(r) = rows.next()? {
        if std::time::Instant::now() >= deadline {
            return Err(Error::Invalid("snapshot page budget"));
        }
        for (i, max) in PAGE_FIELD_BYTES.iter().enumerate() {
            page_text(
                &r.get::<_, String>(i)?,
                *max,
                if i == 0 || i == 2 { 4 } else { 2 },
            )?;
        }
        revision(&r.get::<_, String>(4)?)?;
    }
    if db
        .prepare("SELECT 1 FROM member_plans GROUP BY member_id HAVING count(*)>?1")?
        .exists([c.history])?
    {
        return Err(Error::Invalid("snapshot plan quota"));
    }
    if db.prepare("SELECT 1 FROM member_plans p LEFT JOIN member_pages m ON m.member_id=p.member_id WHERE m.member_id IS NULL")?.exists([])?{return Err(Error::Invalid("orphaned plan update"))}
    let mut q = db.prepare("SELECT request_id,body FROM member_plans")?;
    let mut rows = q.query([])?;
    while let Some(r) = rows.next()? {
        if std::time::Instant::now() >= deadline {
            return Err(Error::Invalid("snapshot plan budget"));
        }
        revision(&r.get::<_, String>(0)?)?;
        let b: String = r.get(1)?;
        page_text(&b, 480, 8)?;
        if b.trim().is_empty() {
            return Err(Error::Invalid("empty snapshot plan"));
        }
    }
    Ok(())
}

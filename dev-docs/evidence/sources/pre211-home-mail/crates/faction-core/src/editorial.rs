//! Local editorial records. Authenticated author approval binds one issue token.
use super::*;
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MagazineText {
    pub title: String,
    pub section: String,
    pub rights: String,
    pub provenance: String,
    pub body: String,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MagazineIssueDraft {
    pub number: u16,
    pub edition: u16,
    pub name: String,
    pub title: String,
    pub released: String,
    pub schedule: String,
}
#[derive(Debug, Clone)]
pub struct MagazineSubmission {
    pub id: i64,
    pub author: String,
    pub revision: String,
    pub state: String,
    pub mine: bool,
    pub text: MagazineText,
}
#[derive(Debug, Clone)]
pub struct MagazineEntry {
    pub slot: u8,
    pub submission: i64,
    pub author: String,
    pub mine: bool,
    pub approved: bool,
    pub current: bool,
    pub text: MagazineText,
}
#[derive(Debug, Clone)]
pub struct MagazineIssue {
    pub id: i64,
    pub board: String,
    pub editor: String,
    pub token: String,
    pub state: String,
    pub draft: MagazineIssueDraft,
    pub entries: Vec<MagazineEntry>,
}
#[derive(Debug, Clone)]
pub struct MagazineSummary {
    pub id: i64,
    pub number: u16,
    pub edition: u16,
    pub title: String,
    pub state: String,
}
fn ascii(s: &str, cap: usize, multi: bool) -> Result<()> {
    if s.trim().is_empty()
        || s.len() > cap
        || s.contains("faction-file:")
        || !s
            .bytes()
            .all(|b| (32..=126).contains(&b) || (multi && b == b'\n'))
    {
        return Err(Error::Invalid("bounded inert ASCII magazine text required"));
    }
    Ok(())
}
impl MagazineText {
    pub fn validate(&self) -> Result<()> {
        ascii(&self.title, 64, false)?;
        ascii(&self.section, 24, false)?;
        ascii(&self.rights, 96, false)?;
        ascii(&self.provenance, 128, false)?;
        ascii(&self.body, 4096, true)?;
        if self.body.split('\n').count() > 64 || self.body.split('\n').any(|l| l.len() > 120) {
            return Err(Error::Invalid("64 lines / 120 columns per submission"));
        }
        Ok(())
    }
}
impl MagazineIssueDraft {
    pub fn validate(&self) -> Result<()> {
        if !(1..=9999).contains(&self.number) || !(1..=9999).contains(&self.edition) {
            return Err(Error::Invalid("issue number/revision 1-9999"));
        }
        ascii(&self.name, 24, false)?;
        ascii(&self.title, 64, false)?;
        ascii(&self.released, 32, false)?;
        ascii(&self.schedule, 64, false)
    }
}
impl MagazineIssue {
    pub fn text(&self) -> String {
        let d = &self.draft;
        let mut out = format!(
            "{} / #{} / revision {}\n{}\nBoard: {} / issue {}\nEditor: {}\nEdition: {}\nSchedule: {}\nLocal approved-member edition; no automatic public redistribution permission.\n",
            d.name,
            d.number,
            d.edition,
            d.title,
            self.board,
            self.id,
            self.editor,
            d.released,
            d.schedule
        );
        out.push_str("\nContents\n");
        for e in &self.entries {
            out.push_str(&format!(
                "{}: {} / {} / {}\n",
                e.slot, e.text.section, e.text.title, e.author
            ));
        }
        for e in &self.entries {
            out.push_str(&format!(
                "\n== {} / {} ==\nBy {}\nRights: {}\nProvenance: {}\n\n{}\n",
                e.text.section,
                e.text.title,
                e.author,
                e.text.rights,
                e.text.provenance,
                e.text.body
            ));
        }
        out
    }
}
fn submission(db: &Connection, member: i64, sysop: bool, id: i64) -> Result<MagazineSubmission> {
    db.query_row("SELECT s.id,m.handle,s.revision,s.state,s.author=?2,s.title,s.section,s.rights,s.provenance,s.body FROM magazine_submissions s JOIN members m ON m.id=s.author WHERE s.id=?1 AND (s.author=?2 OR ?3=1)",params![id,member,sysop],|r|Ok(MagazineSubmission{id:r.get(0)?,author:r.get(1)?,revision:r.get(2)?,state:r.get(3)?,mine:r.get(4)?,text:MagazineText{title:r.get(5)?,section:r.get(6)?,rights:r.get(7)?,provenance:r.get(8)?,body:r.get(9)?}})).optional()?.ok_or(Error::NotFound)
}
fn public(db: &Connection, id: i64) -> Result<bool> {
    Ok(db.prepare("SELECT 1 FROM magazine_issues i WHERE i.id=?1 AND i.state='published' AND EXISTS(SELECT 1 FROM magazine_entries WHERE issue=i.id) AND NOT EXISTS(SELECT 1 FROM magazine_entries e JOIN magazine_submissions s ON s.id=e.submission JOIN members m ON m.id=s.author WHERE e.issue=i.id AND (e.approved IS NULL OR e.approved<>i.token OR s.state='withdrawn' OR m.status<>'approved'))")?.exists([id])?)
}
fn access(db: &Connection, member: i64, sysop: bool, id: i64) -> Result<()> {
    if !db
        .prepare("SELECT 1 FROM magazine_issues WHERE id=?1")?
        .exists([id])?
    {
        return Err(Error::NotFound);
    }
    if sysop || public(db, id)? {
        return Ok(());
    }
    let mine=db.prepare("SELECT 1 FROM magazine_entries e JOIN magazine_submissions s ON s.id=e.submission WHERE e.issue=?1 AND s.author=?2")?.exists(params![id,member])?;
    let unsafe_authors=db.prepare("SELECT 1 FROM magazine_entries e JOIN magazine_submissions s ON s.id=e.submission JOIN members m ON m.id=s.author WHERE e.issue=?1 AND (s.state='withdrawn' OR m.status<>'approved')")?.exists([id])?;
    if mine && !unsafe_authors {
        Ok(())
    } else {
        Err(Error::NotFound)
    }
}
fn issue(db: &Connection, member: i64, sysop: bool, id: i64) -> Result<MagazineIssue> {
    access(db, member, sysop, id)?;
    let mut v=db.query_row("SELECT i.id,m.handle,i.token,i.state,i.number,i.edition,i.name,i.title,i.released,i.schedule FROM magazine_issues i JOIN members m ON m.id=i.editor WHERE i.id=?1",[id],|r|Ok(MagazineIssue{id:r.get(0)?,board:String::new(),editor:r.get(1)?,token:r.get(2)?,state:r.get(3)?,draft:MagazineIssueDraft{number:r.get(4)?,edition:r.get(5)?,name:r.get(6)?,title:r.get(7)?,released:r.get(8)?,schedule:r.get(9)?},entries:vec![]}))?;
    v.board = db.query_row("SELECT id FROM file_namespace", [], |r| r.get(0))?;
    v.entries=db.prepare("SELECT e.slot,e.submission,m.handle,s.author=?2,e.approved=?3,e.source_revision=s.revision AND s.state='submitted' AND m.status='approved',e.title,e.section,e.rights,e.provenance,e.body FROM magazine_entries e JOIN magazine_submissions s ON s.id=e.submission JOIN members m ON m.id=s.author WHERE e.issue=?1 ORDER BY e.slot LIMIT 9")?.query_map(params![id,member,v.token],|r|Ok(MagazineEntry{slot:r.get(0)?,submission:r.get(1)?,author:r.get(2)?,mine:r.get(3)?,approved:r.get::<_,Option<bool>>(4)?.unwrap_or(false),current:r.get(5)?,text:MagazineText{title:r.get(6)?,section:r.get(7)?,rights:r.get(8)?,provenance:r.get(9)?,body:r.get(10)?}}))?.collect::<std::result::Result<Vec<_>,_>>()?;
    if v.entries.len() > 8 || v.text().len() > 32768 {
        return Err(Error::Invalid("issue content budget"));
    }
    Ok(v)
}
fn review(
    db: &Connection,
    member: i64,
    sysop: bool,
    id: i64,
    expected: &str,
    draft: bool,
) -> Result<MagazineIssue> {
    file_id(expected)?;
    let v = issue(db, member, sysop, id)?;
    if v.token != expected || (draft && v.state != "draft") {
        return Err(Error::Invalid("issue changed; reopen it"));
    }
    Ok(v)
}
fn rotate(db: &Connection, id: i64, editor: i64) -> Result<()> {
    db.execute(
        "UPDATE magazine_issues SET token=?1,editor=?2 WHERE id=?3",
        params![request_id()?, editor, id],
    )?;
    Ok(())
}
impl Board {
    pub fn magazine_submissions(
        &mut self,
        s: &Session,
        all: bool,
        through: i64,
    ) -> Result<Vec<MagazineSubmission>> {
        if through < 0 {
            return Err(Error::Invalid("negative cursor"));
        }
        let tx = self.db.transaction()?;
        let m = current(&tx, &self.identity, s, true, all)?;
        let ids=tx.prepare("SELECT id FROM magazine_submissions WHERE id<=?1 AND (?2 OR author=?3) ORDER BY id DESC LIMIT 4")?.query_map(params![through,all,s.member],|r|r.get::<_,i64>(0))?.collect::<std::result::Result<Vec<_>,_>>()?;
        ids.into_iter()
            .map(|id| submission(&tx, s.member, m.role == Role::Sysop, id))
            .collect()
    }
    pub fn magazine_submission(&mut self, s: &Session, id: i64) -> Result<MagazineSubmission> {
        let tx = self.db.transaction()?;
        let m = current(&tx, &self.identity, s, true, false)?;
        submission(&tx, s.member, m.role == Role::Sysop, id)
    }
    pub fn magazine_save_submission(
        &mut self,
        s: &Session,
        id: Option<i64>,
        expected: &str,
        text: &MagazineText,
        submit: bool,
    ) -> Result<i64> {
        file_id(expected)?;
        text.validate()?;
        let tx = self
            .db
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        current(&tx, &self.identity, s, true, false)?;
        let state = if submit { "submitted" } else { "draft" };
        if let Some(id) = id {
            let old = submission(&tx, s.member, false, id)?;
            if old.revision != expected || old.state == "withdrawn" {
                return Err(Error::Invalid("submission changed or withdrawn; reopen it"));
            }
            tx.execute("UPDATE magazine_submissions SET revision=?1,state=?2,title=?3,section=?4,rights=?5,provenance=?6,body=?7 WHERE id=?8",params![request_id()?,state,text.title,text.section,text.rights,text.provenance,text.body,id])?;
            tx.commit()?;
            return Ok(id);
        }
        if let Some(id) = tx
            .query_row(
                "SELECT id FROM magazine_submissions WHERE author=?1 AND nonce=?2",
                params![s.member, expected],
                |r| r.get::<_, i64>(0),
            )
            .optional()?
        {
            let old = submission(&tx, s.member, false, id)?;
            return if old.text == *text && old.state == state {
                Ok(id)
            } else {
                Err(Error::Invalid("submission retry differs"))
            };
        }
        if tx.query_row("SELECT count(*) FROM magazine_submissions", [], |r| {
            r.get::<_, i64>(0)
        })? >= 128
            || tx.query_row(
                "SELECT count(*) FROM magazine_submissions WHERE author=?1 AND state<>'withdrawn'",
                [s.member],
                |r| r.get::<_, i64>(0),
            )? >= 8
        {
            return Err(Error::Invalid(
                "submission desk full: 128 retained / 8 active per author",
            ));
        }
        tx.execute("INSERT INTO magazine_submissions(author,nonce,revision,state,title,section,rights,provenance,body) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9)",params![s.member,expected,request_id()?,state,text.title,text.section,text.rights,text.provenance,text.body])?;
        let id = tx.last_insert_rowid();
        tx.commit()?;
        Ok(id)
    }
    pub fn magazine_withdraw_submission(
        &mut self,
        s: &Session,
        id: i64,
        expected: &str,
        delete: bool,
    ) -> Result<()> {
        file_id(expected)?;
        let tx = self
            .db
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let m = current(&tx, &self.identity, s, true, false)?;
        let v = submission(&tx, s.member, m.role == Role::Sysop, id)?;
        if v.revision != expected {
            return Err(Error::Invalid("submission changed; reopen it"));
        }
        if delete {
            if tx
                .prepare("SELECT 1 FROM magazine_entries WHERE submission=?1")?
                .exists([id])?
            {
                return Err(Error::Invalid(
                    "selected submissions must be retained or their issues removed first",
                ));
            }
            tx.execute("DELETE FROM magazine_submissions WHERE id=?1", [id])?;
        } else {
            tx.execute(
                "UPDATE magazine_submissions SET state='withdrawn',revision=?1 WHERE id=?2",
                params![request_id()?, id],
            )?;
            tx.execute("UPDATE magazine_issues SET state='withdrawn' WHERE state='published' AND id IN(SELECT issue FROM magazine_entries WHERE submission=?1)",[id])?;
        }
        tx.commit()?;
        Ok(())
    }
    pub fn magazine_issues(&mut self, s: &Session, through: i64) -> Result<Vec<MagazineSummary>> {
        if through < 0 {
            return Err(Error::Invalid("negative cursor"));
        }
        let tx = self.db.transaction()?;
        let m = current(&tx, &self.identity, s, true, false)?;
        let mut out = vec![];
        let ids = tx
            .prepare("SELECT id FROM magazine_issues WHERE id<=?1 ORDER BY id DESC LIMIT 32")?
            .query_map([through], |r| r.get::<_, i64>(0))?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        for id in ids {
            match access(&tx, s.member, m.role == Role::Sysop, id) {
                Ok(()) => (),
                Err(Error::NotFound) => continue,
                Err(e) => return Err(e),
            }
            out.push(tx.query_row(
                "SELECT id,number,edition,title,state FROM magazine_issues WHERE id=?1",
                [id],
                |r| {
                    Ok(MagazineSummary {
                        id: r.get(0)?,
                        number: r.get(1)?,
                        edition: r.get(2)?,
                        title: r.get(3)?,
                        state: r.get(4)?,
                    })
                },
            )?);
            if out.len() == 4 {
                break;
            }
        }
        Ok(out)
    }
    pub fn magazine_issue(&mut self, s: &Session, id: i64) -> Result<MagazineIssue> {
        let tx = self.db.transaction()?;
        let m = current(&tx, &self.identity, s, true, false)?;
        issue(&tx, s.member, m.role == Role::Sysop, id)
    }
    pub fn magazine_save_issue(
        &mut self,
        s: &Session,
        id: Option<i64>,
        expected: &str,
        d: &MagazineIssueDraft,
    ) -> Result<i64> {
        file_id(expected)?;
        d.validate()?;
        let tx = self
            .db
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        current(&tx, &self.identity, s, true, true)?;
        if let Some(id) = id {
            review(&tx, s.member, true, id, expected, true)?;
            if tx
                .prepare("SELECT 1 FROM magazine_issues WHERE number=?1 AND edition=?2 AND id<>?3")?
                .exists(params![d.number, d.edition, id])?
            {
                return Err(Error::Invalid("issue number/revision already retained"));
            }
            tx.execute("UPDATE magazine_issues SET number=?1,edition=?2,name=?3,title=?4,released=?5,schedule=?6 WHERE id=?7",params![d.number,d.edition,d.name,d.title,d.released,d.schedule,id])?;
            rotate(&tx, id, s.member)?;
            tx.commit()?;
            return Ok(id);
        }
        if let Some(id) = tx
            .query_row(
                "SELECT id FROM magazine_issues WHERE nonce=?1",
                [expected],
                |r| r.get::<_, i64>(0),
            )
            .optional()?
        {
            let old = issue(&tx, s.member, true, id)?;
            return if old.draft == *d {
                Ok(id)
            } else {
                Err(Error::Invalid("issue retry differs"))
            };
        }
        if tx.query_row("SELECT count(*) FROM magazine_issues", [], |r| {
            r.get::<_, i64>(0)
        })? >= 32
        {
            return Err(Error::Invalid("32 retained issues per board"));
        }
        if tx
            .prepare("SELECT 1 FROM magazine_issues WHERE number=?1 AND edition=?2")?
            .exists(params![d.number, d.edition])?
        {
            return Err(Error::Invalid("issue number/revision already retained"));
        }
        tx.execute("INSERT INTO magazine_issues(editor,nonce,token,number,edition,name,title,released,schedule,state) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,'draft')",params![s.member,expected,request_id()?,d.number,d.edition,d.name,d.title,d.released,d.schedule])?;
        let id = tx.last_insert_rowid();
        tx.commit()?;
        Ok(id)
    }
    pub fn magazine_select(
        &mut self,
        s: &Session,
        id: i64,
        expected: &str,
        slot: u8,
        source: i64,
    ) -> Result<()> {
        if !(1..=8).contains(&slot) {
            return Err(Error::Invalid("issue slot 1-8"));
        }
        let tx = self
            .db
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        current(&tx, &self.identity, s, true, true)?;
        review(&tx, s.member, true, id, expected, true)?;
        if source == 0 {
            tx.execute(
                "DELETE FROM magazine_entries WHERE issue=?1 AND slot=?2",
                params![id, slot],
            )?;
        } else {
            let sub = submission(&tx, s.member, true, source)?;
            let author_ok=tx.prepare("SELECT 1 FROM magazine_submissions s JOIN members m ON m.id=s.author WHERE s.id=?1 AND s.state='submitted' AND m.status='approved'")?.exists([source])?;
            if !author_ok {
                return Err(Error::NotFound);
            }
            if tx
                .prepare(
                    "SELECT 1 FROM magazine_entries WHERE issue=?1 AND submission=?2 AND slot<>?3",
                )?
                .exists(params![id, source, slot])?
            {
                return Err(Error::Invalid("submission already selected"));
            }
            tx.execute(
                "DELETE FROM magazine_entries WHERE issue=?1 AND slot=?2",
                params![id, slot],
            )?;
            tx.execute(
                "INSERT INTO magazine_entries VALUES(?1,?2,?3,?4,NULL,?5,?6,?7,?8,?9)",
                params![
                    id,
                    slot,
                    source,
                    sub.revision,
                    sub.text.title,
                    sub.text.section,
                    sub.text.rights,
                    sub.text.provenance,
                    sub.text.body
                ],
            )?;
        }
        rotate(&tx, id, s.member)?;
        issue(&tx, s.member, true, id)?;
        tx.commit()?;
        Ok(())
    }
    pub fn magazine_edit_entry(
        &mut self,
        s: &Session,
        id: i64,
        expected: &str,
        slot: u8,
        text: &MagazineText,
    ) -> Result<()> {
        text.validate()?;
        let tx = self
            .db
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        current(&tx, &self.identity, s, true, true)?;
        review(&tx, s.member, true, id, expected, true)?;
        if tx.execute("UPDATE magazine_entries SET title=?1,section=?2,rights=?3,provenance=?4,body=?5 WHERE issue=?6 AND slot=?7",params![text.title,text.section,text.rights,text.provenance,text.body,id,slot])?!=1{return Err(Error::NotFound);}
        rotate(&tx, id, s.member)?;
        issue(&tx, s.member, true, id)?;
        tx.commit()?;
        Ok(())
    }
    pub fn magazine_approve(
        &mut self,
        s: &Session,
        id: i64,
        expected: &str,
        approve: bool,
    ) -> Result<()> {
        let tx = self
            .db
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let m = current(&tx, &self.identity, s, true, false)?;
        let v = review(&tx, s.member, m.role == Role::Sysop, id, expected, approve)?;
        let own = v.entries.iter().filter(|e| e.mine).collect::<Vec<_>>();
        if own.is_empty() {
            return Err(Error::Denied);
        }
        if approve && own.iter().any(|e| !e.current) {
            return Err(Error::Invalid(
                "source changed; editor must select its current submission again",
            ));
        }
        tx.execute("UPDATE magazine_entries SET approved=?1 WHERE issue=?2 AND submission IN(SELECT id FROM magazine_submissions WHERE author=?3)",params![if approve{Some(expected)}else{None},id,s.member])?;
        if !approve {
            tx.execute(
                "UPDATE magazine_issues SET state='withdrawn' WHERE id=?1 AND state='published'",
                [id],
            )?;
        }
        tx.commit()?;
        Ok(())
    }
    pub fn magazine_publish(&mut self, s: &Session, id: i64, expected: &str) -> Result<()> {
        let tx = self
            .db
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        current(&tx, &self.identity, s, true, true)?;
        let v = review(&tx, s.member, true, id, expected, true)?;
        if v.entries.is_empty() || v.entries.iter().any(|e| !e.current || !e.approved) {
            return Err(Error::Invalid(
                "every current contribution must approve this exact issue revision",
            ));
        }
        tx.execute(
            "UPDATE magazine_issues SET state='published' WHERE id=?1",
            [id],
        )?;
        tx.commit()?;
        Ok(())
    }
    pub fn magazine_withdraw_issue(
        &mut self,
        s: &Session,
        id: i64,
        expected: &str,
        delete: bool,
    ) -> Result<()> {
        let tx = self
            .db
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        current(&tx, &self.identity, s, true, true)?;
        let v = review(&tx, s.member, true, id, expected, false)?;
        if delete {
            if v.state == "published" {
                return Err(Error::Invalid("withdraw published issues before deletion"));
            }
            tx.execute("DELETE FROM magazine_issues WHERE id=?1", [id])?;
        } else {
            tx.execute(
                "UPDATE magazine_issues SET state='withdrawn' WHERE id=?1",
                [id],
            )?;
        }
        tx.commit()?;
        Ok(())
    }
    pub fn magazine_offer(
        &mut self,
        s: &Session,
        id: i64,
        expected: &str,
        object: &str,
    ) -> Result<()> {
        if !object.is_empty() {
            file_id(object)?;
        }
        let tx = self
            .db
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        current(&tx, &self.identity, s, true, true)?;
        let v = review(&tx, s.member, true, id, expected, false)?;
        if v.state != "published" || !public(&tx, id)? {
            return Err(Error::Invalid(
                "offer files only on currently published editions",
            ));
        }
        if !object.is_empty() {
            file_links::visible(&tx, s.member, &v.board, object)?;
        }
        tx.execute(
            "UPDATE magazine_issues SET object=?1 WHERE id=?2",
            params![
                if object.is_empty() {
                    None
                } else {
                    Some(object)
                },
                id
            ],
        )?;
        tx.commit()?;
        Ok(())
    }
    pub fn magazine_file(&mut self, s: &Session, id: i64) -> Result<Option<String>> {
        let tx = self.db.transaction()?;
        let m = current(&tx, &self.identity, s, true, false)?;
        let v = issue(&tx, s.member, m.role == Role::Sysop, id)?;
        if !public(&tx, id)? {
            return Ok(None);
        }
        let object: Option<String> = tx.query_row(
            "SELECT object FROM magazine_issues WHERE id=?1",
            [id],
            |r| r.get(0),
        )?;
        if let Some(o) = object {
            match file_links::visible(&tx, s.member, &v.board, &o) {
                Ok(_) => return Ok(Some(o)),
                Err(Error::Denied | Error::NotFound) => (),
                Err(e) => return Err(e),
            }
        }
        Ok(None)
    }
}
pub(crate) fn owner_edition(db: &Connection, id: i64) -> Result<MagazineIssue> {
    if !public(db, id)? {
        return Err(Error::NotFound);
    }
    issue(db, 0, false, id)
}
pub(crate) fn validate(db: &Connection, deadline: std::time::Instant) -> Result<()> {
    if db.prepare("SELECT 1 FROM magazine_issues i WHERE i.state='published' AND (NOT EXISTS(SELECT 1 FROM magazine_entries WHERE issue=i.id) OR EXISTS(SELECT 1 FROM magazine_entries WHERE issue=i.id AND (approved IS NULL OR approved<>i.token))) LIMIT 1")?.exists([])? {return Err(Error::Invalid("published edition approval binding"));}
    if db.query_row("SELECT count(*) FROM magazine_submissions",[],|r|r.get::<_,i64>(0))?>128 || db.query_row("SELECT count(*) FROM magazine_issues",[],|r|r.get::<_,i64>(0))?>32 || db.prepare("SELECT 1 FROM magazine_submissions WHERE state<>'withdrawn' GROUP BY author HAVING count(*)>8 LIMIT 1")?.exists([])?{return Err(Error::Invalid("snapshot editorial quota"));}
    let ids = db
        .prepare("SELECT id FROM magazine_submissions LIMIT 129")?
        .query_map([], |r| r.get::<_, i64>(0))?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    for id in ids {
        if std::time::Instant::now() >= deadline {
            return Err(Error::Invalid("editorial validation deadline"));
        }
        let v = submission(db, 0, true, id)?;
        file_id(&v.revision)?;
        let n: String = db.query_row(
            "SELECT nonce FROM magazine_submissions WHERE id=?1",
            [id],
            |r| r.get(0),
        )?;
        file_id(&n)?;
        v.text.validate()?;
    }
    let ids = db
        .prepare("SELECT id FROM magazine_issues LIMIT 33")?
        .query_map([], |r| r.get::<_, i64>(0))?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    for id in ids {
        if std::time::Instant::now() >= deadline {
            return Err(Error::Invalid("editorial validation deadline"));
        }
        let v = issue(db, 0, true, id)?;
        file_id(&v.token)?;
        v.draft.validate()?;
        let (n, o): (String, Option<String>) = db.query_row(
            "SELECT nonce,object FROM magazine_issues WHERE id=?1",
            [id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )?;
        file_id(&n)?;
        if let Some(o) = o {
            file_id(&o)?;
        }
        for e in &v.entries {
            e.text.validate()?;
        }
        let keys = db
            .prepare("SELECT source_revision,approved FROM magazine_entries WHERE issue=?1")?
            .query_map([id], |r| {
                Ok((r.get::<_, String>(0)?, r.get::<_, Option<String>>(1)?))
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        for (k, a) in keys {
            file_id(&k)?;
            if let Some(a) = a {
                file_id(&a)?;
            }
        }
    }
    Ok(())
}

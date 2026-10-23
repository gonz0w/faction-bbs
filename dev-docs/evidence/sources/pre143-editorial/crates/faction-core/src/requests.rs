//! Bounded local file requests; references never publish bytes or grant file access.
use super::*;
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileRequestDraft {
    pub title: String,
    pub description: String,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileRequestOffer {
    pub object: String,
    pub rights: String,
    pub provenance: String,
}
#[derive(Debug, Clone)]
pub struct FileRequest {
    pub id: i64,
    pub creator: String,
    pub revision: String,
    pub draft: FileRequestDraft,
    pub state: String,
    pub mine: bool,
    pub offer: Option<FileRequestOffer>,
}
fn ascii(s: &str, cap: usize) -> Result<()> {
    if s.trim().is_empty() || s.len() > cap || !s.bytes().all(|b| (32..=126).contains(&b)) {
        return Err(Error::Invalid(
            "bounded printable ASCII request text required",
        ));
    }
    // File links belong only in the permission-filtered offer, never free-text metadata.
    if s.contains("faction-file:") {
        return Err(Error::Invalid("use fulfillment for file links"));
    }
    Ok(())
}
impl FileRequestDraft {
    pub fn validate(&self) -> Result<()> {
        ascii(&self.title, 64)?;
        ascii(&self.description, 128)
    }
}
impl FileRequestOffer {
    pub fn validate(&self) -> Result<()> {
        file_id(&self.object)?;
        ascii(&self.rights, 96)?;
        ascii(&self.provenance, 128)
    }
}
fn row(tx: &Transaction<'_>, s: &Session, m: &Member, id: i64) -> Result<FileRequest> {
    let (creator,revision,title,description,state,mine,object,rights,provenance):(String,String,String,String,String,bool,Option<String>,String,String)=tx.query_row(
        "SELECT m.handle,r.revision,r.title,r.description,r.state,r.creator=?2,r.object,r.rights,r.provenance FROM file_requests r JOIN members m ON m.id=r.creator WHERE r.id=?1 AND (?3=1 OR m.status='approved' OR r.creator=?2)",
        params![id,s.member,m.role==Role::Sysop],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?,r.get(5)?,r.get(6)?,r.get(7)?,r.get(8)?))).optional()?.ok_or(Error::NotFound)?;
    let mut offer = None;
    if let Some(object) = object {
        let ns: String =
            tx.query_row("SELECT id FROM file_namespace WHERE singleton=1", [], |r| {
                r.get(0)
            })?;
        match file_links::visible(tx, s.member, &ns, &object) {
            Ok(_) => {
                offer = Some(FileRequestOffer {
                    object,
                    rights,
                    provenance,
                })
            }
            Err(Error::Denied | Error::NotFound) => (),
            Err(e) => return Err(e),
        }
    }
    Ok(FileRequest {
        id,
        creator,
        revision,
        draft: FileRequestDraft { title, description },
        state,
        mine,
        offer,
    })
}
impl Board {
    pub fn file_requests(&mut self, s: &Session, through: i64) -> Result<Vec<FileRequest>> {
        if through < 0 {
            return Err(Error::Invalid("negative request cursor"));
        }
        let tx = self.db.transaction()?;
        let m = current(&tx, &self.identity, s, true, false)?;
        let ids=tx.prepare("SELECT r.id FROM file_requests r JOIN members m ON m.id=r.creator WHERE r.id<=?1 AND (?2=1 OR m.status='approved' OR r.creator=?3) ORDER BY r.id DESC LIMIT 4")?.query_map(params![through,m.role==Role::Sysop,s.member],|r|r.get::<_,i64>(0))?.collect::<std::result::Result<Vec<_>,_>>()?;
        ids.into_iter().map(|id| row(&tx, s, &m, id)).collect()
    }
    pub fn file_request(&mut self, s: &Session, id: i64) -> Result<FileRequest> {
        let tx = self.db.transaction()?;
        let m = current(&tx, &self.identity, s, true, false)?;
        row(&tx, s, &m, id)
    }
    pub fn create_file_request(
        &mut self,
        s: &Session,
        nonce: &str,
        d: &FileRequestDraft,
    ) -> Result<i64> {
        file_id(nonce)?;
        d.validate()?;
        let tx = self
            .db
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        current(&tx, &self.identity, s, true, false)?;
        if let Some((id, title, description)) = tx
            .query_row(
                "SELECT id,title,description FROM file_requests WHERE creator=?1 AND nonce=?2",
                params![s.member, nonce],
                |r| {
                    Ok((
                        r.get::<_, i64>(0)?,
                        r.get::<_, String>(1)?,
                        r.get::<_, String>(2)?,
                    ))
                },
            )
            .optional()?
        {
            return if title == d.title && description == d.description {
                Ok(id)
            } else {
                Err(Error::Invalid("request retry differs"))
            };
        }
        if tx.query_row("SELECT count(*) FROM file_requests", [], |r| {
            r.get::<_, i64>(0)
        })? >= 256
            || tx.query_row(
                "SELECT count(*) FROM file_requests WHERE creator=?1 AND state='open'",
                [s.member],
                |r| r.get::<_, i64>(0),
            )? >= 8
        {
            return Err(Error::Invalid(
                "request desk full: 256 retained / 8 open per member",
            ));
        }
        tx.execute("INSERT INTO file_requests(creator,nonce,revision,title,description,state,rights,provenance) VALUES(?1,?2,?3,?4,?5,'open','','')",params![s.member,nonce,request_id()?,d.title,d.description])?;
        let id = tx.last_insert_rowid();
        tx.commit()?;
        Ok(id)
    }
    /// Any current approved member may supply an eligible file. The requester must
    /// also still be approved and able to read it, in the same write transaction.
    pub fn fulfill_file_request(
        &mut self,
        s: &Session,
        id: i64,
        expected: &str,
        o: &FileRequestOffer,
    ) -> Result<()> {
        file_id(expected)?;
        o.validate()?;
        let tx = self
            .db
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let m = current(&tx, &self.identity, s, true, false)?;
        let r = row(&tx, s, &m, id)?;
        if r.revision != expected || r.state != "open" {
            return Err(Error::Invalid("request changed; reopen it"));
        }
        let creator:i64=tx.query_row("SELECT r.creator FROM file_requests r JOIN members m ON m.id=r.creator WHERE r.id=?1 AND m.status='approved'",[id],|r|r.get(0)).optional()?.ok_or(Error::NotFound)?;
        let ns: String =
            tx.query_row("SELECT id FROM file_namespace WHERE singleton=1", [], |r| {
                r.get(0)
            })?;
        file_links::visible(&tx, s.member, &ns, &o.object).map_err(|e| match e {
            Error::Denied => Error::Invalid("file unavailable to fulfiller"),
            e => e,
        })?;
        file_links::visible(&tx, creator, &ns, &o.object).map_err(|e| match e {
            Error::Denied => Error::Invalid("file unavailable to requester"),
            e => e,
        })?;
        tx.execute("UPDATE file_requests SET state='fulfilled',fulfiller=?1,object=?2,rights=?3,provenance=?4,revision=?5 WHERE id=?6",params![s.member,o.object,o.rights,o.provenance,request_id()?,id])?;
        tx.commit()?;
        Ok(())
    }
    /// Author close/delete and sysop moderation use an exact reviewed revision.
    pub fn finish_file_request(
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
        let r = row(&tx, s, &m, id)?;
        if !r.mine && m.role != Role::Sysop {
            return Err(Error::Denied);
        }
        if r.revision != expected {
            return Err(Error::Invalid("request changed; reopen it"));
        }
        if delete {
            tx.execute("DELETE FROM file_requests WHERE id=?1", [id])?;
        } else {
            tx.execute(
                "UPDATE file_requests SET state='closed',revision=?1 WHERE id=?2",
                params![request_id()?, id],
            )?;
        }
        tx.commit()?;
        Ok(())
    }
}
pub(crate) fn validate(db: &Connection, deadline: std::time::Instant) -> Result<()> {
    if db.query_row("SELECT count(*) FROM file_requests",[],|r|r.get::<_,i64>(0))?>256 || db.prepare("SELECT 1 FROM file_requests WHERE state='open' GROUP BY creator HAVING count(*)>8 LIMIT 1")?.exists([])? {return Err(Error::Invalid("snapshot request quota"));}
    let mut q=db.prepare("SELECT nonce,revision,title,description,object,rights,provenance FROM file_requests LIMIT 257")?;
    let rows = q.query_map([], |r| {
        Ok((
            r.get::<_, String>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, String>(2)?,
            r.get::<_, String>(3)?,
            r.get::<_, Option<String>>(4)?,
            r.get::<_, String>(5)?,
            r.get::<_, String>(6)?,
        ))
    })?;
    for r in rows {
        if std::time::Instant::now() >= deadline {
            return Err(Error::Invalid("request validation deadline"));
        }
        let (n, v, title, description, object, rights, provenance) = r?;
        file_id(&n)?;
        file_id(&v)?;
        FileRequestDraft { title, description }.validate()?;
        if let Some(object) = object {
            FileRequestOffer {
                object,
                rights,
                provenance,
            }
            .validate()?;
        }
    }
    Ok(())
}

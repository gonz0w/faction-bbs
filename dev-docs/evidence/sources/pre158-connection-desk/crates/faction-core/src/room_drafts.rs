//! Explicit room-post checkpoints. Plaintext board data, never private mail.
use super::*;
pub const ROOM_DRAFTS: i64 = 4;
pub const ROOM_DRAFT_SECONDS: i64 = 30 * 24 * 60 * 60;
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SavedRoomDraft {
    pub request_id: String,
    pub subject: String,
    pub body: String,
    pub reply_to: Option<i64>,
    pub updated_at: i64,
}
fn valid_body(body: &str) -> Result<()> {
    if body.len() > POST_BYTES
        || body.split('\n').count() > 32
        || body.chars().any(|c| c.is_control() && c != '\n')
    {
        return Err(Error::Invalid(
            "room draft needs up to 1024 bytes / 32 lines without controls",
        ));
    }
    Ok(())
}
fn expire(tx: &Transaction<'_>, s: &Session) -> Result<()> {
    tx.execute(
        "DELETE FROM room_drafts WHERE member_id=?1 AND (epoch!=?2 OR updated_at < unixepoch()-?3)",
        params![s.member, s.epoch, ROOM_DRAFT_SECONDS],
    )?;
    Ok(())
}
impl Board {
    pub fn room_draft(&mut self, s: &Session, room: i64) -> Result<Option<SavedRoomDraft>> {
        let tx = self
            .db
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        current(&tx, &self.identity, s, true, false)?;
        expire(&tx, s)?;
        let saved=tx.query_row("SELECT request_id,subject,body,reply_to,updated_at FROM room_drafts WHERE member_id=?1 AND room_id=?2",params![s.member,room],|r|Ok(SavedRoomDraft{request_id:r.get(0)?,subject:r.get(1)?,body:r.get(2)?,reply_to:r.get(3)?,updated_at:r.get(4)?})).optional()?;
        tx.commit()?;
        Ok(saved)
    }
    /// expected=None creates; a saved request ID replaces only that exact revision.
    pub fn save_room_draft(
        &mut self,
        s: &Session,
        room: i64,
        expected: Option<&str>,
        draft: PostDraft<'_>,
    ) -> Result<SavedRoomDraft> {
        valid_body(draft.body)?;
        post_metadata::subject(draft.subject)?;
        let key = request_id()?;
        let tx = self
            .db
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        current(&tx, &self.identity, s, true, false)?;
        expire(&tx, s)?;
        if !tx
            .prepare("SELECT 1 FROM rooms WHERE id=?1")?
            .exists([room])?
        {
            return Err(Error::NotFound);
        }
        if let Some(parent) = draft.reply_to {
            post_metadata::resolve_subject(&tx, room, Some(parent), "", 0)?;
        }
        let old: Option<String> = tx
            .query_row(
                "SELECT request_id FROM room_drafts WHERE member_id=?1 AND room_id=?2",
                params![s.member, room],
                |r| r.get(0),
            )
            .optional()?;
        if old.as_deref() != expected {
            return Err(Error::RequestConflict);
        }
        if old.is_none() {
            let n: i64 = tx.query_row(
                "SELECT count(*) FROM room_drafts WHERE member_id=?1",
                [s.member],
                |r| r.get(0),
            )?;
            if n >= ROOM_DRAFTS {
                return Err(Error::Invalid(
                    "four saved-room-draft quota reached; delete a checkpoint first",
                ));
            }
        }
        tx.execute("INSERT INTO room_drafts VALUES(?1,?2,?3,?4,?5,?6,?7,unixepoch()) ON CONFLICT(member_id,room_id) DO UPDATE SET epoch=excluded.epoch,request_id=excluded.request_id,subject=excluded.subject,body=excluded.body,reply_to=excluded.reply_to,updated_at=excluded.updated_at",params![s.member,room,s.epoch,key,draft.subject,draft.body,draft.reply_to])?;
        let updated_at = tx.query_row(
            "SELECT updated_at FROM room_drafts WHERE member_id=?1 AND room_id=?2",
            params![s.member, room],
            |r| r.get(0),
        )?;
        tx.commit()?;
        Ok(SavedRoomDraft {
            request_id: key,
            subject: draft.subject.into(),
            body: draft.body.into(),
            reply_to: draft.reply_to,
            updated_at,
        })
    }
    /// Never delete another connection's newer checkpoint.
    pub fn delete_room_draft(&mut self, s: &Session, room: i64, key: &str) -> Result<()> {
        let tx = self
            .db
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        current(&tx, &self.identity, s, true, false)?;
        if tx.execute(
            "DELETE FROM room_drafts WHERE member_id=?1 AND room_id=?2 AND request_id=?3",
            params![s.member, room, key],
        )? != 1
        {
            return Err(Error::RequestConflict);
        }
        tx.commit()?;
        Ok(())
    }
}
pub(crate) fn validate(db: &rusqlite::Connection, deadline: std::time::Instant) -> Result<()> {
    if db
        .prepare("SELECT 1 FROM room_drafts GROUP BY member_id HAVING count(*)>4 LIMIT 1")?
        .exists([])?
    {
        return Err(Error::Invalid("snapshot room-draft quota"));
    }
    let mut q=db.prepare("SELECT d.request_id,d.subject,d.body,d.epoch,m.epoch,d.reply_to,d.room_id,p.room_id FROM room_drafts d JOIN members m ON m.id=d.member_id LEFT JOIN posts p ON p.id=d.reply_to")?;
    let mut rows = q.query([])?;
    while let Some(r) = rows.next()? {
        if std::time::Instant::now() >= deadline {
            return Err(Error::Invalid("snapshot room-draft budget"));
        }
        let key: String = r.get(0)?;
        let subject: String = r.get(1)?;
        let body: String = r.get(2)?;
        if key.len() != 32
            || !key
                .bytes()
                .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
            || r.get::<_, i64>(3)? != r.get::<_, i64>(4)?
            || (r.get::<_, Option<i64>>(5)?.is_some()
                && r.get::<_, i64>(6)? != r.get::<_, i64>(7)?)
        {
            return Err(Error::Invalid("snapshot room-draft binding"));
        }
        post_metadata::subject(&subject)?;
        valid_body(&body)?;
    }
    Ok(())
}

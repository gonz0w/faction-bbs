//! Local article metadata: references, not credentials, publication or home proof.
use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PostMetadata {
    /// Local proto-article identifier. The reserved domain is not a mail endpoint.
    pub article_id: String,
    /// Authored subject or generated default; legacy labels are derived.
    pub subject: String,
    pub created_at: Option<i64>,
    pub created_utc: Option<String>,
    pub legacy: bool,
}
pub(crate) fn valid(id: &str, subject: &str, time: Option<i64>, origin: &str) -> bool {
    id.len() == 32
        && id
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        && (1..=160).contains(&subject.len())
        && subject.bytes().all(|b| (32..=126).contains(&b))
        && match origin {
            "legacy" => time.is_none(),
            "local" => time.is_some_and(|n| (0..=253402300799).contains(&n)),
            _ => false,
        }
}
impl Board {
    /// Preview the same bounded reply default used by the posting transaction.
    pub fn reply_subject(&mut self, session: &Session, room: i64, parent: i64) -> Result<String> {
        let tx = self.db.transaction()?;
        current(&tx, &self.identity, session, true, false)?;
        resolve_subject(&tx, room, Some(parent), "", 0)
    }
    pub fn post_metadata(&mut self, session: &Session, post: i64) -> Result<PostMetadata> {
        let tx = self.db.transaction()?;
        current(&tx, &self.identity, session, true, false)?;
        let row=tx.query_row("SELECT article_id,subject,created_at,origin,strftime('%Y-%m-%d %H:%M UTC',created_at,'unixepoch') FROM post_metadata WHERE post_id=?1",[post],|r| Ok((r.get::<_,String>(0)?,r.get::<_,String>(1)?,r.get::<_,Option<i64>>(2)?,r.get::<_,String>(3)?,r.get::<_,Option<String>>(4)?))).optional()?.ok_or(Error::NotFound)?;
        if !valid(&row.0, &row.1, row.2, &row.3) || row.2.is_some() != row.4.is_some() {
            return Err(Error::UnsupportedSchema);
        }
        Ok(PostMetadata {
            article_id: format!("<{}@faction.invalid>", row.0),
            subject: row.1,
            created_at: row.2,
            created_utc: row.4,
            legacy: row.3 == "legacy",
        })
    }
}
pub(crate) fn validate(db: &Connection, deadline: std::time::Instant) -> Result<()> {
    if db.prepare("SELECT 1 FROM posts p LEFT JOIN post_metadata m ON m.post_id=p.id WHERE m.post_id IS NULL LIMIT 1")?.exists([])? {
        return Err(Error::Invalid("snapshot post metadata missing"));
    }
    let mut stmt = db.prepare("SELECT article_id,subject,created_at,origin FROM post_metadata")?;
    let mut rows = stmt.query([])?;
    while let Some(row) = rows.next()? {
        if std::time::Instant::now() >= deadline {
            return Err(Error::Invalid("snapshot metadata work budget exceeded"));
        }
        if !valid(
            &row.get::<_, String>(0)?,
            &row.get::<_, String>(1)?,
            row.get(2)?,
            &row.get::<_, String>(3)?,
        ) {
            return Err(Error::Invalid("invalid snapshot post metadata"));
        }
    }
    Ok(())
}

pub(crate) fn subject(value: &str) -> Result<()> {
    if value.len() > 160
        || value.trim() != value
        || value.contains("=?")
        || !value.bytes().all(|b| (32..=126).contains(&b))
    {
        return Err(Error::Invalid(
            "subject needs up to 160 printable ASCII bytes, no surrounding spaces or encoded words",
        ));
    }
    Ok(())
}
pub(crate) fn resolve_subject(
    db: &Connection,
    room: i64,
    reply: Option<i64>,
    value: &str,
    id: i64,
) -> Result<String> {
    if !value.is_empty() {
        return Ok(value.into());
    }
    if let Some(parent) = reply {
        let original: String = db.query_row("SELECT m.subject FROM post_metadata m JOIN posts p ON p.id=m.post_id WHERE p.id=?1 AND p.room_id=?2", params![parent,room], |r| r.get(0)).optional()?.ok_or(Error::Invalid("reply must reference a post in this room"))?;
        subject(&original)?;
        let mut result = if original
            .get(..4)
            .is_some_and(|s| s.eq_ignore_ascii_case("Re: "))
        {
            original
        } else {
            format!("Re: {original}")
        };
        result.truncate(160);
        return Ok(result.trim_end().into());
    }
    Ok(format!("Message #{id}"))
}

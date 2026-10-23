//! Local article metadata: references, not credentials, publication or home proof.
use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PostMetadata {
    /// Local proto-article identifier. The reserved domain is not a mail endpoint.
    pub article_id: String,
    /// Generated label; the initial composer does not collect authored subjects.
    pub subject: String,
    pub created_at: Option<i64>,
    pub created_utc: Option<String>,
    pub legacy: bool,
}
fn valid(id: &str, subject: &str, time: Option<i64>, origin: &str) -> bool {
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

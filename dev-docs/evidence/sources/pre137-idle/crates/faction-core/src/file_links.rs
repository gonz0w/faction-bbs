//! Public local file references. Possession of a reference never grants access.
use super::*;
use std::ops::Range;
const PREFIX: &str = "faction-file:";
const LENGTH: usize = PREFIX.len() + 65;
const LINKS: usize = 4;
fn references(body: &str) -> Result<Vec<(Range<usize>, &str, &str)>> {
    let mut rows = Vec::new();
    let mut start = 0;
    while let Some(offset) = body[start..].find(PREFIX) {
        let from = start + offset;
        let end = from + LENGTH;
        let token = body.get(from + PREFIX.len()..end);
        let parts = token
            .and_then(|s| s.split_once('/'))
            .filter(|(board, object)| file_id(board).is_ok() && file_id(object).is_ok());
        let Some((board, object)) = parts else {
            start = from + PREFIX.len();
            continue;
        };
        if body
            .as_bytes()
            .get(end)
            .is_some_and(|b| b.is_ascii_alphanumeric() || b"/_-".contains(b))
        {
            start = from + PREFIX.len();
            continue;
        }
        rows.push((from..end, board, object));
        if rows.len() > LINKS {
            return Err(Error::Invalid("at most four file references per message"));
        }
        start = end;
    }
    Ok(rows)
}
fn namespace(db: &Connection) -> Result<String> {
    let id: String = db.query_row("SELECT id FROM file_namespace WHERE singleton=1", [], |r| {
        r.get(0)
    })?;
    file_id(&id)?;
    Ok(id)
}
pub(crate) fn visible(
    tx: &Transaction<'_>,
    member: i64,
    board: &str,
    object: &str,
) -> Result<AreaFile> {
    if namespace(tx)? != board {
        return Err(Error::Denied);
    }
    let row: Option<(i64,String,u32,String,String)> = tx.query_row("SELECT o.area_id,o.filename,o.bytes,o.sha256,m.handle FROM file_objects o JOIN members m ON m.id=o.uploader_id WHERE o.id=?1 AND o.state='published'", [object], |r| Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?))).optional()?;
    let (area, filename, bytes, sha256, uploader) = row.ok_or(Error::Denied)?;
    files::scope(tx, member, area)?;
    Ok(AreaFile {
        id: object.into(),
        filename,
        bytes,
        sha256,
        state: "published".into(),
        uploader,
        description: String::new(),
    })
}
pub(crate) fn validate_links(tx: &Transaction<'_>, member: i64, body: &str) -> Result<()> {
    for (_, board, object) in references(body)? {
        visible(tx, member, board, object)?;
    }
    Ok(())
}
pub(crate) fn render_links(tx: &Transaction<'_>, member: i64, body: &str) -> Result<String> {
    let mut out = String::new();
    let mut start = 0;
    for (range, board, object) in references(body)? {
        out.push_str(&body[start..range.start]);
        match visible(tx, member, board, object) {
            Ok(f) => {
                out.push_str(&body[range.clone()]);
                out.push_str(&format!(" [File: {} ({} bytes)]", f.filename, f.bytes));
            }
            Err(Error::Denied) => out.push_str("[file unavailable]"),
            Err(e) => return Err(e),
        }
        start = range.end;
    }
    out.push_str(&body[start..]);
    Ok(out)
}
impl Board {
    /// Stable public namespace, deliberately independent of session identity.
    pub fn file_namespace(&mut self, s: &Session) -> Result<String> {
        let tx = self.db.transaction()?;
        current(&tx, &self.identity, s, true, false)?;
        namespace(&tx)
    }
    pub fn file_reference(&mut self, s: &Session, object: &str) -> Result<String> {
        file_id(object)?;
        let tx = self.db.transaction()?;
        current(&tx, &self.identity, s, true, false)?;
        let board = namespace(&tx)?;
        visible(&tx, s.member, &board, object)?;
        Ok(format!("{PREFIX}{board}/{object}"))
    }
    /// Resolve on this board only; never interpret a reference as a network locator.
    pub fn file_resolve(&mut self, s: &Session, reference: &str) -> Result<AreaFile> {
        let rows = references(reference)?;
        if rows.len() != 1 || rows[0].0 != (0..reference.len()) {
            return Err(Error::Denied);
        }
        let tx = self.db.transaction()?;
        current(&tx, &self.identity, s, true, false)?;
        visible(&tx, s.member, rows[0].1, rows[0].2)
    }
}
pub(crate) fn validate_namespace(db: &Connection) -> Result<()> {
    namespace(db)?;
    let count: i64 = db.query_row("SELECT count(*) FROM file_namespace", [], |r| r.get(0))?;
    if count != 1 {
        return Err(Error::Invalid("one public file namespace required"));
    }
    Ok(())
}

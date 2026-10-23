//! Bounded local discussion discovery. No mail, ranking or content index.
use super::*;
use std::time::Instant;
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscussionConfig {
    pub enabled: bool,
    pub name: String,
    pub compact: bool,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiscussionField {
    Subject,
    Body,
    Both,
}
#[derive(Debug, Clone)]
pub struct DiscussionQuery {
    pub text: String,
    pub room: Option<i64>,
    pub author: String,
    pub field: DiscussionField,
}
impl DiscussionQuery {
    pub fn validate(&self) -> Result<()> {
        text(&self.text, 64, false)?;
        if self.text.trim().is_empty() {
            return Err(Error::Invalid("enter literal search text"));
        }
        if self.room.is_some_and(|id| id < 1) {
            return Err(Error::Invalid("positive room ID required"));
        }
        if !self.author.is_empty() {
            handle(&self.author)?;
        }
        Ok(())
    }
}
#[derive(Debug, Clone)]
pub struct DiscussionHit {
    pub id: i64,
    pub room: i64,
    pub room_title: String,
    pub author: String,
    pub subject: String,
    pub article: String,
    pub snippet: String,
}
#[derive(Debug, Clone)]
pub struct DiscussionItem {
    pub hit: DiscussionHit,
    pub body: String,
    pub reply: Option<i64>,
    pub posted: String,
}
#[derive(Debug, Clone)]
pub struct DiscussionPage {
    pub hits: Vec<DiscussionHit>,
    pub next: Option<i64>,
    pub scanned: usize,
}
impl DiscussionConfig {
    pub fn validate(&self) -> Result<()> {
        if self.name.trim().is_empty()
            || self.name.len() > 24
            || !self.name.bytes().all(|b| (32..=126).contains(&b))
        {
            return Err(Error::Invalid(
                "discussion label needs 1-24 printable ASCII bytes",
            ));
        }
        Ok(())
    }
}
fn policy(db: &Connection) -> Result<DiscussionConfig> {
    Ok(db.query_row(
        "SELECT enabled,name,compact FROM discussion_settings WHERE singleton=1",
        [],
        |r| {
            Ok(DiscussionConfig {
                enabled: r.get(0)?,
                name: r.get(1)?,
                compact: r.get(2)?,
            })
        },
    )?)
}
fn enabled(db: &Connection) -> Result<()> {
    if !policy(db)?.enabled {
        return Err(Error::Invalid("discussion discovery is disabled"));
    }
    Ok(())
}
fn item(tx: &Transaction<'_>, member: i64, id: i64) -> Result<DiscussionItem> {
    let (room,title,author,subject,article,body,reply,posted):(i64,String,String,String,String,String,Option<i64>,Option<String>)=tx.query_row("SELECT p.room_id,r.title,m.handle,d.subject,d.article_id,p.body,p.reply_to,strftime('%Y-%m-%d %H:%M UTC',d.created_at,'unixepoch') FROM posts p JOIN rooms r ON r.id=p.room_id JOIN members m ON m.id=p.author_id JOIN post_metadata d ON d.post_id=p.id WHERE p.id=?1",[id],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?,r.get(5)?,r.get(6)?,r.get(7)?))).optional()?.ok_or(Error::NotFound)?;
    let body = if body.contains("faction-file:") {
        file_links::render_links(tx, member, &body)?
    } else {
        body
    };
    let snippet = body
        .chars()
        .take(120)
        .map(|c| if c == '\n' { ' ' } else { c })
        .collect();
    Ok(DiscussionItem {
        hit: DiscussionHit {
            id,
            room,
            room_title: title,
            author,
            subject,
            article: format!("<{article}@faction.invalid>"),
            snippet,
        },
        body,
        reply,
        posted: posted.unwrap_or_else(|| "Unknown (legacy)".into()),
    })
}
impl Board {
    pub fn discussion_config(&mut self, s: &Session) -> Result<DiscussionConfig> {
        let tx = self.db.transaction()?;
        current(&tx, &self.identity, s, true, false)?;
        policy(&tx)
    }
    pub fn configure_discussion(&mut self, s: &Session, c: &DiscussionConfig) -> Result<()> {
        c.validate()?;
        let tx = self
            .db
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        current(&tx, &self.identity, s, true, true)?;
        tx.execute(
            "UPDATE discussion_settings SET enabled=?1,name=?2,compact=?3",
            params![c.enabled, c.name, c.compact],
        )?;
        tx.commit()?;
        Ok(())
    }
    pub fn discussion_start(&mut self, s: &Session) -> Result<i64> {
        let tx = self.db.transaction()?;
        current(&tx, &self.identity, s, true, false)?;
        enabled(&tx)?;
        Ok(tx.query_row("SELECT coalesce(max(id),0) FROM posts", [], |r| r.get(0))?)
    }
    /// Newest first, one bounded scan. No global total or persisted query is computed.
    pub fn discussion_search(
        &mut self,
        s: &Session,
        q: &DiscussionQuery,
        through: i64,
    ) -> Result<DiscussionPage> {
        q.validate()?;
        if through < 0 {
            return Err(Error::Invalid("negative discussion cursor"));
        }
        let tx = self.db.transaction()?;
        current(&tx, &self.identity, s, true, false)?;
        enabled(&tx)?;
        let ids = if let Some(room) = q.room {
            if !tx
                .prepare("SELECT 1 FROM rooms WHERE id=?1")?
                .exists([room])?
            {
                return Err(Error::NotFound);
            }
            tx.prepare("SELECT id FROM posts INDEXED BY room_posts WHERE room_id=?1 AND id<=?2 ORDER BY id DESC LIMIT 101")?.query_map(params![room,through],|r|r.get::<_,i64>(0))?.collect::<std::result::Result<Vec<_>,_>>()?
        } else if !q.author.is_empty() {
            let author = handle(&q.author)?;
            let member = tx
                .query_row("SELECT id FROM members WHERE handle=?1", [author], |r| {
                    r.get::<_, i64>(0)
                })
                .optional()?
                .unwrap_or(0);
            tx.prepare("SELECT id FROM posts INDEXED BY discussion_author_posts WHERE author_id=?1 AND id<=?2 ORDER BY id DESC LIMIT 101")?.query_map(params![member,through],|r|r.get::<_,i64>(0))?.collect::<std::result::Result<Vec<_>,_>>()?
        } else {
            tx.prepare("SELECT id FROM posts WHERE id<=?1 ORDER BY id DESC LIMIT 101")?
                .query_map([through], |r| r.get::<_, i64>(0))?
                .collect::<std::result::Result<Vec<_>, _>>()?
        };
        let deadline = Instant::now() + Duration::from_millis(20);
        let needle = q.text.to_ascii_lowercase();
        let mut hits = vec![];
        let mut scanned = 0;
        let mut last = 0;
        for id in ids.iter().take(100) {
            if scanned > 0 && Instant::now() >= deadline {
                break;
            }
            let value = item(&tx, s.member, *id)?;
            scanned += 1;
            last = *id;
            if !q.author.is_empty() && !value.hit.author.eq_ignore_ascii_case(&q.author) {
                continue;
            }
            let found = match q.field {
                DiscussionField::Subject => {
                    value.hit.subject.to_ascii_lowercase().contains(&needle)
                }
                DiscussionField::Body => value.body.to_ascii_lowercase().contains(&needle),
                DiscussionField::Both => {
                    value.hit.subject.to_ascii_lowercase().contains(&needle)
                        || value.body.to_ascii_lowercase().contains(&needle)
                }
            };
            if found {
                hits.push(value.hit);
                if hits.len() == 4 {
                    break;
                }
            }
        }
        let next = (scanned < ids.len() && last > 1).then_some(last.saturating_sub(1));
        Ok(DiscussionPage {
            hits,
            next,
            scanned,
        })
    }
    pub fn discussion_item(&mut self, s: &Session, id: i64) -> Result<DiscussionItem> {
        let tx = self.db.transaction()?;
        current(&tx, &self.identity, s, true, false)?;
        enabled(&tx)?;
        item(&tx, s.member, id)
    }
    pub fn discussion_bookmark(&mut self, s: &Session, id: i64, add: bool) -> Result<()> {
        let tx = self
            .db
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        current(&tx, &self.identity, s, true, false)?;
        enabled(&tx)?;
        if id < 1 {
            return Err(Error::Invalid("positive post ID required"));
        }
        if add {
            item(&tx, s.member, id)?;
            let exists = tx
                .prepare("SELECT 1 FROM discussion_bookmarks WHERE member=?1 AND post=?2")?
                .exists(params![s.member, id])?;
            let count: i64 = tx.query_row(
                "SELECT count(*) FROM discussion_bookmarks WHERE member=?1",
                [s.member],
                |r| r.get(0),
            )?;
            if !exists && count >= 128 {
                return Err(Error::Invalid("128 bookmarks per member"));
            }
            tx.execute(
                "INSERT OR IGNORE INTO discussion_bookmarks VALUES(?1,?2)",
                params![s.member, id],
            )?;
        } else {
            tx.execute(
                "DELETE FROM discussion_bookmarks WHERE member=?1 AND post=?2",
                params![s.member, id],
            )?;
        }
        tx.commit()?;
        Ok(())
    }
    pub fn discussion_bookmarks(&mut self, s: &Session, through: i64) -> Result<DiscussionPage> {
        if through < 0 {
            return Err(Error::Invalid("negative bookmark cursor"));
        }
        let tx = self.db.transaction()?;
        current(&tx, &self.identity, s, true, false)?;
        enabled(&tx)?;
        let ids=tx.prepare("SELECT b.post FROM discussion_bookmarks b JOIN posts p ON p.id=b.post JOIN rooms r ON r.id=p.room_id WHERE b.member=?1 AND b.post<=?2 ORDER BY b.post DESC LIMIT 5")?.query_map(params![s.member,through],|r|r.get::<_,i64>(0))?.collect::<std::result::Result<Vec<_>,_>>()?;
        let mut hits = vec![];
        for id in ids.iter().take(4) {
            hits.push(item(&tx, s.member, *id)?.hit);
        }
        let next = (ids.len() > 4).then(|| ids[3] - 1);
        Ok(DiscussionPage {
            hits,
            next,
            scanned: ids.len().min(4),
        })
    }
    /// Copyable paged reference export, re-fetching live visible data; never a host path.
    pub fn discussion_bookmark_export(
        &mut self,
        s: &Session,
        through: i64,
    ) -> Result<(String, Option<i64>)> {
        let page = self.discussion_bookmarks(s, through)?;
        let board = self.file_namespace(s)?;
        let mut out = format!("FACTION-BOOKMARKS/1 board={board}\n");
        for h in page.hits {
            let subject = h
                .subject
                .as_bytes()
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect::<String>();
            out.push_str(&format!("{}\t{}\t{}\t{subject}\n", h.room, h.id, h.article));
        }
        Ok((out, page.next))
    }
}
pub(crate) fn validate(db: &Connection, _deadline: Instant) -> Result<()> {
    policy(db)?.validate()?;
    if db.query_row("SELECT count(*) FROM discussion_settings", [], |r| {
        r.get::<_, i64>(0)
    })? != 1
        || db
            .prepare(
                "SELECT 1 FROM discussion_bookmarks GROUP BY member HAVING count(*)>128 LIMIT 1",
            )?
            .exists([])?
    {
        return Err(Error::Invalid(
            "snapshot discussion policy or bookmark quota",
        ));
    }
    Ok(())
}

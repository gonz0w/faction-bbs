//! Local proto-article projection only. No injection, publication or remote import.
use super::*;
use faction_articles::{Article, Headers, REFERENCES};

impl Board {
    /// Exact local text profile for an approved reader. Unavailable file links,
    /// unknown legacy dates and unsupported profiles fail without changing data.
    /// The returned value is not authorization to publish or send a room.
    pub fn room_article(&mut self, session: &Session, post: i64) -> Result<Article> {
        let tx = self.db.transaction()?;
        current(&tx, &self.identity, session, true, false)?;
        let (room, mut parent, body, id, subject, time, origin): (i64, Option<i64>, String, String, String, Option<i64>, String) = tx.query_row(
            "SELECT p.room_id,p.reply_to,p.body,m.article_id,m.subject,m.created_at,m.origin FROM posts p JOIN post_metadata m ON m.post_id=p.id WHERE p.id=?1", [post],
            |r| Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?,r.get(5)?,r.get(6)?))).optional()?.ok_or(Error::NotFound)?;
        if !post_metadata::valid(&id, &subject, time, &origin) {
            return Err(Error::UnsupportedSchema);
        }
        let time = time.ok_or(Error::Invalid(
            "legacy posting date is unknown; no wire article can be synthesized",
        ))?;
        // Keep canonical bodies exact. Never create viewer-dependent copies under
        // one ID or reveal a currently inaccessible private file reference.
        file_links::validate_links(&tx, session.member, &body)?;
        let mut references = Vec::new();
        let mut previous = post;
        while let Some(next) = parent {
            if references.len() == REFERENCES || next >= previous {
                return Err(Error::Invalid(
                    "reply chain exceeds the local article profile or is cyclic",
                ));
            }
            let (r, reply, token, title, stamp, provenance): (i64, Option<i64>, String, String, Option<i64>, String) = tx.query_row(
                "SELECT p.room_id,p.reply_to,m.article_id,m.subject,m.created_at,m.origin FROM posts p JOIN post_metadata m ON m.post_id=p.id WHERE p.id=?1", [next],
                |r| Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?,r.get(5)?))).optional()?.ok_or(Error::UnsupportedSchema)?;
            if r != room || !post_metadata::valid(&token, &title, stamp, &provenance) {
                return Err(Error::UnsupportedSchema);
            }
            references.push(format!("<{token}@faction.invalid>"));
            previous = next;
            parent = reply;
        }
        references.reverse();
        let raw: String = tx.query_row(
            "SELECT strftime('%d %m %Y %H:%M:%S +0000',?1,'unixepoch')",
            [time],
            |r| r.get(0),
        )?;
        let month = raw[3..5]
            .parse::<usize>()
            .ok()
            .filter(|m| (1..=12).contains(m))
            .ok_or(Error::UnsupportedSchema)?;
        let month = [
            "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
        ][month - 1];
        let date = format!("{} {} {}", &raw[..2], month, &raw[6..]);
        Article::new(
            Headers {
                message_id: format!("<{id}@faction.invalid>"),
                // A per-article alias deliberately avoids handles or a cross-board
                // identifier. It is neither a mailbox nor authenticated author proof.
                from: format!("author-{id}@authors.faction.invalid"),
                date,
                subject,
                newsgroups: vec![format!("faction.local.r{room}")],
                path: "not-for-mail".into(),
                references,
            },
            body,
        )
        .map_err(|_| Error::Invalid("post is outside the supported local article text profile"))
    }
}

//! Opt-in availability and bodyless invitations during a live local chat lease.
use super::*;
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SocialConfig {
    pub enabled: bool,
    pub name: String,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SocialPerson {
    pub handle: String,
    pub mode: String,
    pub status: String,
    pub sysop: bool,
}
#[derive(Debug)]
pub struct SocialRoster {
    pub people: Vec<SocialPerson>,
    pub next: Option<String>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SocialPrefs {
    pub muted: bool,
    pub quiet: Option<(u16, u16)>,
    pub blocks: Vec<String>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SocialPage {
    pub id: i64,
    pub from: String,
    pub to: String,
    pub state: String,
    pub outgoing: bool,
}
fn ascii(s: &str, cap: usize, empty: bool) -> Result<()> {
    if s.len() > cap
        || (!empty && s.trim().is_empty())
        || !s.bytes().all(|b| (32..=126).contains(&b))
    {
        return Err(Error::Invalid(
            "bounded printable ASCII social text required",
        ));
    }
    Ok(())
}
fn now(db: &Connection) -> Result<i64> {
    Ok(db.query_row("SELECT unixepoch()", [], |r| r.get(0))?)
}
fn config(db: &Connection) -> Result<SocialConfig> {
    Ok(db.query_row(
        "SELECT enabled,name FROM social_settings WHERE singleton=1",
        [],
        |r| {
            Ok(SocialConfig {
                enabled: r.get(0)?,
                name: r.get(1)?,
            })
        },
    )?)
}
fn enabled(db: &Connection) -> Result<SocialConfig> {
    let c = config(db)?;
    if !c.enabled {
        return Err(Error::Invalid("social lounge disabled by sysop"));
    }
    Ok(c)
}
fn prefs(db: &Connection, member: i64) -> Result<SocialPrefs> {
    let row = db
        .query_row(
            "SELECT muted,quiet_start,quiet_end FROM social_prefs WHERE member=?1",
            [member],
            |r| {
                Ok((
                    r.get::<_, bool>(0)?,
                    r.get::<_, Option<u16>>(1)?,
                    r.get::<_, Option<u16>>(2)?,
                ))
            },
        )
        .optional()?;
    let blocks = db
        .prepare("SELECT handle FROM social_blocks WHERE member=?1 ORDER BY handle LIMIT 17")?
        .query_map([member], |r| r.get(0))?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    if blocks.len() > 16 {
        return Err(Error::Invalid("social block quota"));
    }
    Ok(SocialPrefs {
        muted: row.is_some_and(|r| r.0),
        quiet: row.and_then(|r| r.1.zip(r.2)),
        blocks,
    })
}
fn quiet(p: &SocialPrefs, at: i64) -> bool {
    let Some((start, end)) = p.quiet else {
        return false;
    };
    let minute = ((at / 60) % 1440) as u16;
    if start < end {
        minute >= start && minute < end
    } else {
        minute >= start || minute < end
    }
}
fn person(
    db: &Connection,
    id: &[u8],
    at: i64,
    paging: bool,
) -> Result<Option<(i64, SocialPerson)>> {
    let row=db.query_row("SELECT p.member,m.handle,p.mode,p.status,m.role='sysop' FROM social_presence p JOIN chat_sessions c ON c.id=p.session AND c.member=p.member JOIN members m ON m.id=p.member WHERE p.session=?1 AND m.status='approved' AND m.epoch=c.epoch AND c.touched>?2-60 AND c.touched<=?2+5",params![id,at],|r|Ok((r.get::<_,i64>(0)?,SocialPerson{handle:r.get(1)?,mode:r.get(2)?,status:r.get(3)?,sysop:r.get(4)?}))).optional()?;
    if let Some((member, v)) = row {
        let p = prefs(db, member)?;
        if !paging || (v.mode == "available" && !p.muted && !quiet(&p, at)) {
            return Ok(Some((member, v)));
        }
    }
    Ok(None)
}
fn blocked(db: &Connection, a: i64, b: i64) -> Result<bool> {
    Ok(db.prepare("SELECT 1 FROM social_blocks x JOIN members m ON m.handle=x.handle WHERE (x.member=?1 AND m.id=?2) OR (x.member=?2 AND m.id=?1)")?.exists(params![a,b])?)
}
fn cleanup(tx: &Transaction<'_>, at: i64) -> Result<()> {
    tx.execute("DELETE FROM social_presence WHERE NOT EXISTS(SELECT 1 FROM chat_sessions c JOIN members m ON m.id=c.member WHERE c.id=social_presence.session AND c.member=social_presence.member AND m.status='approved' AND m.epoch=c.epoch AND c.touched>?1-60 AND c.touched<=?1+5)",[at])?;
    tx.execute(
        "DELETE FROM social_pages WHERE created<=?1-60 OR created>?1+5",
        [at],
    )?;
    tx.execute("DELETE FROM social_cooldowns WHERE last_page<=?1-30", [at])?;
    Ok(())
}
fn clear_pages(tx: &Transaction<'_>, member: i64) -> Result<()> {
    tx.execute("DELETE FROM social_pages WHERE sender IN(SELECT session FROM social_presence WHERE member=?1) OR recipient IN(SELECT session FROM social_presence WHERE member=?1)",[member])?;
    Ok(())
}
impl Board {
    /// Presentation discovery before a visit lease is allocated. Never grants opt-in.
    pub fn social_session_config(&mut self, session: &Session) -> Result<SocialConfig> {
        let tx = self.db.transaction()?;
        current(&tx, &self.identity, session, true, false)?;
        config(&tx)
    }
    pub fn social_advertisement(&mut self, l: &ChatLease) -> Result<Option<SocialPerson>> {
        let tx = self.db.transaction()?;
        chat::lease_current(&tx, &self.identity, l)?;
        enabled(&tx)?;
        Ok(person(&tx, l.id.as_slice(), now(&tx)?, false)?.map(|(_, p)| p))
    }
    pub fn social_config(&mut self, l: &ChatLease) -> Result<SocialConfig> {
        let tx = self.db.transaction()?;
        chat::lease_current(&tx, &self.identity, l)?;
        config(&tx)
    }
    pub fn social_configure(&mut self, l: &ChatLease, c: &SocialConfig) -> Result<()> {
        ascii(&c.name, 24, false)?;
        let tx = self
            .db
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let m = chat::lease_current(&tx, &self.identity, l)?;
        if m.role != Role::Sysop {
            return Err(Error::Denied);
        }
        tx.execute(
            "UPDATE social_settings SET enabled=?1,name=?2 WHERE singleton=1",
            params![c.enabled, c.name],
        )?;
        if !c.enabled {
            tx.execute("DELETE FROM social_presence", [])?;
        }
        tx.commit()?;
        Ok(())
    }
    pub fn social_availability(&mut self, l: &ChatLease, mode: &str, status: &str) -> Result<()> {
        if !["available", "busy", "away", "hidden"].contains(&mode) {
            return Err(Error::Invalid("available, busy, away or hidden required"));
        }
        ascii(status, 48, true)?;
        let tx = self
            .db
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        chat::lease_current(&tx, &self.identity, l)?;
        enabled(&tx)?;
        cleanup(&tx, now(&tx)?)?;
        // One account has one advertised chat entry. A new opt-in replaces the old one.
        tx.execute(
            "DELETE FROM social_presence WHERE member=?1",
            [l.session.member],
        )?;
        if mode != "hidden" {
            tx.execute(
                "INSERT INTO social_presence VALUES(?1,?2,?3,?4)",
                params![l.id.as_slice(), l.session.member, mode, status],
            )?;
        }
        tx.commit()?;
        Ok(())
    }
    pub fn social_prefs(&mut self, l: &ChatLease) -> Result<SocialPrefs> {
        let tx = self.db.transaction()?;
        chat::lease_current(&tx, &self.identity, l)?;
        enabled(&tx)?;
        prefs(&tx, l.session.member)
    }
    pub fn social_preferences(
        &mut self,
        l: &ChatLease,
        muted: bool,
        quiet: Option<(u16, u16)>,
    ) -> Result<()> {
        if quiet.is_some_and(|(a, b)| a >= 1440 || b >= 1440 || a == b) {
            return Err(Error::Invalid("distinct UTC quiet minutes 0-1439 required"));
        }
        let tx = self
            .db
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        chat::lease_current(&tx, &self.identity, l)?;
        enabled(&tx)?;
        clear_pages(&tx, l.session.member)?;
        tx.execute("INSERT INTO social_prefs(member,muted,quiet_start,quiet_end) VALUES(?1,?2,?3,?4) ON CONFLICT(member) DO UPDATE SET muted=excluded.muted,quiet_start=excluded.quiet_start,quiet_end=excluded.quiet_end",params![l.session.member,muted,quiet.map(|v|v.0),quiet.map(|v|v.1)])?;
        tx.commit()?;
        Ok(())
    }
    pub fn social_block(&mut self, l: &ChatLease, name: &str, block: bool) -> Result<()> {
        let name = handle(name)?;
        let tx = self
            .db
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        chat::lease_current(&tx, &self.identity, l)?;
        enabled(&tx)?;
        // Store the normalized name without resolving its existence/status.
        if block {
            if !tx
                .prepare("SELECT 1 FROM social_blocks WHERE member=?1 AND handle=?2")?
                .exists(params![l.session.member, name])?
                && tx.query_row(
                    "SELECT count(*) FROM social_blocks WHERE member=?1",
                    [l.session.member],
                    |r| r.get::<_, i64>(0),
                )? >= 16
            {
                return Err(Error::Invalid("social block limit 16"));
            }
            tx.execute(
                "INSERT OR IGNORE INTO social_blocks VALUES(?1,?2)",
                params![l.session.member, name],
            )?;
        } else {
            tx.execute(
                "DELETE FROM social_blocks WHERE member=?1 AND handle=?2",
                params![l.session.member, name],
            )?;
        }
        // Preference changes do not revive old invitations.
        clear_pages(&tx, l.session.member)?;
        tx.commit()?;
        Ok(())
    }
    pub fn social_roster(
        &mut self,
        l: &ChatLease,
        after: Option<&str>,
        sysops: bool,
    ) -> Result<SocialRoster> {
        let after = after.map(handle).transpose()?.unwrap_or_default();
        let tx = self.db.transaction()?;
        chat::lease_current(&tx, &self.identity, l)?;
        enabled(&tx)?;
        let at = now(&tx)?;
        let ids=tx.prepare("SELECT p.session FROM social_presence p JOIN members m ON m.id=p.member WHERE m.handle>?1 ORDER BY m.handle LIMIT 256")?.query_map([after],|r|r.get::<_,Vec<u8>>(0))?.collect::<std::result::Result<Vec<_>,_>>()?;
        let mut people = vec![];
        for id in ids {
            if let Some((member, p)) = person(&tx, &id, at, sysops)?
                && (!sysops || p.sysop)
                && !blocked(&tx, l.session.member, member)?
            {
                people.push(p);
                if people.len() == 9 {
                    break;
                }
            }
        }
        let more = people.len() > 8;
        people.truncate(8);
        let next = if more {
            people.last().map(|p| p.handle.clone())
        } else {
            None
        };
        Ok(SocialRoster { people, next })
    }
    /// Generic outcome prevents unknown/hidden/blocked recipients becoming an oracle.
    pub fn social_request(
        &mut self,
        l: &ChatLease,
        name: &str,
        nonce: &str,
        sysop_only: bool,
    ) -> Result<()> {
        let name = handle(name)?;
        file_id(nonce)?;
        let tx = self
            .db
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        chat::lease_current(&tx, &self.identity, l)?;
        enabled(&tx)?;
        let at = now(&tx)?;
        cleanup(&tx, at)?;
        let prior=tx.query_row("SELECT m.handle FROM social_pages p JOIN social_presence s ON s.session=p.recipient JOIN members m ON m.id=s.member WHERE p.sender=?1 AND p.nonce=?2",params![l.id.as_slice(),nonce],|r|r.get::<_,String>(0)).optional()?;
        if let Some(prior) = prior {
            return if prior == name {
                Ok(())
            } else {
                Err(Error::Invalid("page retry differs"))
            };
        }
        let Some((sender, _)) = person(&tx, l.id.as_slice(), at, true)? else {
            return Err(Error::Invalid("opt in as available and allow pages first"));
        };
        let last = tx
            .query_row(
                "SELECT last_page FROM social_cooldowns WHERE member=?1",
                [sender],
                |r| r.get::<_, i64>(0),
            )
            .optional()?
            .unwrap_or(0);
        if last > at + 5 || (last != 0 && at.saturating_sub(last) < 30) {
            return Err(Error::Invalid(
                "page cooldown: wait 30 seconds; clock changes may delay it",
            ));
        }
        if last == 0
            && tx.query_row("SELECT count(*) FROM social_cooldowns", [], |r| {
                r.get::<_, i64>(0)
            })? >= 256
        {
            return Err(Error::Invalid("social cooldown slots full"));
        }
        tx.execute("INSERT INTO social_cooldowns(member,last_page) VALUES(?1,?2) ON CONFLICT(member) DO UPDATE SET last_page=excluded.last_page",params![sender,at])?;
        let target=tx.query_row("SELECT p.session FROM social_presence p JOIN members m ON m.id=p.member WHERE m.handle=?1",[name],|r|r.get::<_,Vec<u8>>(0)).optional()?;
        if let Some(target) = target
            && let Some((member, p)) = person(&tx, &target, at, true)?
            && member != sender
            && (!sysop_only || p.sysop)
            && !blocked(&tx, sender, member)?
            && tx.query_row("SELECT count(*) FROM social_pages", [], |r| {
                r.get::<_, i64>(0)
            })? < 128
            && tx.query_row(
                "SELECT count(*) FROM social_pages WHERE sender=?1 OR recipient=?1",
                [l.id.as_slice()],
                |r| r.get::<_, i64>(0),
            )? < 8
            && tx.query_row(
                "SELECT count(*) FROM social_pages WHERE sender=?1 OR recipient=?1",
                [&target],
                |r| r.get::<_, i64>(0),
            )? < 8
        {
            tx.execute("INSERT INTO social_pages(sender,recipient,nonce,created,state) VALUES(?1,?2,?3,?4,'pending')",params![l.id.as_slice(),target,nonce,at])?;
        }
        tx.commit()?;
        Ok(())
    }
    pub fn social_pages(&mut self, l: &ChatLease) -> Result<Vec<SocialPage>> {
        let tx = self.db.transaction()?;
        chat::lease_current(&tx, &self.identity, l)?;
        enabled(&tx)?;
        let at = now(&tx)?;
        if person(&tx, l.id.as_slice(), at, true)?.is_none() {
            return Ok(vec![]);
        }
        let rows=tx.prepare("SELECT id,sender,recipient,state FROM social_pages WHERE (sender=?1 OR recipient=?1) AND created>?2-60 AND created<=?2+5 ORDER BY id DESC LIMIT 128")?.query_map(params![l.id.as_slice(),at],|r|Ok((r.get::<_,i64>(0)?,r.get::<_,Vec<u8>>(1)?,r.get::<_,Vec<u8>>(2)?,r.get::<_,String>(3)?)))?.collect::<std::result::Result<Vec<_>,_>>()?;
        let mut out = vec![];
        for (id, a, b, state) in rows {
            if let (Some((am, from)), Some((bm, to))) =
                (person(&tx, &a, at, true)?, person(&tx, &b, at, true)?)
                && !blocked(&tx, am, bm)?
            {
                out.push(SocialPage {
                    id,
                    from: from.handle,
                    to: to.handle,
                    state,
                    outgoing: a == l.id.as_slice(),
                });
                if out.len() == 8 {
                    break;
                }
            }
        }
        Ok(out)
    }
    pub fn social_answer(&mut self, l: &ChatLease, id: i64, accept: bool) -> Result<()> {
        let allowed = self
            .social_pages(l)?
            .iter()
            .any(|p| p.id == id && !p.outgoing && p.state == "pending");
        if !allowed {
            return Err(Error::NotFound);
        }
        let tx = self
            .db
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        chat::lease_current(&tx, &self.identity, l)?;
        enabled(&tx)?;
        let at = now(&tx)?;
        cleanup(&tx, at)?;
        let sender = tx
            .query_row(
                "SELECT sender FROM social_pages WHERE id=?1 AND recipient=?2 AND state='pending'",
                params![id, l.id.as_slice()],
                |r| r.get::<_, Vec<u8>>(0),
            )
            .optional()?
            .ok_or(Error::NotFound)?;
        let Some((a, _)) = person(&tx, &sender, at, true)? else {
            return Err(Error::NotFound);
        };
        let Some((b, _)) = person(&tx, l.id.as_slice(), at, true)? else {
            return Err(Error::NotFound);
        };
        if blocked(&tx, a, b)? {
            return Err(Error::NotFound);
        }
        tx.execute(
            "UPDATE social_pages SET state=?1 WHERE id=?2",
            params![if accept { "accepted" } else { "declined" }, id],
        )?;
        tx.commit()?;
        Ok(())
    }
}
pub(crate) fn validate(db: &Connection) -> Result<()> {
    let c = config(db)?;
    ascii(&c.name, 24, false)?;
    if db.query_row("SELECT count(*) FROM social_settings", [], |r| {
        r.get::<_, i64>(0)
    })? != 1
        || db.query_row("SELECT count(*) FROM social_presence", [], |r| {
            r.get::<_, i64>(0)
        })? > 256
        || db.query_row("SELECT count(*) FROM social_cooldowns", [], |r| {
            r.get::<_, i64>(0)
        })? > 256
        || db.query_row("SELECT count(*) FROM social_pages", [], |r| {
            r.get::<_, i64>(0)
        })? > 128
        || db
            .prepare("SELECT 1 FROM social_blocks GROUP BY member HAVING count(*)>16 LIMIT 1")?
            .exists([])?
    {
        return Err(Error::Invalid("social snapshot quota"));
    }
    let mut q = db.prepare("SELECT handle FROM social_blocks")?;
    let mut rows = q.query([])?;
    while let Some(r) = rows.next()? {
        let s: String = r.get(0)?;
        if handle(&s)? != s {
            return Err(Error::Invalid("social block handle"));
        }
    }
    let mut q = db.prepare("SELECT status FROM social_presence")?;
    let mut rows = q.query([])?;
    while let Some(r) = rows.next()? {
        ascii(&r.get::<_, String>(0)?, 48, true)?;
    }
    let mut q = db.prepare("SELECT nonce FROM social_pages")?;
    let mut rows = q.query([])?;
    while let Some(r) = rows.next()? {
        file_id(&r.get::<_, String>(0)?)?;
    }
    if db.prepare("SELECT 1 FROM social_presence p JOIN chat_sessions c ON c.id=p.session WHERE p.member<>c.member LIMIT 1")?.exists([])? {return Err(Error::Invalid("social lease owner mismatch"))}
    Ok(())
}

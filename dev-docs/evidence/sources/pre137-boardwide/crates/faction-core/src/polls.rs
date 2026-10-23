//! Bounded local single-choice polls; ordinary plaintext ballots, no voter lists.
use super::*;
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PollConfig {
    pub enabled: bool,
    pub name: String,
    pub compact: bool,
    pub member_creators: bool,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PollResults {
    Always,
    AfterVote,
    AfterClose,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PollState {
    Open,
    Closed,
    Cancelled,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PollDraft {
    pub question: String,
    pub description: String,
    pub choices: Vec<String>,
    pub results: PollResults,
}
#[derive(Debug, Clone)]
pub struct Poll {
    pub id: i64,
    pub creator: String,
    pub revision: String,
    pub draft: PollDraft,
    pub state: PollState,
    pub hidden: bool,
    pub mine: bool,
    pub creator_approved: bool,
    pub ever_voted: bool,
    /// Zero means no ballot; choices are numbered 1..=8.
    pub own_vote: u8,
    /// Aggregate counts only when the selected result policy permits them.
    pub counts: Option<Vec<u16>>,
}
impl PollResults {
    fn number(self) -> u8 {
        match self {
            Self::Always => 0,
            Self::AfterVote => 1,
            Self::AfterClose => 2,
        }
    }
    fn parse(n: u8) -> Result<Self> {
        match n {
            0 => Ok(Self::Always),
            1 => Ok(Self::AfterVote),
            2 => Ok(Self::AfterClose),
            _ => Err(Error::Invalid("invalid poll result policy")),
        }
    }
}
impl PollState {
    fn number(self) -> u8 {
        match self {
            Self::Open => 0,
            Self::Closed => 1,
            Self::Cancelled => 2,
        }
    }
    fn parse(n: u8) -> Result<Self> {
        match n {
            0 => Ok(Self::Open),
            1 => Ok(Self::Closed),
            2 => Ok(Self::Cancelled),
            _ => Err(Error::Invalid("invalid poll state")),
        }
    }
}
fn ascii(s: &str, bytes: usize, lines: usize, empty: bool) -> Result<()> {
    if s.len() > bytes
        || (!empty && s.trim().is_empty())
        || s.split('\n').count() > lines
        || s.split('\n')
            .any(|l| l.len() > if lines == 1 { bytes } else { 72 })
        || !s.bytes().all(|b| b == b'\n' || (32..=126).contains(&b))
    {
        return Err(Error::Invalid(
            "poll text exceeds printable ASCII/line bounds",
        ));
    }
    Ok(())
}
fn key(s: &str) -> Result<()> {
    if s.len() != 32
        || !s
            .bytes()
            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
    {
        return Err(Error::Invalid("invalid poll revision/request ID"));
    }
    Ok(())
}
impl PollConfig {
    pub fn validate(&self) -> Result<()> {
        ascii(&self.name, 24, 1, false)
    }
}
impl PollDraft {
    pub fn validate(&self) -> Result<()> {
        ascii(&self.question, 96, 1, false)?;
        ascii(&self.description, 240, 4, true)?;
        if !(2..=8).contains(&self.choices.len()) {
            return Err(Error::Invalid("poll needs 2-8 choices"));
        }
        let mut labels = std::collections::HashSet::new();
        for label in &self.choices {
            ascii(label, 48, 1, false)?;
            if !labels.insert(label.trim().to_ascii_lowercase()) {
                return Err(Error::Invalid("poll choices must be distinct"));
            }
        }
        Ok(())
    }
}
fn policy(tx: &Connection) -> Result<PollConfig> {
    Ok(tx.query_row(
        "SELECT enabled,name,compact,member_creators FROM poll_settings WHERE singleton=1",
        [],
        |r| {
            Ok(PollConfig {
                enabled: r.get(0)?,
                name: r.get(1)?,
                compact: r.get(2)?,
                member_creators: r.get(3)?,
            })
        },
    )?)
}
fn enabled(tx: &Connection) -> Result<PollConfig> {
    let c = policy(tx)?;
    if !c.enabled {
        return Err(Error::Invalid("polls are disabled"));
    }
    Ok(c)
}
fn choices(tx: &Connection, id: i64) -> Result<Vec<String>> {
    Ok(tx
        .prepare("SELECT label FROM poll_choices WHERE poll=?1 ORDER BY number")?
        .query_map([id], |r| r.get(0))?
        .collect::<std::result::Result<Vec<_>, _>>()?)
}
fn row(tx: &Connection, s: &Session, m: &Member, id: i64) -> Result<Poll> {
    let (creator,revision,question,description,results,state,hidden,mine,creator_approved,ever_voted):(String,String,String,String,u8,u8,bool,bool,bool,bool)=tx.query_row("SELECT m.handle,p.revision,p.question,p.description,p.results,p.state,p.hidden,p.creator=?2,m.status='approved',p.ever_voted FROM polls p JOIN members m ON m.id=p.creator WHERE p.id=?1 AND (?3=1 OR (m.status='approved' AND (p.hidden=0 OR p.creator=?2)))",params![id,s.member,m.role==Role::Sysop],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?,r.get(5)?,r.get(6)?,r.get(7)?,r.get(8)?,r.get(9)?))).optional()?.ok_or(Error::NotFound)?;
    let own_vote = tx
        .query_row(
            "SELECT choice FROM poll_ballots WHERE poll=?1 AND member=?2",
            params![id, s.member],
            |r| r.get::<_, u8>(0),
        )
        .optional()?
        .unwrap_or(0);
    let results = PollResults::parse(results)?;
    let state = PollState::parse(state)?;
    let visible = state != PollState::Cancelled
        && !hidden
        && creator_approved
        && match results {
            PollResults::Always => true,
            PollResults::AfterVote => own_vote > 0,
            PollResults::AfterClose => state == PollState::Closed,
        };
    let counts = if visible {
        Some(tx.prepare("SELECT coalesce(sum(m.status='approved'),0) FROM poll_choices c LEFT JOIN poll_ballots b ON b.poll=c.poll AND b.choice=c.number LEFT JOIN members m ON m.id=b.member WHERE c.poll=?1 GROUP BY c.number ORDER BY c.number")?.query_map([id],|r|r.get(0))?.collect::<std::result::Result<Vec<u16>,_>>()?)
    } else {
        None
    };
    Ok(Poll {
        id,
        creator,
        revision,
        draft: PollDraft {
            question,
            description,
            choices: choices(tx, id)?,
            results,
        },
        state,
        hidden,
        mine,
        creator_approved,
        ever_voted,
        own_vote,
        counts,
    })
}
impl Board {
    pub fn poll_config(&mut self, s: &Session) -> Result<PollConfig> {
        let tx = self.db.transaction()?;
        current(&tx, &self.identity, s, true, false)?;
        policy(&tx)
    }
    pub fn configure_polls(&mut self, s: &Session, c: &PollConfig) -> Result<()> {
        c.validate()?;
        let tx = self
            .db
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        current(&tx, &self.identity, s, true, true)?;
        tx.execute(
            "UPDATE poll_settings SET enabled=?1,name=?2,compact=?3,member_creators=?4",
            params![c.enabled, c.name, c.compact, c.member_creators],
        )?;
        tx.commit()?;
        Ok(())
    }
    pub fn polls(&mut self, s: &Session, finished: bool, offset: usize) -> Result<Vec<Poll>> {
        if offset > 32 {
            return Err(Error::Invalid("poll page outside bounds"));
        }
        let tx = self.db.transaction()?;
        let m = current(&tx, &self.identity, s, true, false)?;
        enabled(&tx)?;
        let ids=tx.prepare("SELECT p.id FROM polls p JOIN members m ON m.id=p.creator WHERE (?1=1 OR (m.status='approved' AND (p.hidden=0 OR p.creator=?2))) AND ((?3=0 AND state=0) OR (?3=1 AND state<>0)) ORDER BY p.id DESC LIMIT 4 OFFSET ?4")?.query_map(params![m.role==Role::Sysop,s.member,finished,offset as i64],|r|r.get::<_,i64>(0))?.collect::<std::result::Result<Vec<_>,_>>()?;
        ids.into_iter().map(|id| row(&tx, s, &m, id)).collect()
    }
    pub fn poll(&mut self, s: &Session, id: i64) -> Result<Poll> {
        let tx = self.db.transaction()?;
        let m = current(&tx, &self.identity, s, true, false)?;
        enabled(&tx)?;
        row(&tx, s, &m, id)
    }
    pub fn save_poll(
        &mut self,
        s: &Session,
        id: Option<i64>,
        expected: &str,
        d: &PollDraft,
    ) -> Result<i64> {
        key(expected)?;
        d.validate()?;
        let tx = self
            .db
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let m = current(&tx, &self.identity, s, true, false)?;
        let c = enabled(&tx)?;
        if m.role != Role::Sysop && !c.member_creators {
            return Err(Error::Denied);
        }
        let id = if let Some(id) = id {
            let old = row(&tx, s, &m, id)?;
            if !old.mine {
                return Err(Error::Denied);
            }
            if old.revision != expected {
                return Err(Error::RequestConflict);
            }
            if old.state != PollState::Open || old.ever_voted {
                return Err(Error::Invalid(
                    "wording locks after the first ballot or closing",
                ));
            }
            tx.execute("DELETE FROM poll_choices WHERE poll=?1", [id])?;
            tx.execute("UPDATE polls SET question=?1,description=?2,results=?3,revision=?4,changed_at=unixepoch() WHERE id=?5",params![d.question,d.description,d.results.number(),request_id()?,id])?;
            id
        } else {
            if let Some(id) = tx
                .query_row(
                    "SELECT id FROM polls WHERE creator=?1 AND request_id=?2",
                    params![s.member, expected],
                    |r| r.get::<_, i64>(0),
                )
                .optional()?
            {
                if row(&tx, s, &m, id)?.draft != *d {
                    return Err(Error::RequestConflict);
                }
                return Ok(id);
            }
            if tx.query_row("SELECT count(*) FROM polls", [], |r| r.get::<_, i64>(0))? >= 32 {
                return Err(Error::Invalid(
                    "32 retained polls; ask a sysop to remove old polls",
                ));
            }
            if m.role != Role::Sysop
                && tx.query_row(
                    "SELECT count(*) FROM polls WHERE creator=?1 AND state=0",
                    [s.member],
                    |r| r.get::<_, i64>(0),
                )? >= 4
            {
                return Err(Error::Invalid("four open polls per member creator"));
            }
            tx.execute("INSERT INTO polls(creator,request_id,revision,question,description,results,state,hidden,ever_voted,changed_at) VALUES(?1,?2,?2,?3,?4,?5,0,0,0,unixepoch())",params![s.member,expected,d.question,d.description,d.results.number()])?;
            tx.last_insert_rowid()
        };
        for (n, label) in d.choices.iter().enumerate() {
            tx.execute(
                "INSERT INTO poll_choices VALUES(?1,?2,?3)",
                params![id, (n + 1) as u8, label],
            )?;
        }
        tx.commit()?;
        Ok(id)
    }
    pub fn vote_poll(&mut self, s: &Session, id: i64, expected: &str, choice: u8) -> Result<()> {
        let tx = self
            .db
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let m = current(&tx, &self.identity, s, true, false)?;
        enabled(&tx)?;
        let p = row(&tx, s, &m, id)?;
        if p.revision != expected {
            return Err(Error::RequestConflict);
        }
        if p.state != PollState::Open || p.hidden || !p.creator_approved {
            return Err(Error::Invalid(
                "poll is closed, cancelled, hidden or inactive",
            ));
        }
        if usize::from(choice) > p.draft.choices.len() {
            return Err(Error::Invalid(
                "choose a displayed choice or zero to withdraw",
            ));
        }
        if choice == 0 {
            tx.execute(
                "DELETE FROM poll_ballots WHERE poll=?1 AND member=?2",
                params![id, s.member],
            )?;
        } else {
            if p.own_vote == 0
                && tx.query_row(
                    "SELECT count(*) FROM poll_ballots WHERE poll=?1",
                    [id],
                    |r| r.get::<_, i64>(0),
                )? >= 250
            {
                return Err(Error::Invalid("250 retained ballots per poll"));
            }
            tx.execute("INSERT INTO poll_ballots VALUES(?1,?2,?3) ON CONFLICT(poll,member) DO UPDATE SET choice=excluded.choice",params![id,s.member,choice])?;
            tx.execute("UPDATE polls SET ever_voted=1 WHERE id=?1", [id])?;
        }
        tx.commit()?;
        Ok(())
    }
    pub fn set_poll_state(
        &mut self,
        s: &Session,
        id: i64,
        expected: &str,
        state: PollState,
    ) -> Result<()> {
        let tx = self
            .db
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let m = current(&tx, &self.identity, s, true, false)?;
        enabled(&tx)?;
        let p = row(&tx, s, &m, id)?;
        if !p.mine {
            return Err(Error::Denied);
        }
        if p.revision != expected {
            return Err(Error::RequestConflict);
        }
        if p.state != PollState::Open || state == PollState::Open {
            return Err(Error::Invalid("closing/cancelling is final"));
        }
        tx.execute(
            "UPDATE polls SET state=?1,revision=?2,changed_at=unixepoch() WHERE id=?3",
            params![state.number(), request_id()?, id],
        )?;
        tx.commit()?;
        Ok(())
    }
    pub fn moderate_poll(
        &mut self,
        s: &Session,
        id: i64,
        expected: &str,
        action: &str,
    ) -> Result<()> {
        let tx = self
            .db
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let m = current(&tx, &self.identity, s, true, true)?;
        enabled(&tx)?;
        let p = row(&tx, s, &m, id)?;
        if p.revision != expected {
            return Err(Error::RequestConflict);
        }
        match action {
            "hide" | "show" => {
                tx.execute(
                    "UPDATE polls SET hidden=?1,revision=?2,changed_at=unixepoch() WHERE id=?3",
                    params![action == "hide", request_id()?, id],
                )?;
            }
            "remove" => {
                tx.execute("DELETE FROM polls WHERE id=?1", [id])?;
            }
            _ => return Err(Error::Invalid("use hide, show or remove")),
        };
        tx.commit()?;
        Ok(())
    }
}
pub(crate) fn validate(db: &Connection, deadline: std::time::Instant) -> Result<()> {
    policy(db)?.validate()?;
    if db.query_row("SELECT count(*) FROM poll_settings", [], |r| {
        r.get::<_, i64>(0)
    })? != 1
        || db.query_row("SELECT count(*) FROM polls", [], |r| r.get::<_, i64>(0))? > 32
    {
        return Err(Error::Invalid("snapshot poll policy/quota"));
    }
    let mut q = db.prepare(
        "SELECT id,request_id,revision,question,description,results,ever_voted FROM polls",
    )?;
    let mut rows = q.query([])?;
    while let Some(r) = rows.next()? {
        if std::time::Instant::now() >= deadline {
            return Err(Error::Invalid("snapshot poll budget"));
        }
        let id: i64 = r.get(0)?;
        key(&r.get::<_, String>(1)?)?;
        key(&r.get::<_, String>(2)?)?;
        PollDraft {
            question: r.get(3)?,
            description: r.get(4)?,
            choices: choices(db, id)?,
            results: PollResults::parse(r.get(5)?)?,
        }
        .validate()?;
        let (count, max): (i64, i64) = db.query_row(
            "SELECT count(*),max(number) FROM poll_choices WHERE poll=?1",
            [id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )?;
        if count != max {
            return Err(Error::Invalid("snapshot poll choice order"));
        }
        let ballots: i64 = db.query_row(
            "SELECT count(*) FROM poll_ballots WHERE poll=?1",
            [id],
            |r| r.get(0),
        )?;
        if ballots > 250 || (ballots > 0 && !r.get::<_, bool>(6)?) {
            return Err(Error::Invalid("snapshot ballot quota/wording lock"));
        }
    }
    Ok(())
}

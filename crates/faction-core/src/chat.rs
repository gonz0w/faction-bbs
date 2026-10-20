//! BBS-local bounded chat. No transport, federation or per-recipient queue.
use super::*;
pub const CHAT_BODY_BYTES: usize = 512;
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatChannel {
    pub id: i64,
    pub name: String,
    pub private: bool,
    pub sequence: i64,
}
pub struct ChatLease {
    session: Session,
    id: [u8; 16],
}
impl fmt::Debug for ChatLease {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ChatLease { .. }")
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChatKind {
    Say,
    Emote,
}
impl ChatKind {
    fn sql(self) -> &'static str {
        match self {
            Self::Say => "say",
            Self::Emote => "emote",
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatMessage {
    pub sequence: i64,
    pub author: String,
    pub body: String,
    pub kind: ChatKind,
}
#[derive(Debug)]
pub struct ChatPage {
    pub messages: Vec<ChatMessage>,
    pub cursor: i64,
    pub gap: bool,
    pub more: bool,
}
#[derive(Debug)]
pub struct ChatNames {
    pub handles: Vec<String>,
    pub next: Option<String>,
}
fn channel_name(value: &str) -> Result<String> {
    if !(2..=32).contains(&value.len())
        || !value.starts_with('#')
        || !value[1..]
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
    {
        return Err(Error::Invalid(
            "channel needs # plus 1-31 ASCII letters, digits, underscores or hyphens",
        ));
    }
    Ok(value.to_ascii_lowercase())
}
fn lease_current(tx: &Transaction<'_>, identity: &[u8; 32], lease: &ChatLease) -> Result<Member> {
    let m = current(tx, identity, &lease.session, true, false)?;
    let valid:bool=tx.query_row("SELECT EXISTS(SELECT 1 FROM chat_sessions WHERE id=?1 AND member=?2 AND epoch=?3 AND touched>unixepoch()-60 AND touched<=unixepoch()+5)",params![lease.id.as_slice(),lease.session.member,lease.session.epoch],|r|r.get(0))?;
    if !valid {
        return Err(Error::StaleSession);
    }
    Ok(m)
}
fn channel(tx: &Transaction<'_>, name: &str, m: &Member, member: i64) -> Result<ChatChannel> {
    let name = channel_name(name)?;
    tx.query_row("SELECT id,name,private,sequence FROM chat_channels c WHERE name=?1 AND (private=0 OR ?2 OR EXISTS(SELECT 1 FROM chat_grants g WHERE g.channel=c.id AND g.member=?3))",params![name,m.role==Role::Sysop,member],|r|Ok(ChatChannel{id:r.get(0)?,name:r.get(1)?,private:r.get(2)?,sequence:r.get(3)?})).optional()?.ok_or(Error::Denied)
}
fn joined(
    tx: &Transaction<'_>,
    identity: &[u8; 32],
    lease: &ChatLease,
    name: &str,
) -> Result<(Member, ChatChannel, i64)> {
    let m = lease_current(tx, identity, lease)?;
    let c = channel(tx, name, &m, lease.session.member)?;
    let floor = tx
        .query_row(
            "SELECT floor FROM chat_joins WHERE session=?1 AND channel=?2",
            params![lease.id.as_slice(), c.id],
            |r| r.get(0),
        )
        .optional()?
        .ok_or(Error::Denied)?;
    Ok((m, c, floor))
}
fn chat_kind(value: &str) -> Result<ChatKind> {
    match value {
        "say" => Ok(ChatKind::Say),
        "emote" => Ok(ChatKind::Emote),
        _ => Err(Error::UnsupportedSchema),
    }
}
impl Board {
    pub fn chat_open(&mut self, s: &Session) -> Result<ChatLease> {
        let mut id = [0; 16];
        getrandom::fill(&mut id).map_err(|_| Error::Entropy)?;
        let tx = self
            .db
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        current(&tx, &self.identity, s, true, false)?;
        tx.execute("DELETE FROM chat_sessions WHERE touched<=unixepoch()-60 OR touched>unixepoch()+5 OR NOT EXISTS(SELECT 1 FROM members m WHERE m.id=chat_sessions.member AND m.epoch=chat_sessions.epoch AND m.status='approved')",[])?;
        let count: i64 = tx.query_row("SELECT count(*) FROM chat_sessions", [], |r| r.get(0))?;
        if count >= 256 {
            return Err(Error::Invalid("live chat lease limit reached"));
        }
        tx.execute(
            "INSERT INTO chat_sessions(id,member,epoch,touched) VALUES(?1,?2,?3,unixepoch())",
            params![id.as_slice(), s.member, s.epoch],
        )?;
        tx.commit()?;
        Ok(ChatLease {
            session: Session {
                board: s.board,
                member: s.member,
                epoch: s.epoch,
            },
            id,
        })
    }
    /// Cleanup is permitted after approval/lease expiry; it returns no content.
    pub fn chat_close(&mut self, lease: &ChatLease) -> Result<()> {
        if lease.session.board != self.identity {
            return Err(Error::Denied);
        }
        self.db.execute(
            "DELETE FROM chat_sessions WHERE id=?1 AND member=?2 AND epoch=?3",
            params![
                lease.id.as_slice(),
                lease.session.member,
                lease.session.epoch
            ],
        )?;
        Ok(())
    }
    pub fn chat_heartbeat(&mut self, lease: &ChatLease) -> Result<()> {
        let tx = self
            .db
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        lease_current(&tx, &self.identity, lease)?;
        tx.execute(
            "UPDATE chat_sessions SET touched=unixepoch() WHERE id=?1",
            [lease.id.as_slice()],
        )?;
        tx.commit()?;
        Ok(())
    }
    pub fn chat_channels(&mut self, lease: &ChatLease) -> Result<Vec<ChatChannel>> {
        let tx = self.db.transaction()?;
        let m = lease_current(&tx, &self.identity, lease)?;
        let mut q=tx.prepare("SELECT id,name,private,sequence FROM chat_channels c WHERE private=0 OR ?1 OR EXISTS(SELECT 1 FROM chat_grants g WHERE g.channel=c.id AND g.member=?2) ORDER BY id LIMIT 32")?;
        Ok(
            q.query_map(params![m.role == Role::Sysop, lease.session.member], |r| {
                Ok(ChatChannel {
                    id: r.get(0)?,
                    name: r.get(1)?,
                    private: r.get(2)?,
                    sequence: r.get(3)?,
                })
            })?
            .collect::<rusqlite::Result<_>>()?,
        )
    }
    pub fn chat_create(&mut self, s: &Session, name: &str, private: bool) -> Result<ChatChannel> {
        let name = channel_name(name)?;
        let tx = self
            .db
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        current(&tx, &self.identity, s, true, true)?;
        let count: i64 = tx.query_row("SELECT count(*) FROM chat_channels", [], |r| r.get(0))?;
        if count >= 32 {
            return Err(Error::Invalid("channel limit reached"));
        }
        tx.execute(
            "INSERT INTO chat_channels(name,private) VALUES(?1,?2)",
            params![name, private],
        )?;
        let id = tx.last_insert_rowid();
        tx.commit()?;
        Ok(ChatChannel {
            id,
            name,
            private,
            sequence: 0,
        })
    }
    pub fn chat_allow(&mut self, s: &Session, name: &str, who: &str, allow: bool) -> Result<()> {
        self.chat_moderate(s, name, who, Some(allow))
    }
    pub fn chat_kick(&mut self, s: &Session, name: &str, who: &str) -> Result<()> {
        self.chat_moderate(s, name, who, None)
    }
    fn chat_moderate(
        &mut self,
        s: &Session,
        name: &str,
        who: &str,
        allow: Option<bool>,
    ) -> Result<()> {
        let who = handle(who)?;
        let tx = self
            .db
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let m = current(&tx, &self.identity, s, true, true)?;
        let c = channel(&tx, name, &m, s.member)?;
        let target: Option<(i64, String, String)> = tx
            .query_row(
                "SELECT id,status,role FROM members WHERE handle=?1",
                [who],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .optional()?;
        let (id, status, role) = target.ok_or(Error::NotFound)?;
        if role == "sysop" {
            return Err(Error::Invalid("sysop access is operator authority"));
        }
        if let Some(allow) = allow {
            if !c.private {
                return Err(Error::Invalid("allow/deny applies to private channels"));
            }
            if allow {
                if status != "approved" {
                    return Err(Error::Invalid(
                        "approve the account before granting a channel",
                    ));
                }
                tx.execute(
                    "INSERT OR IGNORE INTO chat_grants VALUES(?1,?2)",
                    params![c.id, id],
                )?;
            } else {
                tx.execute(
                    "DELETE FROM chat_grants WHERE channel=?1 AND member=?2",
                    params![c.id, id],
                )?;
            }
        }
        // Every grant change/kick requires a new join/history floor, even regrant.
        tx.execute("DELETE FROM chat_joins WHERE channel=?1 AND session IN (SELECT id FROM chat_sessions WHERE member=?2)",params![c.id,id])?;
        tx.commit()?;
        Ok(())
    }
    pub fn chat_join(&mut self, lease: &ChatLease, name: &str) -> Result<ChatChannel> {
        let tx = self
            .db
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let m = lease_current(&tx, &self.identity, lease)?;
        let c = channel(&tx, name, &m, lease.session.member)?;
        let exists: bool = tx.query_row(
            "SELECT EXISTS(SELECT 1 FROM chat_joins WHERE session=?1 AND channel=?2)",
            params![lease.id.as_slice(), c.id],
            |r| r.get(0),
        )?;
        if !exists {
            let count: i64 = tx.query_row(
                "SELECT count(*) FROM chat_joins WHERE session=?1",
                [lease.id.as_slice()],
                |r| r.get(0),
            )?;
            if count >= 8 {
                return Err(Error::Invalid("joined-channel limit reached"));
            }
            tx.execute(
                "INSERT INTO chat_joins VALUES(?1,?2,?3)",
                params![lease.id.as_slice(), c.id, c.sequence],
            )?;
        }
        tx.commit()?;
        Ok(c)
    }
    pub fn chat_part(&mut self, lease: &ChatLease, name: &str) -> Result<()> {
        let tx = self
            .db
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let (_, c, _) = joined(&tx, &self.identity, lease, name)?;
        tx.execute(
            "DELETE FROM chat_joins WHERE session=?1 AND channel=?2",
            params![lease.id.as_slice(), c.id],
        )?;
        tx.commit()?;
        Ok(())
    }
    pub fn chat_send(
        &mut self,
        lease: &ChatLease,
        name: &str,
        request: i64,
        body: &str,
        kind: ChatKind,
    ) -> Result<ChatMessage> {
        text(body, CHAT_BODY_BYTES, false)?;
        if request < 1 {
            return Err(Error::Invalid("positive chat request sequence required"));
        }
        let tx = self
            .db
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let (m, c, floor) = joined(&tx, &self.identity, lease, name)?;
        let previous: i64 = tx.query_row(
            "SELECT last_request FROM chat_sessions WHERE id=?1",
            [lease.id.as_slice()],
            |r| r.get(0),
        )?;
        if request == previous {
            let (target,sequence,old_floor,old_body,old_kind):(i64,i64,i64,String,String)=tx.query_row("SELECT last_channel,last_sequence,last_floor,last_body,last_kind FROM chat_sessions WHERE id=?1",[lease.id.as_slice()],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?)))?;
            if target != c.id || old_floor != floor || old_body != body || old_kind != kind.sql() {
                return Err(Error::RequestConflict);
            }
            tx.commit()?;
            return Ok(ChatMessage {
                sequence,
                author: m.handle,
                body: body.to_owned(),
                kind,
            });
        }
        if previous.checked_add(1) != Some(request) {
            return Err(Error::RequestConflict);
        }
        let sequence = c
            .sequence
            .checked_add(1)
            .ok_or(Error::Invalid("channel sequence exhausted"))?;
        tx.execute(
            "UPDATE chat_channels SET sequence=?1 WHERE id=?2",
            params![sequence, c.id],
        )?;
        tx.execute(
            "INSERT INTO chat_messages VALUES(?1,?2,?3,?4,?5)",
            params![c.id, sequence, lease.session.member, body, kind.sql()],
        )?;
        tx.execute("UPDATE chat_sessions SET last_request=?1,last_channel=?2,last_sequence=?3,last_floor=?4,last_body=?5,last_kind=?6,touched=unixepoch() WHERE id=?7",params![request,c.id,sequence,floor,body,kind.sql(),lease.id.as_slice()])?;
        tx.execute(
            "DELETE FROM chat_messages WHERE channel=?1 AND sequence<=?2",
            params![c.id, sequence.saturating_sub(64)],
        )?;
        tx.commit()?;
        Ok(ChatMessage {
            sequence,
            author: m.handle,
            body: body.to_owned(),
            kind,
        })
    }
    pub fn chat_read(&mut self, lease: &ChatLease, name: &str, after: i64) -> Result<ChatPage> {
        if after < 0 {
            return Err(Error::Invalid("negative chat cursor"));
        }
        let tx = self.db.transaction()?;
        let (_, c, floor) = joined(&tx, &self.identity, lease, name)?;
        if after > c.sequence {
            return Err(Error::Invalid("chat cursor is ahead of channel"));
        }
        let start = after.max(floor);
        let retained = c.sequence.saturating_sub(64);
        let gap = start < retained;
        let mut cursor = start.max(retained);
        let mut q=tx.prepare("SELECT p.sequence,m.handle,p.body,p.kind FROM chat_messages p JOIN members m ON m.id=p.author WHERE p.channel=?1 AND p.sequence>?2 ORDER BY p.sequence LIMIT 16")?;
        let mut messages = Vec::new();
        let mut charged = 0;
        for row in q.query_map(params![c.id, cursor], |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, String>(3)?,
            ))
        })? {
            let (sequence, author, body, kind) = row?;
            let cost = author.len() + body.len() + 96;
            if charged + cost > PAGE_BYTES {
                break;
            }
            charged += cost;
            cursor = sequence;
            messages.push(ChatMessage {
                sequence,
                author,
                body,
                kind: chat_kind(&kind)?,
            });
        }
        Ok(ChatPage {
            messages,
            cursor,
            gap,
            more: cursor < c.sequence,
        })
    }
    pub fn chat_names(
        &mut self,
        lease: &ChatLease,
        name: &str,
        after: Option<&str>,
    ) -> Result<ChatNames> {
        let after = after.map(handle).transpose()?.unwrap_or_default();
        let tx = self.db.transaction()?;
        let (_, c, _) = joined(&tx, &self.identity, lease, name)?;
        let mut q=tx.prepare("SELECT DISTINCT m.handle FROM chat_joins j JOIN chat_sessions s ON s.id=j.session JOIN members m ON m.id=s.member WHERE j.channel=?1 AND s.touched>unixepoch()-60 AND s.touched<=unixepoch()+5 AND s.epoch=m.epoch AND m.status='approved' AND m.handle>?2 AND (?3=0 OR m.role='sysop' OR EXISTS(SELECT 1 FROM chat_grants g WHERE g.channel=?1 AND g.member=m.id)) ORDER BY m.handle LIMIT 33")?;
        let mut handles = q
            .query_map(params![c.id, after, c.private], |r| r.get(0))?
            .collect::<rusqlite::Result<Vec<String>>>()?;
        let next = if handles.len() > 32 {
            handles.truncate(32);
            handles.last().cloned()
        } else {
            None
        };
        Ok(ChatNames { handles, next })
    }
}
/// BBS slash commands only, deliberately not the IRC wire protocol.
#[derive(Debug, PartialEq, Eq)]
pub enum ChatCommand<'a> {
    Help,
    List,
    Join(&'a str),
    Part,
    Names(Option<&'a str>),
    History(i64),
    Poll,
    Say(&'a str),
    Emote(&'a str),
    Quit,
    Create(&'a str, bool),
    Allow(&'a str, &'a str, bool),
    Kick(&'a str, &'a str),
}
impl<'a> ChatCommand<'a> {
    pub fn parse(line: &'a str) -> Result<Self> {
        text(line, 768, false)?;
        if line.starts_with("//") {
            return Ok(Self::Say(&line[1..]));
        }
        let Some(rest) = line.strip_prefix('/') else {
            return Ok(Self::Say(line));
        };
        let (name, args) = rest.split_once(' ').unwrap_or((rest, ""));
        let args = args.trim();
        let mut words = args.split_whitespace();
        let first = words.next();
        let second = words.next();
        let extra = words.next();
        match (name.to_ascii_lowercase().as_str(), first, second, extra) {
            ("help", None, _, _) => Ok(Self::Help),
            ("list", None, _, _) => Ok(Self::List),
            ("poll", None, _, _) => Ok(Self::Poll),
            ("part", None, _, _) => Ok(Self::Part),
            ("quit", None, _, _) => Ok(Self::Quit),
            ("join", Some(c), None, _) => Ok(Self::Join(c)),
            ("names", a, None, _) => Ok(Self::Names(a)),
            ("history", None, _, _) => Ok(Self::History(0)),
            ("history", Some(n), None, _) => {
                Ok(Self::History(n.parse().map_err(|_| {
                    Error::Invalid("history needs a numeric cursor")
                })?))
            }
            ("me", Some(_), _, _) => Ok(Self::Emote(args)),
            ("create", Some(c), Some("board"), None) => Ok(Self::Create(c, false)),
            ("create", Some(c), Some("private"), None) => Ok(Self::Create(c, true)),
            ("allow", Some(c), Some(m), None) => Ok(Self::Allow(c, m, true)),
            ("deny", Some(c), Some(m), None) => Ok(Self::Allow(c, m, false)),
            ("kick", Some(c), Some(m), None) => Ok(Self::Kick(c, m)),
            _ => Err(Error::Invalid(
                "unknown chat command or wrong arguments; /help",
            )),
        }
    }
}

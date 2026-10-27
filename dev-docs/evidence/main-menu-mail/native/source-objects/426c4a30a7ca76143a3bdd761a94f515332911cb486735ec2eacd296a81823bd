//! Local bests, never a game wallet. The host retains each nonserializable run.
use crate::{Board, Error, Result, Session, current};
use rusqlite::{OptionalExtension, TransactionBehavior, params};

pub const SCORE_MAX: u32 = 1_000_000_000;
pub const SCORE_GAMES: i64 = 64;
pub const SCORE_ROWS: usize = 10;

/// Cannot be forged, cloned or imported through a terminal/game protocol.
/// Drop abandons a run; an accepted completion cannot be counted again.
#[derive(Debug)]
pub struct GameRun {
    board: [u8; 32],
    member: i64,
    epoch: i64,
    roaming: Option<crate::roaming::Provenance>,
    game: String,
    finished: bool,
}
#[derive(Debug, PartialEq, Eq)]
pub struct GameScore {
    pub handle: String,
    pub best: u32,
    pub plays: u32,
}
#[derive(Debug, PartialEq, Eq)]
pub struct ScoreUpdate {
    pub best: u32,
    pub plays: u32,
    pub improved: bool,
}
pub(crate) fn game_slug(game: &str) -> Result<()> {
    if game.is_empty()
        || game.len() > 32
        || !game
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
    {
        return Err(Error::Invalid("invalid game key"));
    }
    Ok(())
}
impl Board {
    pub fn game_start(&mut self, session: &Session, game: &str) -> Result<GameRun> {
        game_slug(game)?;
        let tx = self.db.transaction()?;
        current(&tx, &self.identity, session, true, false)?;
        Ok(GameRun {
            board: self.identity,
            member: session.member,
            epoch: session.epoch,
            roaming: session.roaming,
            game: game.into(),
            finished: false,
        })
    }
    /// Score authority is only for trusted host-installed game results. Never
    /// expose this method as a member-selected score/member/game RPC.
    pub fn game_finish(
        &mut self,
        session: &Session,
        run: &mut GameRun,
        score: u32,
    ) -> Result<ScoreUpdate> {
        let tx = self
            .db
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        current(&tx, &self.identity, session, true, false)?;
        if run.board != self.identity || run.member != session.member || run.epoch != session.epoch
        {
            return Err(Error::Denied);
        }
        if let Some(p) = run.roaming {
            crate::roaming::check(&tx, run.member, p)?;
        }
        if run.finished {
            return Err(Error::RequestConflict);
        }
        if score > SCORE_MAX {
            return Err(Error::Invalid("score exceeds limit"));
        }
        let old: Option<(u32, u32)> = tx
            .query_row(
                "SELECT best,plays FROM game_scores WHERE member_id=?1 AND game=?2",
                params![run.member, run.game],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .optional()?;
        if old.is_none() {
            let count: i64 = tx.query_row(
                "SELECT count(*) FROM game_scores WHERE member_id=?1",
                [run.member],
                |r| r.get(0),
            )?;
            if count >= SCORE_GAMES {
                return Err(Error::Invalid("personal game limit reached"));
            }
        }
        let best = old.map_or(score, |(best, _)| best.max(score));
        let plays = old.map_or(1, |(_, plays)| (plays + 1).min(SCORE_MAX));
        tx.execute("INSERT INTO game_scores(member_id,game,best,plays) VALUES(?1,?2,?3,?4) ON CONFLICT(member_id,game) DO UPDATE SET best=excluded.best,plays=excluded.plays", params![run.member,run.game,best,plays])?;
        tx.commit()?;
        run.finished = true;
        Ok(ScoreUpdate {
            best,
            plays,
            improved: old.is_none_or(|(best, _)| score > best),
        })
    }
    pub fn game_scores(&mut self, session: &Session, game: &str) -> Result<Vec<GameScore>> {
        game_slug(game)?;
        let tx = self.db.transaction()?;
        current(&tx, &self.identity, session, true, false)?;
        let mut stmt = tx.prepare("SELECT m.handle,s.best,s.plays FROM game_scores s JOIN members m ON m.id=s.member_id WHERE s.game=?1 AND m.status='approved' ORDER BY s.best DESC,s.member_id LIMIT 10")?;
        Ok(stmt
            .query_map([game], |r| {
                Ok(GameScore {
                    handle: r.get(0)?,
                    best: r.get(1)?,
                    plays: r.get(2)?,
                })
            })?
            .collect::<std::result::Result<_, _>>()?)
    }
    pub fn game_personal_best(
        &mut self,
        session: &Session,
        game: &str,
    ) -> Result<Option<GameScore>> {
        game_slug(game)?;
        let tx = self.db.transaction()?;
        let member = current(&tx, &self.identity, session, true, false)?;
        Ok(tx
            .query_row(
                "SELECT best,plays FROM game_scores WHERE member_id=?1 AND game=?2",
                params![session.member, game],
                |r| {
                    Ok(GameScore {
                        handle: member.handle.clone(),
                        best: r.get(0)?,
                        plays: r.get(1)?,
                    })
                },
            )
            .optional()?)
    }
}

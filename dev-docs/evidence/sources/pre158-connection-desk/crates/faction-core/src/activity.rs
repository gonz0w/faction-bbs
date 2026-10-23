//! Current host-derived broad location, never a retained activity history.
use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VisitActivity {
    Menu,
    Rooms,
    Chat,
    Doors,
    Files,
    Lounge,
    Who,
    Settings,
    Community,
    Gallery,
    Library,
    Directory,
}
impl VisitActivity {
    pub fn label(self) -> &'static str {
        match self {
            Self::Menu => "Menu",
            Self::Rooms => "Rooms",
            Self::Chat => "Chat",
            Self::Doors => "Doors",
            Self::Files => "Files",
            Self::Lounge => "Lounge",
            Self::Who => "Who",
            Self::Settings => "Settings",
            Self::Community => "Community",
            Self::Gallery => "Gallery",
            Self::Library => "Library",
            Self::Directory => "Directory",
        }
    }
}
impl Board {
    /// Trusted adapter transition, not a member-supplied status or authority.
    pub fn visit_activity(&mut self, lease: &ChatLease, activity: VisitActivity) -> Result<()> {
        let tx = self
            .db
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        chat::lease_current(&tx, &self.identity, lease)?;
        tx.execute("INSERT INTO visit_activity(session,activity) VALUES(?1,?2) ON CONFLICT(session) DO UPDATE SET activity=excluded.activity WHERE activity<>excluded.activity",params![lease.id.as_slice(),activity.label()])?;
        tx.commit()?;
        Ok(())
    }
}

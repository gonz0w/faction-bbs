//! Opt-in aggregate publication in the existing private state directory.
use super::{Client, CredentialJobs};
use faction_core::{
    Board,
    operator::{self, Snapshot},
};
use std::{
    path::PathBuf,
    sync::atomic::Ordering,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
pub(super) struct Publisher {
    enabled: bool,
    directory: PathBuf,
    started: Instant,
    next: Instant,
}
impl Publisher {
    pub fn new(enabled: bool, directory: PathBuf) -> Self {
        Self {
            enabled,
            directory,
            started: Instant::now(),
            next: Instant::now(),
        }
    }
    pub fn tick(
        &mut self,
        board: &Board,
        clients: &[Client],
        jobs: &CredentialJobs,
        capacity: usize,
    ) {
        if !self.enabled || Instant::now() < self.next {
            return;
        }
        self.next = Instant::now() + Duration::from_secs(5);
        let snapshot = Snapshot {
            sampled: stamp(),
            pid: std::process::id(),
            uptime: self.started.elapsed().as_secs(),
            running: true,
            connected: clients.len() as u16,
            capacity: capacity as u16,
            credentials: jobs.pending.load(Ordering::Acquire) as u8,
            output_bytes: clients.iter().map(|c| c.output.len() as u64).sum(),
            doors: clients
                .iter()
                .filter(|c| c.door.is_some() || c.classic.is_some())
                .count() as u16,
            counts: board.operator_counts(),
        };
        let _ = operator::publish(&self.directory, &snapshot);
    }
    pub fn stopped(&self, board: &Board, capacity: usize) {
        if !self.enabled {
            return;
        }
        let snapshot = Snapshot {
            sampled: stamp(),
            pid: std::process::id(),
            uptime: self.started.elapsed().as_secs(),
            running: false,
            connected: 0,
            capacity: capacity as u16,
            credentials: 0,
            output_bytes: 0,
            doors: 0,
            counts: board.operator_counts(),
        };
        let _ = operator::publish(&self.directory, &snapshot);
    }
}
fn stamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |v| v.as_secs())
}

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
    lease: Option<operator::lease::Lease>,
    directory: PathBuf,
    started: Instant,
    next: Instant,
}
impl Publisher {
    pub fn new(enabled: bool, directory: PathBuf) -> Self {
        let lease = if enabled {
            match operator::lease::Lease::acquire(&directory) {
                Ok(lease) => Some(lease),
                Err(_) => {
                    eprintln!("operator status unavailable: publisher lease could not be acquired");
                    None
                }
            }
        } else {
            None
        };
        Self {
            lease,
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
        if self.lease.is_none() || Instant::now() < self.next {
            return;
        }
        self.next = Instant::now() + Duration::from_secs(5);
        let snapshot = Snapshot {
            sampled: stamp(),
            pid: std::process::id(),
            instance: self.lease.as_ref().map(|lease| lease.instance().to_owned()),
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
        if self.lease.is_none() {
            return;
        }
        let snapshot = Snapshot {
            sampled: stamp(),
            pid: std::process::id(),
            instance: self.lease.as_ref().map(|lease| lease.instance().to_owned()),
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::DirBuilderExt;
    #[test]
    fn disabled_publisher_creates_no_lifetime_authority() {
        let dir = std::env::temp_dir().join(faction_core::request_id().unwrap());
        std::fs::DirBuilder::new().mode(0o700).create(&dir).unwrap();
        let publisher = Publisher::new(false, dir.clone());
        assert!(publisher.lease.is_none());
        assert_eq!(std::fs::read_dir(&dir).unwrap().count(), 0);
        drop(publisher);
        std::fs::remove_dir(dir).unwrap();
    }
    #[test]
    fn optional_publication_failure_never_steals_a_live_lease() {
        let dir = std::env::temp_dir().join(faction_core::request_id().unwrap());
        std::fs::DirBuilder::new().mode(0o700).create(&dir).unwrap();
        let first = Publisher::new(true, dir.clone());
        assert!(first.lease.is_some());
        let second = Publisher::new(true, dir.clone());
        assert!(second.lease.is_none());
        drop(second);
        assert!(operator::lease::Lease::acquire(&dir).is_err());
        drop(first);
        let next = operator::lease::Lease::acquire(&dir).unwrap();
        drop(next);
        std::fs::remove_dir_all(dir).unwrap();
    }
}

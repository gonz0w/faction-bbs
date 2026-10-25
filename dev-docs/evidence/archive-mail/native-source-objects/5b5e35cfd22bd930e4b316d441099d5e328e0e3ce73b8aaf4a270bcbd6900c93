//! Aggregate local-operator telemetry, never member authority or content.
use crate::Board;
use std::{
    fs::{self, File, OpenOptions},
    io::{self, Read, Write},
    os::unix::fs::{MetadataExt, OpenOptionsExt},
    path::Path,
    time::{Duration, Instant},
};

pub mod backup;
pub mod lease;

pub const STATUS_BYTES: usize = 2048;
pub const STATUS_FILE: &str = "operator.status";
/// Totals: members, pending, rooms, posts, channels, retained chat, published
/// files, pending files, active transfers, completed game runs.
pub type Counts = [u64; 10];
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Snapshot {
    pub sampled: u64,
    pub pid: u32,
    /// Per-start publisher identity; absent in legacy v1 observations.
    pub instance: Option<String>,
    pub uptime: u64,
    pub running: bool,
    pub connected: u16,
    pub capacity: u16,
    pub credentials: u8,
    pub output_bytes: u64,
    pub doors: u16,
    pub counts: Option<Counts>,
}
impl Snapshot {
    pub fn encode(&self) -> String {
        let counts = self.counts.map_or_else(
            || "unavailable".to_owned(),
            |a| a.map(|n| n.to_string()).join(","),
        );
        let mut text = format!(
            "version={}\nsampled={}\npid={}\nuptime={}\nrunning={}\nconnected={}\ncapacity={}\ncredentials={}\noutput_bytes={}\ndoors={}\ncounts={}\n",
            if self.instance.is_some() { 2 } else { 1 },
            self.sampled,
            self.pid,
            self.uptime,
            u8::from(self.running),
            self.connected,
            self.capacity,
            self.credentials,
            self.output_bytes,
            self.doors,
            counts
        );
        if let Some(instance) = &self.instance {
            text.push_str(&format!("instance={instance}\n"));
        }
        text
    }
    pub fn parse(text: &str) -> Option<Self> {
        if text.len() > STATUS_BYTES || !text.is_ascii() {
            return None;
        }
        let mut lines = text.lines();
        let mut value = |name: &str| lines.next()?.strip_prefix(name);
        let version = value("version=")?;
        if !matches!(version, "1" | "2") {
            return None;
        }
        let sampled = value("sampled=")?.parse().ok()?;
        let pid = value("pid=")?.parse().ok()?;
        let uptime = value("uptime=")?.parse().ok()?;
        let running = match value("running=")? {
            "1" => true,
            "0" => false,
            _ => return None,
        };
        let connected = value("connected=")?.parse().ok()?;
        let capacity = value("capacity=")?.parse().ok()?;
        let credentials = value("credentials=")?.parse().ok()?;
        let output_bytes = value("output_bytes=")?.parse().ok()?;
        let doors = value("doors=")?.parse().ok()?;
        let counts = match value("counts=")? {
            "unavailable" => None,
            raw => {
                let mut counts = [0; 10];
                let mut values = raw.split(',');
                for v in &mut counts {
                    *v = values.next()?.parse().ok()?;
                }
                if values.next().is_some() {
                    return None;
                }
                Some(counts)
            }
        };
        let instance = if version == "2" {
            let raw = value("instance=")?;
            if !lease::valid_instance(raw) {
                return None;
            }
            Some(raw.to_owned())
        } else {
            None
        };
        if lines.next().is_some()
            || !(1..=250).contains(&capacity)
            || connected > capacity
            || credentials > 2
            || doors > connected
            || output_bytes > u64::from(capacity) * 32768
            || pid == 0
            || sampled == 0
        {
            return None;
        }
        Some(Self {
            sampled,
            pid,
            instance,
            uptime,
            running,
            connected,
            capacity,
            credentials,
            output_bytes,
            doors,
            counts,
        })
    }
}

struct Budget<'a> {
    db: &'a rusqlite::Connection,
    timeout: u32,
}
impl Drop for Budget<'_> {
    fn drop(&mut self) {
        let _ = self.db.progress_handler(0, None::<fn() -> bool>);
        let _ = self
            .db
            .busy_timeout(Duration::from_millis(u64::from(self.timeout)));
    }
}
impl Board {
    /// Filesystem-owner aggregate API only. A 5-ms / 50k-op progress budget
    /// prevents a large catalog holding up the shared owner. No bodies fetched.
    pub fn operator_counts(&self) -> Option<Counts> {
        let timeout: u32 = self
            .db
            .query_row("PRAGMA busy_timeout", [], |r| r.get(0))
            .ok()?;
        let _budget = Budget {
            db: &self.db,
            timeout,
        };
        self.db.busy_timeout(Duration::ZERO).ok()?;
        let start = Instant::now();
        let mut calls = 0;
        self.db
            .progress_handler(
                1000,
                Some(move || {
                    calls += 1;
                    calls >= 50 || start.elapsed() >= Duration::from_millis(5)
                }),
            )
            .ok()?;
        self.db
            .query_row(
                "SELECT (SELECT count(*) FROM members),
             (SELECT count(*) FROM members WHERE status='pending'),
             (SELECT count(*) FROM rooms), (SELECT count(*) FROM posts),
             (SELECT count(*) FROM chat_channels), (SELECT count(*) FROM chat_messages),
             (SELECT count(*) FROM file_objects WHERE state='published'),
             (SELECT count(*) FROM file_objects WHERE state='pending'),
             (SELECT count(*) FROM file_transfers WHERE state='active'),
             (SELECT coalesce(sum(plays),0) FROM game_scores)",
                [],
                |row| {
                    let mut counts = [0; 10];
                    for (i, value) in counts.iter_mut().enumerate() {
                        *value = u64::try_from(row.get::<_, i64>(i)?)
                            .map_err(|_| rusqlite::Error::InvalidQuery)?;
                    }
                    Ok(counts)
                },
            )
            .ok()
    }
}

/// Atomic one-way owner snapshot. Failure affects telemetry, never BBS authority.
pub fn publish(directory: &Path, snapshot: &Snapshot) -> io::Result<()> {
    let text = snapshot.encode();
    if Snapshot::parse(&text).is_none() {
        return Err(io::ErrorKind::InvalidData.into());
    }
    let temp = directory.join(".operator.status.next");
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(&temp)?;
    let result = (|| {
        file.write_all(text.as_bytes())?;
        fs::rename(&temp, directory.join(STATUS_FILE))
    })();
    if result.is_err() {
        let _ = fs::remove_file(temp);
    }
    result
}

/// Unix owner/root access only. No login, sysop-role, environment or TERM bypass.
/// Does not open SQLite, create files or acquire the board's writer lock.
pub fn read(directory: &Path) -> io::Result<Snapshot> {
    Snapshot::parse(&read_text(directory, STATUS_FILE, STATUS_BYTES)?)
        .ok_or_else(|| io::ErrorKind::InvalidData.into())
}

fn open_private(directory: &Path, name: &str, cap: usize) -> io::Result<File> {
    let parent = fs::symlink_metadata(directory)?;
    let uid = rustix::process::geteuid().as_raw();
    if !parent.is_dir() || parent.mode() & 0o077 != 0 || (uid != 0 && parent.uid() != uid) {
        return Err(io::ErrorKind::PermissionDenied.into());
    }
    let file = File::options()
        .read(true)
        .custom_flags(
            (rustix::fs::OFlags::NOFOLLOW
                | rustix::fs::OFlags::NONBLOCK
                | rustix::fs::OFlags::CLOEXEC)
                .bits() as i32,
        )
        .open(directory.join(name))?;
    let m = file.metadata()?;
    if !m.is_file()
        || m.nlink() != 1
        || m.mode() & 0o077 != 0
        || m.uid() != parent.uid()
        || m.len() > cap as u64
    {
        return Err(io::ErrorKind::PermissionDenied.into());
    }
    Ok(file)
}

fn read_text(directory: &Path, name: &str, cap: usize) -> io::Result<String> {
    let file = open_private(directory, name, cap)?;
    let mut text = String::new();
    file.take(cap as u64 + 1).read_to_string(&mut text)?;
    if text.len() > cap {
        return Err(io::ErrorKind::InvalidData.into());
    }
    Ok(text)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn strict_versioned_aggregate_protocol() {
        let s = Snapshot {
            sampled: 10,
            pid: 2,
            instance: None,
            uptime: 2,
            running: true,
            connected: 1,
            capacity: 8,
            credentials: 2,
            output_bytes: 100,
            doors: 1,
            counts: Some([0; 10]),
        };
        let raw = s.encode();
        assert_eq!(Snapshot::parse(&raw), Some(s));
        for invalid in [
            raw.replace("version=1", "version=2"),
            raw.replace("connected=1", "connected=9"),
            raw.replace("credentials=2", "credentials=3"),
            raw.clone() + "body=secret\n",
            raw.replace("counts=0,0,0,0,0,0,0,0,0,0", "counts=0"),
            raw.replace("pid=2", "pid=0"),
        ] {
            assert!(Snapshot::parse(&invalid).is_none());
        }
        assert!(Snapshot::parse(&"x".repeat(STATUS_BYTES + 1)).is_none());
    }
    #[test]
    fn aggregate_budget_restores_normal_queries_and_busy_timeout() {
        let p = std::env::temp_dir().join(format!(
            "faction-operator-budget-{}",
            crate::request_id().unwrap()
        ));
        let b = Board::open(&p).unwrap();
        b.db.execute_batch("WITH RECURSIVE n(x) AS (VALUES(1) UNION ALL SELECT x+1 FROM n WHERE x<20000)
            INSERT INTO members(handle,password_hash,role,status) SELECT 'u'||x,'x','member','pending' FROM n").unwrap();
        let before: i64 =
            b.db.query_row("PRAGMA busy_timeout", [], |r| r.get(0))
                .unwrap();
        assert_eq!(b.operator_counts(), None);
        assert_eq!(
            b.db.query_row::<i64, _, _>("SELECT count(*) FROM members", [], |r| r.get(0))
                .unwrap(),
            20000
        );
        assert_eq!(
            b.db.query_row::<i64, _, _>("PRAGMA busy_timeout", [], |r| r.get(0))
                .unwrap(),
            before
        );
        drop(b);
        std::fs::remove_dir_all(p).unwrap();
    }
}

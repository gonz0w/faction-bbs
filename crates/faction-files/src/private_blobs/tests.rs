use super::*;
use crate::private_blobs::*;
use sha2::{Digest, Sha256};
use std::{
    fs as stdfs,
    os::unix::fs::DirBuilderExt,
    path::PathBuf,
    process::{Command, Stdio},
    sync::{
        Mutex, MutexGuard,
        atomic::{AtomicU64, Ordering},
    },
    time::{Duration, Instant},
};
// A spawn can briefly retain another thread's CLOEXEC lock description until
// exec. Keep store fixtures and child launch/reap in one process-local window;
// production continues to report legitimate contention immediately as Busy.
static FIXTURE_LIFETIME: Mutex<()> = Mutex::new(());
static NEXT_FIXTURE: AtomicU64 = AtomicU64::new(0);
struct Fixture {
    root: PathBuf,
    _lifetime: MutexGuard<'static, ()>,
}
impl Fixture {
    fn new() -> Self {
        let lifetime = FIXTURE_LIFETIME
            .lock()
            .expect("fixture coordination poisoned");
        let unique = NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed);
        let p = std::env::temp_dir().canonicalize().unwrap().join(format!(
            "faction260-fault-{}-{unique}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        stdfs::DirBuilder::new().mode(0o700).create(&p).unwrap();
        stdfs::write(p.join("OWNED"), b"Owned issue260 fault fixture").unwrap();
        stdfs::DirBuilder::new()
            .mode(0o700)
            .create(p.join("store"))
            .unwrap();
        Self {
            root: p,
            _lifetime: lifetime,
        }
    }
    fn store(&self) -> PathBuf {
        self.root.join("store")
    }
    fn bounded(&self) {
        let entries = stdfs::read_dir(self.store())
            .unwrap()
            .map(|p| p.unwrap().metadata().unwrap().len())
            .collect::<Vec<_>>();
        assert!(entries.len() <= 3);
        assert!(entries.iter().sum::<u64>() <= MAX_DISK_BYTES as u64);
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        FAULT.set(Fault::None);
        assert_eq!(
            stdfs::read(self.root.join("OWNED")).unwrap(),
            b"Owned issue260 fault fixture"
        );
        stdfs::remove_dir_all(&self.root).unwrap();
        assert!(!self.root.exists());
    }
}
struct Allow;
impl Access for Allow {
    fn require(&self, _: &Principal, _: &Binding, _: Action, _: u64) -> Result<(), Error> {
        Ok(())
    }
}
fn binding() -> Binding {
    Binding {
        sender: Principal {
            account: "alice@home.test".into(),
            device: 1,
            identity: [1; 32],
        },
        recipient: Principal {
            account: "bob@other.test".into(),
            device: 2,
            identity: [2; 32],
        },
        id: Sha256::digest([9u8; 32]).into(),
        size: 32,
        created: 100,
        expires: 200,
    }
}
#[test]
fn each_write_sync_and_publication_error_poison_requires_safe_reopen() {
    for point in [
        Point::Write,
        Point::FileSync,
        Point::BeforeRename,
        Point::AfterRename,
        Point::DirectorySync,
    ] {
        let f = Fixture::new();
        let mut s = Store::open(&f.store()).unwrap();
        let b = binding();
        FAULT.set(Fault::Error(point));
        let expected = if matches!(point, Point::AfterRename | Point::DirectorySync) {
            Error::PublicationUncertain
        } else {
            Error::Storage
        };
        assert_eq!(
            s.dispatch(&b.sender, &Allow, Request::Begin(b.clone()), 100),
            Err(expected)
        );
        assert_eq!(
            s.dispatch(&b.sender, &Allow, Request::Status { id: b.id }, 100),
            Err(Error::Poisoned)
        );
        f.bounded();
        drop(s);
        FAULT.set(Fault::None);
        let mut reopened = Store::open(&f.store()).unwrap();
        let status = reopened.dispatch(&b.sender, &Allow, Request::Status { id: b.id }, 100);
        if expected == Error::PublicationUncertain {
            assert!(status.is_ok())
        } else {
            assert_eq!(status, Err(Error::NotFound))
        }
        f.bounded();
    }
}
#[test]
#[ignore = "bounded parent invokes this process-death helper"]
fn death_helper() {
    let root = PathBuf::from(std::env::var_os("FACTION260_DEATH_STORE").expect("owned child path"));
    let point = match std::env::var("FACTION260_DEATH_POINT").unwrap().as_str() {
        "write" => Point::Write,
        "sync" => Point::FileSync,
        "before" => Point::BeforeRename,
        "after" => Point::AfterRename,
        "directory" => Point::DirectorySync,
        _ => panic!("invalid test point"),
    };
    let mut s = Store::open(&root).unwrap();
    FAULT.set(Fault::Death(point));
    let b = binding();
    let _ = s.dispatch(&b.sender, &Allow, Request::Begin(b.clone()), 100);
    panic!("helper missed point")
}
#[test]
fn actual_process_death_at_each_boundary_reopens_exact_old_or_new_snapshot() {
    for (name, published) in [
        ("write", false),
        ("sync", false),
        ("before", false),
        ("after", true),
        ("directory", true),
    ] {
        let f = Fixture::new();
        drop(Store::open(&f.store()).unwrap());
        let mut child = Command::new(std::env::current_exe().unwrap())
            .args([
                "--ignored",
                "--exact",
                "private_blobs::disk::tests::death_helper",
                "--nocapture",
            ])
            .env("FACTION260_DEATH_STORE", f.store())
            .env("FACTION260_DEATH_POINT", name)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        let deadline = Instant::now() + Duration::from_secs(5);
        let marker = f.store().join("BOUNDARY");
        // Creation alone is not the rendezvous: a kill between create and
        // write would leave an empty marker before the child reached its park.
        let complete =
            || stdfs::read(&marker).is_ok_and(|bytes| bytes == b"owned issue260 exact boundary");
        while !complete() && Instant::now() < deadline {
            if child.try_wait().unwrap().is_some() {
                break;
            }
            std::thread::sleep(Duration::from_millis(5));
        }
        let reached = complete();
        let _ = child.kill();
        let status = child.wait().unwrap();
        assert!(reached, "child did not reach exact boundary");
        assert!(!status.success());
        assert_eq!(
            stdfs::read(&marker).unwrap(),
            b"owned issue260 exact boundary"
        );
        stdfs::remove_file(marker).unwrap();
        f.bounded();
        let mut reopened = Store::open(&f.store()).unwrap();
        let b = binding();
        let got = reopened.dispatch(&b.sender, &Allow, Request::Status { id: b.id }, 100);
        if published {
            assert!(got.is_ok())
        } else {
            assert_eq!(got, Err(Error::NotFound))
        }
        f.bounded();
    }
}

#[test]
fn authorized_expiry_clock_publication_failures_preserve_poison_and_uncertainty() {
    for point in [
        Point::Write,
        Point::FileSync,
        Point::BeforeRename,
        Point::AfterRename,
        Point::DirectorySync,
    ] {
        let f = Fixture::new();
        let mut s = Store::open(&f.store()).unwrap();
        let b = binding();
        s.dispatch(&b.sender, &Allow, Request::Begin(b.clone()), 100)
            .unwrap();
        FAULT.set(Fault::Error(point));
        let expected = if matches!(point, Point::AfterRename | Point::DirectorySync) {
            Error::PublicationUncertain
        } else {
            Error::Storage
        };
        assert_eq!(
            s.dispatch(&b.sender, &Allow, Request::Status { id: b.id }, 200),
            Err(expected)
        );
        assert_eq!(
            s.dispatch(&b.sender, &Allow, Request::Status { id: b.id }, 199),
            Err(Error::Poisoned)
        );
        drop(s);
        FAULT.set(Fault::None);
        let mut reopened = Store::open(&f.store()).unwrap();
        let status = reopened.dispatch(&b.sender, &Allow, Request::Status { id: b.id }, 199);
        if expected == Error::PublicationUncertain {
            assert_eq!(status, Err(Error::Clock));
        } else {
            assert!(status.is_ok());
        }
        assert_eq!(
            reopened.dispatch(&b.sender, &Allow, Request::Status { id: b.id }, 200),
            Err(Error::Expired)
        );
        f.bounded();
    }
}

#[test]
fn fixture_lifetime_excludes_other_store_and_process_launch_windows() {
    use std::sync::mpsc::{self, RecvTimeoutError};
    let first = Fixture::new();
    let (attempt_tx, attempt_rx) = mpsc::channel();
    let (acquired_tx, acquired_rx) = mpsc::channel();
    let (finished_tx, finished_rx) = mpsc::channel();
    let thread = std::thread::spawn(move || {
        let result = std::panic::catch_unwind(|| {
            attempt_tx.send(()).unwrap();
            let second = Fixture::new();
            acquired_tx.send(()).unwrap();
            drop(second);
        });
        finished_tx.send(()).unwrap();
        result
    });
    let attempted = attempt_rx.recv_timeout(Duration::from_secs(5));
    let early = acquired_rx.recv_timeout(Duration::from_millis(50));
    // Release even on RED or failed rendezvous, before assertions or joining.
    drop(first);
    let finished = finished_rx.recv_timeout(Duration::from_secs(5));
    let joined = if finished.is_ok() {
        Some(thread.join())
    } else {
        None
    };
    assert!(
        attempted.is_ok(),
        "fixture contender did not reach acquisition"
    );
    assert!(
        finished.is_ok(),
        "fixture contender did not finish after release"
    );
    assert!(
        matches!(joined, Some(Ok(Ok(())))),
        "fixture contender panicked"
    );
    assert_eq!(
        early,
        Err(RecvTimeoutError::Timeout),
        "another fixture entered the store/process-launch window"
    );
    assert!(
        acquired_rx.try_recv().is_ok(),
        "fixture contender did not acquire after release"
    );
}

#[test]
fn an_inherited_open_description_retains_the_lock_until_the_last_reference_closes() {
    let f = Fixture::new();
    let store = Store::open(&f.store()).unwrap();
    // A safe duplicate models the shared open-file description retained by a
    // spawned child before exec closes CLOEXEC descriptors. It does not claim
    // to trace the exact platform spawn syscall sequence.
    let inherited = store.disk._lock.try_clone().unwrap();
    drop(store);
    assert!(matches!(Store::open(&f.store()), Err(Error::Busy)));
    drop(inherited);
    assert!(Store::open(&f.store()).is_ok());
}

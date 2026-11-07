//! Test-only coordinator: no worker publishes before both cold receipts.
use super::*;
use std::sync::mpsc;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Failure {
    Preparation,
    EarlyPanic,
    MissingReadiness,
    CoordinatorUnwind,
    ReleaseClosed,
}
#[derive(Clone, Copy, Eq, PartialEq)]
enum Fault {
    None,
    Panic,
    MissingReadiness,
    CoordinatorUnwind,
}
pub(super) struct Report {
    pub(super) outcome: Result<(), Failure>,
    joined: usize,
}
pub(super) fn two_cold_candidates(credentials: &ClientCredentials) -> Report {
    coordinate([credentials, credentials], Fault::None)
}
fn coordinate(credentials: [&ClientCredentials; 2], fault: Fault) -> Report {
    std::thread::scope(|threads| {
        let (ready, received) = mpsc::channel();
        let mut releases = Vec::new();
        let mut workers = Vec::new();
        for (index, credentials) in credentials.into_iter().enumerate() {
            let ready = ready.clone();
            let (release, resume) = mpsc::channel();
            releases.push(release);
            workers.push(threads.spawn(move || {
                let prepared = match credentials.prepare_connector(None) {
                    Ok(prepared) => prepared,
                    Err(stage) => {
                        ready.send(Err(Failure::Preparation)).unwrap();
                        assert_eq!(stage, Stage::Configuration);
                        return Err(Failure::Preparation);
                    }
                };
                assert!(matches!(prepared, PreparedConnector::Cold(_)));
                if index == 1 {
                    match fault {
                        Fault::Panic => panic!("controlled early readiness failure"),
                        Fault::MissingReadiness => return Err(Failure::MissingReadiness),
                        _ => (),
                    }
                }
                ready.send(Ok(())).unwrap();
                // The coordinator owns every sender. Failure/unwind drops them
                // before joining; closure releases this wait without publishing.
                resume.recv().map_err(|_| Failure::ReleaseClosed)?;
                let connector = credentials.finish_connector(prepared, None);
                assert!(
                    connector
                        .configure()
                        .unwrap()
                        .into_ssl("provider.test")
                        .is_ok()
                );
                Ok(())
            }));
        }
        drop(ready);
        let coordination = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            for _ in 0..2 {
                received
                    .recv_timeout(Duration::from_secs(1))
                    .map_err(|_| Failure::MissingReadiness)??;
            }
            // Both workers have Cold candidates and are waiting for release.
            // Neither can access publication while these cache checks run.
            for credentials in credentials {
                assert!(credentials.connector.try_lock().unwrap().is_none());
            }
            if fault == Fault::CoordinatorUnwind {
                panic!("controlled coordinator unwind");
            }
            for release in &releases {
                release.send(()).map_err(|_| Failure::ReleaseClosed)?;
            }
            Ok(())
        }))
        .unwrap_or(Err(Failure::CoordinatorUnwind));
        drop(releases);
        // Join EVERY participant before returning any failure to the caller.
        let outcomes: Vec<_> = workers
            .into_iter()
            .map(|worker| worker.join().unwrap_or(Err(Failure::EarlyPanic)))
            .collect();
        let joined = outcomes.len();
        let failure = outcomes
            .iter()
            .filter_map(|outcome| outcome.as_ref().err().copied())
            .find(|failure| *failure != Failure::ReleaseClosed)
            .or(coordination.err())
            .or_else(|| outcomes.into_iter().find_map(Result::err));
        Report {
            outcome: failure.map_or(Ok(()), Err),
            joined,
        }
    })
}

#[test]
fn shared_setup_rendezvous_preparation_failure_is_reported_after_all_joins() {
    let peer = SetupPeer::new();
    let credentials = peer.credentials();
    let mut bad = credentials.clone();
    bad.key = SetupPeer::new().device.1;
    let report = coordinate([&credentials, &bad], Fault::None);
    assert_eq!(report.joined, 2);
    assert_eq!(report.outcome, Err(Failure::Preparation));
    assert!(credentials.connector.try_lock().unwrap().is_none());
}
#[test]
fn shared_setup_rendezvous_early_panic_releases_and_joins_the_survivor() {
    let peer = SetupPeer::new();
    let credentials = peer.credentials();
    let report = coordinate([&credentials, &credentials], Fault::Panic);
    assert_eq!(report.joined, 2);
    assert_eq!(report.outcome, Err(Failure::EarlyPanic));
    assert!(credentials.connector.try_lock().unwrap().is_none());
}
#[test]
fn shared_setup_rendezvous_missing_receipt_releases_and_joins_the_survivor() {
    let peer = SetupPeer::new();
    let credentials = peer.credentials();
    let report = coordinate([&credentials, &credentials], Fault::MissingReadiness);
    assert_eq!(report.joined, 2);
    assert_eq!(report.outcome, Err(Failure::MissingReadiness));
    assert!(credentials.connector.try_lock().unwrap().is_none());
}
#[test]
fn shared_setup_rendezvous_coordinator_unwind_releases_and_joins_all_workers() {
    let peer = SetupPeer::new();
    let credentials = peer.credentials();
    let report = coordinate([&credentials, &credentials], Fault::CoordinatorUnwind);
    assert_eq!(report.joined, 2);
    assert_eq!(report.outcome, Err(Failure::CoordinatorUnwind));
    assert!(credentials.connector.try_lock().unwrap().is_none());
}

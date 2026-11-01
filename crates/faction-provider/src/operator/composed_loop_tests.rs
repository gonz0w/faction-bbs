//! Exercise the same maintained loop, with real mTLS publication and consumption.
use super::*;
use crate::Error as ProviderError;
use crate::{
    Runtime,
    delivery::{Collection, Delivery, Event, RelayPeer, RelayRoutes},
    tls::Server,
};
use faction_files::private_blobs::Store;
use std::sync::mpsc;
#[path = "composed_loop.rs"]
mod composed_loop;
fn mapped<T, E>(result: Result<T, E>) -> Result<T, ()> {
    result.map_err(|_| ())
}

#[test]
fn maintained_operator_reaps_consumed_delivery_when_real_worker_finishes_after_cleanup() {
    let h = Homes::new(true);
    let local = &h.pair.homes[0];
    let foreign = &h.pair.homes[1];
    let binding = h.pair.binding(0);
    let mut next_binding = binding.clone();
    next_binding.id = sha256(&[7; 32]);
    let store = h.empty(0);
    let _recipient_store = h.empty(1);
    // Preserve exact selected TLS/Store and durable policy inputs before setup
    // assertions can unwind the fixture. No object is seeded into either Store.
    capture_carrier_optional(&h, "maintained-loop-early");
    for i in 0..2 {
        retain_synthetic_fixture(
            &format!("maintained-loop-{}-policy{i}.snapshot", std::process::id()),
            &fs::read(
                h.pair
                    .root
                    .join(format!("policy{i}/provider-policy.snapshot")),
            )
            .unwrap(),
        );
    }
    retain_synthetic_fixture(&format!("maintained-loop-{}-routes.txt",std::process::id()),
        format!("sender_listener={}\nrecipient_listener={}\nsender_home={}\nrecipient_home={}\npolicy=exact-captured-durable-snapshots\nruntime_instances=one-per-home\n",
            local.callback.local_addr().unwrap(),foreign.callback.local_addr().unwrap(),local.name,foreign.name).as_bytes());
    let recipient_before = h.bytes(1);
    let address = local.callback.local_addr().unwrap();
    let listener = local.callback.try_clone().unwrap();
    let server = Server::new_composed(
        &local.server.0,
        &local.server.1,
        &[local.ca.0.clone(), foreign.ca.0.clone()],
    )
    .unwrap();
    let profile = PublicationWireProfile::new(local.name.into(), vec!["observer".into()]).unwrap();
    let routes = RelayRoutes::new(
        local.runtime(),
        carrier_endpoint(local, &local.relay, address),
        vec![RelayPeer {
            home: foreign.name.into(),
            publication: h.endpoint(0),
            introspection: local.endpoint(foreign),
            resource: carrier_endpoint(
                foreign,
                &local.relay,
                foreign.callback.local_addr().unwrap(),
            ),
        }],
    )
    .unwrap();
    let endpoint = carrier_endpoint(local, &local.device, address);
    let cancel = Arc::new(AtomicBool::new(false));
    let (returned_tx, returned_rx) = mpsc::sync_channel(1);
    let (exit_tx, exit_rx) = mpsc::sync_channel(1);
    let (cleanup_tx, cleanup_rx) = mpsc::sync_channel(1);
    let (finished_tx, finished_rx) = mpsc::sync_channel(1);
    let (resume_tx, resume_rx) = mpsc::sync_channel(1);
    let hooks = composed_loop::Hooks {
        worker_once: AtomicBool::new(true),
        cleanup_once: AtomicBool::new(true),
        work_returned: returned_tx,
        worker_exit: Mutex::new(exit_rx),
        cleanup_seen: cleanup_tx,
        physical_finished: finished_tx,
        resume_admission: Mutex::new(resume_rx),
        check_pending: |delivery| {
            assert!(delivery.is_consumed());
            assert_eq!(
                delivery.test_original_attempts(),
                3,
                "test must not exhaust the original eight attempts"
            );
        },
    };
    struct Stop<'a> {
        runtime: &'a Runtime,
        cancel: Arc<AtomicBool>,
    }
    impl Drop for Stop<'_> {
        fn drop(&mut self) {
            self.cancel.store(true, Ordering::SeqCst);
            self.runtime.stop();
        }
    }
    let (submission, loop_result) = thread::scope(|scope| {
        // Drop cancellation before scope joins if a witness assertion fails.
        let stop = Stop {
            runtime: local.runtime(),
            cancel: cancel.clone(),
        };
        let loop_thread = scope.spawn(|| {
            thread::scope(|workers| {
                composed_loop::run(
                    workers,
                    composed_loop::Inputs {
                        server: &server,
                        listener,
                        runtime: local.runtime(),
                        cancel: &cancel,
                        routes: &routes,
                        store: &store,
                        home: local.name,
                        profile: &profile,
                        port: address.port(),
                    },
                    Some(&hooks),
                )
            })
        });
        let callback = scope.spawn(|| h.serve(1));
        let mut old = endpoint.operation(Arc::new(AtomicBool::new(false)));
        old.submit_publication(binding.clone(), Request::Begin(binding.clone()))
            .unwrap();
        metadata_capture_inputs(&h, &old, "maintained-loop-old-actual-source");
        returned_rx.recv_timeout(Duration::from_secs(2)).unwrap();
        callback.join().unwrap().unwrap();
        assert_eq!(local.charged(), 1);
        match old.collect().unwrap() {
            Collection::Complete(Response::Progress {
                binding: got,
                offset,
                committed,
            }) => {
                assert_eq!(got, binding);
                assert_eq!(offset, 0);
                assert!(!committed);
            }
            _ => panic!("actual old publication was not consumed"),
        }
        let one = h.bytes(0);
        // Canonical FBCS0001 has an 8-byte tag, 8-byte clock and count byte.
        assert!(one.starts_with(b"FBCS0001"));
        assert_eq!(one[16], 1);
        assert!(
            h.bytes(1) == recipient_before,
            "recipient Store changed during sender publication"
        );
        assert_eq!(h.callbacks[1].load(Ordering::SeqCst), 1);
        cleanup_rx.recv_timeout(Duration::from_secs(2)).unwrap();
        assert_eq!(
            local.charged(),
            1,
            "old reservation was released while worker lived"
        );
        exit_tx.send(()).unwrap();
        finished_rx.recv_timeout(Duration::from_secs(2)).unwrap();
        assert_eq!(
            local.charged(),
            1,
            "pending must still own its old physical reservation"
        );
        resume_tx.send(()).unwrap();
        let mut next = endpoint.operation(Arc::new(AtomicBool::new(false)));
        let submission =
            next.submit_publication(next_binding.clone(), Request::Begin(next_binding.clone()));
        if submission.is_ok() {
            assert_eq!(
                local.charged(),
                1,
                "new source overlapped a leaked old reservation"
            );
            // The receipt alone is insufficient: run the second actual recipient
            // callback, witness real Work completion and collect its exact result.
            let callback = scope.spawn(|| h.serve(1));
            returned_rx.recv_timeout(Duration::from_secs(2)).unwrap();
            callback.join().unwrap().unwrap();
            match next.collect().unwrap() {
                Collection::Complete(Response::Progress {
                    binding: got,
                    offset,
                    committed,
                }) => {
                    assert!(
                        got == next_binding,
                        "fresh authenticated result substituted its binding"
                    );
                    assert_eq!(offset, 0);
                    assert!(!committed);
                }
                _ => panic!("fresh receipt lacked an authenticated completed Begin result"),
            }
            let two = h.bytes(0);
            assert!(two.starts_with(b"FBCS0001"));
            assert_eq!(two[16], 2);
            assert_eq!(
                two.len(),
                17 + 2 * (one.len() - 17),
                "fresh Begin did not add exactly one empty object"
            );
            assert!(
                h.bytes(1) == recipient_before,
                "recipient Store changed during sender publication"
            );
            assert_eq!(h.callbacks[1].load(Ordering::SeqCst), 2);
            capture_carrier_optional(&h, "maintained-loop-fresh-completed");
        }
        drop(stop);
        let stopped = Instant::now();
        let result = loop_thread.join().unwrap();
        assert!(stopped.elapsed() < Duration::from_secs(5));
        (submission, result)
    });
    assert!(loop_result.is_ok() && loop_result.unwrap().is_ok());
    assert_eq!(
        local.charged(),
        0,
        "real work/pending lease survived maintained-loop drain"
    );
    capture_carrier_optional(&h, "maintained-loop-old-order");
    assert!(
        submission.is_ok(),
        "fresh authorized SUBMIT was refused after physical worker completion: {submission:?}"
    );
}

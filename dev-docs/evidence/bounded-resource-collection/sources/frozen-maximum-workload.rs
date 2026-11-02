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
        dispatch_delay: AtomicBool::new(false),
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

#[test]
fn observe_actual_own_home_get_readiness_through_both_maintained_loops() {
    use crate::delivery::{DiagnosticPhase, OperationSnapshot};

    struct Stop<'a> {
        runtimes: [&'a Runtime; 2],
        cancel: Arc<AtomicBool>,
    }
    impl Drop for Stop<'_> {
        fn drop(&mut self) {
            self.cancel.store(true, Ordering::SeqCst);
            for runtime in self.runtimes {
                runtime.stop();
            }
        }
    }
    fn notifications(returned: mpsc::SyncSender<()>) -> composed_loop::Hooks {
        let (_, worker_exit) = mpsc::sync_channel(1);
        let (cleanup_seen, _) = mpsc::sync_channel(1);
        let (physical_finished, _) = mpsc::sync_channel(1);
        let (_, resume_admission) = mpsc::sync_channel(1);
        composed_loop::Hooks {
            dispatch_delay: AtomicBool::new(false),
            worker_once: AtomicBool::new(false),
            cleanup_once: AtomicBool::new(false),
            work_returned: returned,
            worker_exit: Mutex::new(worker_exit),
            cleanup_seen,
            physical_finished,
            resume_admission: Mutex::new(resume_admission),
            check_pending: |_| (),
        }
    }
    #[derive(Clone, Copy)]
    struct CollectTrace {
        start: Duration,
        end: Duration,
        outcome: &'static str,
        snapshot: Option<OperationSnapshot>,
    }
    fn observe() -> Result<(), String> {
        let h = Homes::new(true);
        let stores = [h.empty(0), h.empty(1)];
        let ciphertext = [37; 17];
        let mut binding = h.pair.binding(0);
        binding.id = sha256(&ciphertext);
        binding.size = ciphertext.len() as u64;
        capture_carrier_optional(&h, "own-home-readiness-early");
        for i in 0..2 {
            retain_synthetic_fixture(
                &format!(
                    "own-home-readiness-{}-policy{i}.snapshot",
                    std::process::id()
                ),
                &fs::read(
                    h.pair
                        .root
                        .join(format!("policy{i}/provider-policy.snapshot")),
                )
                .map_err(|e| e.to_string())?,
            );
        }
        retain_synthetic_fixture(
            &format!("own-home-readiness-{}-binding.bin", std::process::id()),
            &faction_files::private_blobs::encode_request(&Request::Begin(binding.clone()))
                .map_err(|e| format!("binding encoding: {e:?}"))?,
        );
        retain_synthetic_fixture(
            &format!("own-home-readiness-{}-ciphertext.bin", std::process::id()),
            &ciphertext,
        );
        let server = |i: usize| {
            let home = &h.pair.homes[i];
            let foreign = &h.pair.homes[1 - i];
            Server::new_composed(
                &home.server.0,
                &home.server.1,
                &[home.ca.0.clone(), foreign.ca.0.clone()],
            )
            .map_err(|e| format!("composed server: {e:?}"))
        };
        let servers = [server(0)?, server(1)?];
        let profile = |i: usize| {
            PublicationWireProfile::new(h.pair.homes[i].name.into(), vec!["observer".into()])
                .map_err(|e| format!("publication profile: {e:?}"))
        };
        let profiles = [profile(0)?, profile(1)?];
        let routes = |i: usize| {
            let home = &h.pair.homes[i];
            let foreign = &h.pair.homes[1 - i];
            RelayRoutes::new(
                home.runtime(),
                carrier_endpoint(home, &home.relay, home.callback.local_addr().unwrap()),
                vec![RelayPeer {
                    home: foreign.name.into(),
                    publication: h.endpoint(i),
                    introspection: home.endpoint(foreign),
                    resource: carrier_endpoint(
                        foreign,
                        &home.relay,
                        foreign.callback.local_addr().unwrap(),
                    ),
                }],
            )
            .map_err(|e| format!("relay routes: {e:?}"))
        };
        let routes = [routes(0)?, routes(1)?];
        let (returned0, observed0) = mpsc::sync_channel(8);
        let (returned1, observed1) = mpsc::sync_channel(8);
        let hooks = [notifications(returned0), notifications(returned1)];
        let cancel = Arc::new(AtomicBool::new(false));
        let (result, loops, drain) = thread::scope(|scope| {
            let stop = Stop {
                runtimes: [h.pair.homes[0].runtime(), h.pair.homes[1].runtime()],
                cancel: cancel.clone(),
            };
            let loops: [_; 2] = std::array::from_fn(|i| {
                let home = &h.pair.homes[i];
                let server = &servers[i];
                let routes = &routes[i];
                let store = &stores[i];
                let profile = &profiles[i];
                let hooks = &hooks[i];
                let cancel = &cancel;
                let listener = home.callback.try_clone().unwrap();
                scope.spawn(move || {
                    thread::scope(|workers| {
                        composed_loop::run(
                            workers,
                            composed_loop::Inputs {
                                server,
                                listener,
                                runtime: home.runtime(),
                                cancel,
                                routes,
                                store,
                                home: home.name,
                                profile,
                                // This input is only the loop's ready-line display;
                                // keep actual private listener ports out of diagnostics.
                                port: 0,
                            },
                            Some(hooks),
                        )
                    })
                })
            });
            let result = (|| {
                let sender = &h.pair.homes[0];
                let endpoint = carrier_endpoint(
                    sender,
                    &sender.device,
                    sender.callback.local_addr().unwrap(),
                );
                // Legitimate authenticated setup only: no Access or seeded object.
                for (number, request) in [
                    Request::Begin(binding.clone()),
                    Request::PutChunk {
                        id: binding.id,
                        offset: 0,
                        bytes: ciphertext.to_vec(),
                    },
                    Request::Commit { id: binding.id },
                ]
                .into_iter()
                .enumerate()
                {
                    let mut operation = endpoint.operation(cancel.clone());
                    operation
                        .submit_publication(binding.clone(), request)
                        .map_err(|e| format!("setup submit {number}: {e:?}"))?;
                    let mut response = None;
                    for _ in 0..6 {
                        match operation
                            .collect_paced()
                            .map_err(|e| format!("setup collect {number}: {e:?}"))?
                        {
                            Collection::Complete(value) => {
                                response = Some(value);
                                break;
                            }
                            Collection::Busy => (),
                        }
                    }
                    observed0
                        .recv_timeout(tls::CYCLE_LIMIT)
                        .map_err(|_| format!("setup work notification {number} unavailable"))?;
                    if response
                        != Some(Response::Progress {
                            binding: binding.clone(),
                            offset: if number == 0 { 0 } else { 17 },
                            committed: number == 2,
                        })
                        || operation.test_attempts() > 8
                    {
                        return Err(format!("authenticated setup response {number} mismatch"));
                    }
                }
                // Each receiver is bounded at eight, and setup work is excluded
                // from the single recipient own-home notification observation.
                for receiver in [&observed0, &observed1] {
                    for _ in 0..8 {
                        if receiver.try_recv().is_err() {
                            break;
                        }
                    }
                }
                let before = [h.bytes(0), h.bytes(1)];
                let recipient = &h.pair.homes[1];
                let endpoint = carrier_endpoint(
                    recipient,
                    &recipient.device,
                    recipient.callback.local_addr().unwrap(),
                );
                let t0 = Instant::now();
                let parent = t0 + tls::CYCLE_LIMIT;
                let client_binding = binding.clone();
                let client_cancel = cancel.clone();
                let caller = scope.spawn(move || -> Result<_, String> {
                    let mut operation = endpoint
                        .operation_until(parent, client_cancel)
                        .map_err(|e| format!("own-home operation: {e:?}"))?;
                    let submit_start = t0.elapsed();
                    let submitted = operation.submit_own_home(
                        client_binding.clone(),
                        Request::GetChunk {
                            id: client_binding.id,
                            offset: 0,
                        },
                    );
                    let submit_end = t0.elapsed();
                    let mut result = submitted.map(|_| None);
                    let mut traces = [None; 6];
                    if result.is_ok() {
                        for slot in &mut traces {
                            let start = t0.elapsed();
                            let collected = operation.collect_paced();
                            let end = t0.elapsed();
                            let outcome = match &collected {
                                Ok(Collection::Busy) => "busy",
                                Ok(Collection::Complete(_)) => "complete",
                                Err(tls::Stage::Deadline) => "deadline",
                                Err(tls::Stage::Limit) => "limit",
                                Err(tls::Stage::Denied) => "denied",
                                Err(_) => "other-error",
                            };
                            *slot = Some(CollectTrace {
                                start,
                                end,
                                outcome,
                                snapshot: operation.diagnostic_snapshot(),
                            });
                            match collected {
                                Ok(Collection::Complete(value)) => {
                                    result = Ok(Some(value));
                                    break;
                                }
                                Ok(Collection::Busy) => (),
                                Err(error) => {
                                    result = Err(error);
                                    break;
                                }
                            }
                        }
                    }
                    Ok((
                        result,
                        traces,
                        submit_start,
                        submit_end,
                        operation.test_attempts(),
                        operation.diagnostic_snapshot(),
                    ))
                });
                // before_worker_exit sends after actual Work.execute returns and
                // Shared.finished is set; this receive time is only an upper bound.
                let ready = observed1
                    .recv_timeout(tls::CYCLE_LIMIT + Duration::from_secs(1))
                    .ok()
                    .map(|_| t0.elapsed());
                let caller = caller
                    .join()
                    .map_err(|_| "own-home caller panicked".to_string())?;
                let (fetched, traces, submit_start, submit_end, attempts, snapshot) = caller?;
                eprintln!(
                    "own-home submit_start_us={} submit_end_us={} ready_notification_us={} ready_observed={}",
                    submit_start.as_micros(),
                    submit_end.as_micros(),
                    ready.map_or(0, |t| t.as_micros()),
                    ready.is_some()
                );
                for (i, trace) in traces.iter().flatten().enumerate() {
                    let snapshot = trace.snapshot.unwrap_or_default();
                    let phase = match snapshot.phase {
                        DiagnosticPhase::CollectWait => "collect_wait",
                        DiagnosticPhase::CollectConnect => "collect_connect",
                        DiagnosticPhase::CollectExchange => "collect_exchange",
                        DiagnosticPhase::Idle => "idle",
                        _ => "other",
                    };
                    eprintln!(
                        "own-home collect={} start_us={} end_us={} outcome={} phase={phase} attempts={} connections={} busy={} planned_wait_us={} actual_wait_us={}",
                        i + 1,
                        trace.start.as_micros(),
                        trace.end.as_micros(),
                        trace.outcome,
                        snapshot.attempted,
                        snapshot.connections,
                        snapshot.busy,
                        snapshot.timings.planned_wait_us,
                        snapshot.timings.actual_wait_us
                    );
                }
                let after = [h.bytes(0), h.bytes(1)];
                eprintln!(
                    "own-home store0_changed={} store1_changed={} attempts={attempts} ready_charge0={} ready_charge1={}",
                    sha256(&before[0]) != sha256(&after[0]),
                    sha256(&before[1]) != sha256(&after[1]),
                    h.pair.homes[0].charged(),
                    recipient.charged()
                );
                let snapshot =
                    snapshot.ok_or_else(|| "own-home diagnostic unavailable".to_string())?;
                if ready.is_none()
                    || attempts > 8
                    || snapshot.attempted as usize != attempts
                    || snapshot.connections > 8
                    || h.pair.homes.iter().any(|h| h.charged() > 2)
                {
                    return Err(
                        "own-home readiness, original attempts or reservation bound failed".into(),
                    );
                }
                let value = fetched
                    .map_err(|e| format!("observed own-home transport: {e:?}"))?
                    .ok_or_else(|| {
                        "observed own-home six collect bound without result".to_string()
                    })?;
                // Progress during legitimate Commit checked the exact full binding.
                // The source-bound Get response carries offset/total/ciphertext.
                if value
                    != (Response::Chunk {
                        offset: 0,
                        bytes: ciphertext.to_vec(),
                        total: 17,
                    })
                    || sha256(&ciphertext) != binding.id
                    || !after[0].windows(ciphertext.len()).any(|b| b == ciphertext)
                {
                    return Err("exact committed ciphertext response or integrity mismatch".into());
                }
                eprintln!(
                    "own-home exact_response=true ciphertext_bytes=17 committed_integrity=true"
                );
                Ok(())
            })();
            // Capture before assertions/Drop, including setup failures. The Stop
            // guard also runs before scoped joins on every unwind path.
            capture_carrier_optional(&h, "own-home-readiness-final");
            drop(stop);
            let stopped = Instant::now();
            let [left, right] = loops;
            let loops = [left.join(), right.join()];
            (result, loops, stopped.elapsed())
        });
        let cleanup =
            h.pair.homes.iter().all(|home| {
                home.charged() == 0 && !home.runtime().state.active.load(Ordering::SeqCst)
            });
        capture_carrier_optional(&h, "own-home-readiness-drained");
        eprintln!(
            "own-home loops_joined=true drain_us={} cleanup={cleanup}",
            drain.as_micros()
        );
        let loops_ok = loops.into_iter().all(|r| matches!(r, Ok(Ok(Ok(())))));
        drop(routes);
        drop(servers);
        drop(stores);
        let root = h.pair.root.clone();
        drop(h);
        if !cleanup || !loops_ok || drain >= Duration::from_secs(5) || root.exists() {
            return Err("maintained-loop, worker or owned fixture cleanup failed".into());
        }
        result
    }
    let result = observe();
    assert!(
        result.is_ok(),
        "actual own-home observation stopped: {result:?}"
    );
}

fn performance_hooks(returned: mpsc::SyncSender<()>) -> composed_loop::Hooks {
    let (_, worker_exit) = mpsc::sync_channel(1);
    let (cleanup_seen, _) = mpsc::sync_channel(1);
    let (physical_finished, _) = mpsc::sync_channel(1);
    let (_, resume_admission) = mpsc::sync_channel(1);
    composed_loop::Hooks {
        dispatch_delay: AtomicBool::new(false),
        worker_once: AtomicBool::new(false),
        cleanup_once: AtomicBool::new(false),
        work_returned: returned,
        worker_exit: Mutex::new(worker_exit),
        cleanup_seen,
        physical_finished,
        resume_admission: Mutex::new(resume_admission),
        check_pending: |_| (),
    }
}

#[test]
fn performance_dispatch_delay_is_opt_in_fixed_and_cancellation_preserving() {
    let hooks = performance_hooks(mpsc::sync_channel(8).0);
    let cancel = AtomicBool::new(false);
    let calls = std::cell::Cell::new(0);
    assert!(hooks.before_dispatch_with(&cancel, |_| calls.set(calls.get() + 1)));
    assert_eq!(calls.get(), 0, "disabled delay must not invoke a wait");
    hooks.dispatch_delay.store(true, Ordering::SeqCst);
    assert!(hooks.before_dispatch_with(&cancel, |duration| {
        assert_eq!(duration, Duration::from_millis(20));
        calls.set(calls.get() + 1);
    }));
    assert_eq!(calls.get(), 1);
    assert!(!hooks.before_dispatch_with(&cancel, |_| cancel.store(true, Ordering::SeqCst)));
    assert!(!hooks.before_dispatch_with(&cancel, |_| panic!("already cancelled hook waited")));
    cancel.store(false, Ordering::SeqCst);
    let began = Instant::now();
    assert!(hooks.before_dispatch(&cancel));
    assert!(
        began.elapsed() >= Duration::from_millis(20),
        "actual enabled hook must wait the frozen20ms"
    );
    cancel.store(true, Ordering::SeqCst);
    assert!(!hooks.before_dispatch(&cancel));
}

// Frozen experiment overlay: this is a transport workload, not an SDK or UI run.
fn frozen_dispatch_performance_workload(maximum: bool) {
    use crate::delivery::{CarrierEndpoint, DiagnosticTimings, OperationSnapshot};
    struct Stop<'a> {
        runtimes: [&'a Runtime; 2],
        cancel: Arc<AtomicBool>,
    }
    impl Drop for Stop<'_> {
        fn drop(&mut self) {
            self.cancel.store(true, Ordering::SeqCst);
            for runtime in self.runtimes {
                runtime.stop();
            }
        }
    }
    #[derive(Default)]
    struct Counters {
        logical_started: usize,
        logical_completed: usize,
        attempts: usize,
        connections: usize,
        busy: usize,
        last: Option<OperationSnapshot>,
        timings: DiagnosticTimings,
    }
    fn request(
        endpoint: &CarrierEndpoint,
        binding: &Binding,
        request: Request,
        parent: Instant,
        cancel: Arc<AtomicBool>,
        counters: &mut Counters,
    ) -> Result<Response, tls::Stage> {
        counters.logical_started += 1;
        let mut operation = endpoint.operation_until(parent, cancel)?;
        let result = (|| {
            operation.submit_own_home(binding.clone(), request)?;
            for _ in 0..6 {
                match operation.collect_paced()? {
                    Collection::Complete(response) => return Ok(response),
                    Collection::Busy => (),
                }
            }
            Err(tls::Stage::Limit)
        })();
        if let Some(snapshot) = operation.diagnostic_snapshot() {
            counters.attempts += operation.test_attempts();
            counters.connections += usize::from(snapshot.connections);
            counters.busy += usize::from(snapshot.busy);
            counters.timings = counters.timings.saturating_add(snapshot.timings);
            counters.last = Some(snapshot);
            if operation.test_attempts() > 8 || snapshot.attempted > 8 {
                return Err(tls::Stage::Limit);
            }
        }
        if result.is_ok() {
            counters.logical_completed += 1;
        }
        result
    }
    fn observe(maximum: bool) -> Result<(), String> {
        let h = Homes::new(true);
        let stores = [h.empty(0), h.empty(1)];
        let size = if maximum { 1_048_592 } else { 17 };
        let ciphertext: Vec<u8> = (0..size).map(|i| (i % 251) as u8).collect();
        let mut binding = h.pair.binding(0);
        binding.id = sha256(&ciphertext);
        binding.size = size as u64;
        capture_carrier_optional(&h, "performance-early");
        for i in 0..2 {
            retain_synthetic_fixture(
                &format!("performance-{}-policy{i}.snapshot", std::process::id()),
                &fs::read(
                    h.pair
                        .root
                        .join(format!("policy{i}/provider-policy.snapshot")),
                )
                .map_err(|e| e.to_string())?,
            );
        }
        retain_synthetic_fixture(
            &format!("performance-{}-ciphertext.bin", std::process::id()),
            &ciphertext,
        );
        retain_synthetic_fixture(
            &format!("performance-{}-binding.bin", std::process::id()),
            &faction_files::private_blobs::encode_request(&Request::Begin(binding.clone()))
                .map_err(|e| format!("binding encoding: {e:?}"))?,
        );
        let server = |i: usize| {
            let home = &h.pair.homes[i];
            let foreign = &h.pair.homes[1 - i];
            Server::new_composed(
                &home.server.0,
                &home.server.1,
                &[home.ca.0.clone(), foreign.ca.0.clone()],
            )
            .map_err(|e| format!("composed server: {e:?}"))
        };
        let servers = [server(0)?, server(1)?];
        let profile = |i: usize| {
            PublicationWireProfile::new(h.pair.homes[i].name.into(), vec!["observer".into()])
                .map_err(|e| format!("publication profile: {e:?}"))
        };
        let profiles = [profile(0)?, profile(1)?];
        let routes = |i: usize| {
            let home = &h.pair.homes[i];
            let foreign = &h.pair.homes[1 - i];
            RelayRoutes::new(
                home.runtime(),
                carrier_endpoint(home, &home.relay, home.callback.local_addr().unwrap()),
                vec![RelayPeer {
                    home: foreign.name.into(),
                    publication: h.endpoint(i),
                    introspection: home.endpoint(foreign),
                    resource: carrier_endpoint(
                        foreign,
                        &home.relay,
                        foreign.callback.local_addr().unwrap(),
                    ),
                }],
            )
            .map_err(|e| format!("relay routes: {e:?}"))
        };
        let routes = [routes(0)?, routes(1)?];
        let (returned0, observed0) = mpsc::sync_channel(8);
        let (returned1, observed1) = mpsc::sync_channel(8);
        let hooks = [performance_hooks(returned0), performance_hooks(returned1)];
        let cancel = Arc::new(AtomicBool::new(false));
        let (result, loops, drain) = thread::scope(|scope| {
            let stop = Stop {
                runtimes: [h.pair.homes[0].runtime(), h.pair.homes[1].runtime()],
                cancel: cancel.clone(),
            };
            let loops: [_; 2] = std::array::from_fn(|i| {
                let home = &h.pair.homes[i];
                let server = &servers[i];
                let routes = &routes[i];
                let store = &stores[i];
                let profile = &profiles[i];
                let hooks = &hooks[i];
                let cancel = &cancel;
                let listener = home.callback.try_clone().unwrap();
                scope.spawn(move || {
                    thread::scope(|workers| {
                        composed_loop::run(
                            workers,
                            composed_loop::Inputs {
                                server,
                                listener,
                                runtime: home.runtime(),
                                cancel,
                                routes,
                                store,
                                home: home.name,
                                profile,
                                port: 0,
                            },
                            Some(hooks),
                        )
                    })
                })
            });
            let mut counters = Counters::default();
            let mut offset = 0usize;
            let mut cycles = 0usize;
            let mut statuses = 0usize;
            let mut gets = 0usize;
            let mut last_progress = Duration::ZERO;
            let mut elapsed = Duration::ZERO;
            let result = (|| {
                let sender = &h.pair.homes[0];
                let endpoint = carrier_endpoint(
                    sender,
                    &sender.device,
                    sender.callback.local_addr().unwrap(),
                );
                let setup = std::iter::once(Request::Begin(binding.clone()))
                    .chain(ciphertext.chunks(4096).enumerate().map(|(i, bytes)| {
                        Request::PutChunk {
                            id: binding.id,
                            offset: (i * 4096) as u64,
                            bytes: bytes.to_vec(),
                        }
                    }))
                    .chain(std::iter::once(Request::Commit { id: binding.id }));
                let mut setup_calls = 0;
                for request in setup {
                    let (expected_offset, committed) = match &request {
                        Request::Begin(_) => (0, false),
                        Request::PutChunk { offset, bytes, .. } => {
                            (*offset + bytes.len() as u64, false)
                        }
                        Request::Commit { .. } => (size as u64, true),
                        _ => return Err("unexpected setup request".into()),
                    };
                    if hooks
                        .iter()
                        .any(|h| h.dispatch_delay.load(Ordering::SeqCst))
                    {
                        return Err("dispatch delay was enabled during setup".into());
                    }
                    let mut operation = endpoint.operation(cancel.clone());
                    operation
                        .submit_publication(binding.clone(), request)
                        .map_err(|e| format!("setup submit {setup_calls}: {e:?}"))?;
                    let mut response = None;
                    for _ in 0..6 {
                        match operation
                            .collect_paced()
                            .map_err(|e| format!("setup collect {setup_calls}: {e:?}"))?
                        {
                            Collection::Complete(value) => {
                                response = Some(value);
                                break;
                            }
                            Collection::Busy => (),
                        }
                    }
                    observed0
                        .recv_timeout(tls::CYCLE_LIMIT)
                        .map_err(|_| "setup work notification unavailable".to_string())?;
                    if response
                        != Some(Response::Progress {
                            binding: binding.clone(),
                            offset: expected_offset,
                            committed,
                        })
                        || operation.test_attempts() > 8
                    {
                        return Err("exact authenticated publication setup failed".into());
                    }
                    setup_calls += 1;
                }
                let expected_setup = if maximum { 259 } else { 3 };
                if setup_calls != expected_setup {
                    return Err("setup operation count mismatch".into());
                }
                capture_carrier_optional(&h, "performance-committed");
                for receiver in [&observed0, &observed1] {
                    for _ in 0..8 {
                        if receiver.try_recv().is_err() {
                            break;
                        }
                    }
                }
                for hook in &hooks {
                    hook.dispatch_delay.store(true, Ordering::SeqCst);
                }
                let recipient = &h.pair.homes[1];
                let endpoint = carrier_endpoint(
                    recipient,
                    &recipient.device,
                    recipient.callback.local_addr().unwrap(),
                );
                let t0 = Instant::now();
                let global = t0 + Duration::from_secs(60);
                let expected_cycles = if maximum { 65 } else { 1 };
                eprintln!(
                    "performance fetch_start=true bytes={size} setup_calls={setup_calls} delay_ms=20 delay_enabled=true global_seconds=60 cycle_seconds=10"
                );
                let fetched = (|| -> Result<(), String> {
                    for cycle in 0..expected_cycles {
                        let cycle_deadline = (Instant::now() + tls::CYCLE_LIMIT).min(global);
                        statuses += 1;
                        let status = request(
                            &endpoint,
                            &binding,
                            Request::Status { id: binding.id },
                            cycle_deadline,
                            cancel.clone(),
                            &mut counters,
                        )
                        .map_err(|e| format!("status {e:?}"))?;
                        if status
                            != (Response::Progress {
                                binding: binding.clone(),
                                offset: size as u64,
                                committed: true,
                            })
                        {
                            return Err("exact committed status mismatch".into());
                        }
                        for _ in 0..4 {
                            if offset == size {
                                break;
                            }
                            gets += 1;
                            let end = (offset + 4096).min(size);
                            let response = request(
                                &endpoint,
                                &binding,
                                Request::GetChunk {
                                    id: binding.id,
                                    offset: offset as u64,
                                },
                                cycle_deadline,
                                cancel.clone(),
                                &mut counters,
                            )
                            .map_err(|e| format!("get {e:?}"))?;
                            if response
                                != (Response::Chunk {
                                    offset: offset as u64,
                                    bytes: ciphertext[offset..end].to_vec(),
                                    total: size as u64,
                                })
                            {
                                return Err("exact chunk offset, bytes or total mismatch".into());
                            }
                            offset = end;
                            last_progress = t0.elapsed();
                            for receiver in [&observed0, &observed1] {
                                for _ in 0..8 {
                                    if receiver.try_recv().is_err() {
                                        break;
                                    }
                                }
                            }
                        }
                        cycles = cycle + 1;
                        eprintln!(
                            "performance cycle={cycles} offset={offset} elapsed_us={} statuses={statuses} gets={gets} logical_completed={} attempts={} connections={} busy={}",
                            t0.elapsed().as_micros(),
                            counters.logical_completed,
                            counters.attempts,
                            counters.connections,
                            counters.busy
                        );
                    }
                    if offset != size
                        || statuses != expected_cycles
                        || gets != if maximum { 257 } else { 1 }
                        || cycles != expected_cycles
                        || sha256(&ciphertext) != binding.id
                    {
                        return Err("complete workload count or integrity mismatch".into());
                    }
                    if Instant::now() >= global {
                        return Err("complete but late after exact verification".into());
                    }
                    Ok(())
                })();
                elapsed = t0.elapsed();
                let snapshot = counters.last.unwrap_or_default();
                eprintln!(
                    "performance result={} global_expired={} offset={offset} elapsed_us={} last_progress_us={} cycles={cycles} statuses={statuses} gets={gets} logical_started={} logical_completed={} attempts={} connections={} busy={} phase={:?} last_operation_attempts={} last_operation_connections={} planned_wait_us={} actual_wait_us={} delay0={} delay1={}",
                    match &fetched {
                        Ok(()) => "complete",
                        Err(reason) if reason == "complete but late after exact verification" =>
                            "complete_late",
                        Err(_) if offset == size => "verification_error",
                        Err(_) => "incomplete",
                    },
                    Instant::now() >= global,
                    elapsed.as_micros(),
                    last_progress.as_micros(),
                    counters.logical_started,
                    counters.logical_completed,
                    counters.attempts,
                    counters.connections,
                    counters.busy,
                    snapshot.phase,
                    snapshot.attempted,
                    snapshot.connections,
                    counters.timings.planned_wait_us,
                    counters.timings.actual_wait_us,
                    hooks[0].dispatch_delay.load(Ordering::SeqCst),
                    hooks[1].dispatch_delay.load(Ordering::SeqCst)
                );
                if h.pair.homes.iter().any(|h| h.charged() > 2) {
                    return Err("reservation bound failed".into());
                }
                fetched
            })();
            capture_carrier_optional(&h, "performance-final");
            eprintln!(
                "performance outcome={result:?} offset={offset} observed_us={} maxfile={maximum}",
                elapsed.as_micros()
            );
            drop(stop);
            let stopped = Instant::now();
            let [left, right] = loops;
            let loops = [left.join(), right.join()];
            (result, loops, stopped.elapsed())
        });
        let cleanup =
            h.pair.homes.iter().all(|home| {
                home.charged() == 0 && !home.runtime().state.active.load(Ordering::SeqCst)
            });
        capture_carrier_optional(&h, "performance-drained");
        let loops_ok = loops.into_iter().all(|r| matches!(r, Ok(Ok(Ok(())))));
        eprintln!(
            "performance loops_joined=true cleanup={cleanup} loops_ok={loops_ok} drain_us={}",
            drain.as_micros()
        );
        drop(routes);
        drop(servers);
        drop(stores);
        let root = h.pair.root.clone();
        drop(h);
        if !cleanup || !loops_ok || drain >= Duration::from_secs(5) || root.exists() {
            return Err("performance fixture or bounded worker drain failed".into());
        }
        result
    }
    let result = observe(maximum);
    assert!(
        result.is_ok(),
        "frozen performance envelope stopped: {result:?}"
    );
}

#[test]
fn frozen_dispatch_envelope_authenticated_seventeen_byte_smoke() {
    frozen_dispatch_performance_workload(false);
}
#[test]
fn frozen_dispatch_envelope_maximum_fetch_with_original_foreground_deadline() {
    frozen_dispatch_performance_workload(true);
}

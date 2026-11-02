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
        lifecycle: Some(composed_loop::LifecycleHooks {
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
        }),
        attach: None,
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
fn collection_trace_controls_disabled_enabled_order_privacy_and_overflow() {
    use crate::delivery::collection_trace::{Outcome, Role, Trace, WireKind};
    let trace = Trace::new(std::time::Instant::now());
    assert!(trace.validated(Role::Outer, WireKind::Collect).is_none());
    assert!(trace.snapshot().points.is_empty());
    let attached_trace = trace.clone();
    let hooks = composed_loop::Hooks {
        lifecycle: None,
        attach: Some(Box::new(move |delivery| {
            delivery.test_attach_collection_trace(attached_trace.clone())
        })),
    };
    assert!(hooks.lifecycle.is_none());
    trace.arm();
    trace.attached(Role::Outer);
    trace.issue_success();
    let collect = trace.validated(Role::Outer, WireKind::Collect).unwrap();
    collect.outcome(Outcome::Busy);
    trace.finish_before(Role::Outer, true);
    trace.finish_after(Role::Outer, true);
    let snapshot = trace.snapshot();
    assert!(!snapshot.inconclusive);
    assert_eq!(snapshot.points.len(), 5);
}

#[test]
#[ignore = "one source-qualified characterization requires separate runtime release"]
fn characterize_one_small_get_through_both_maintained_loops() {
    use crate::delivery::{
        OperationSnapshot,
        collection_trace::{Event as TraceEvent, Role, Trace, WireKind},
    };
    use std::sync::OnceLock;
    #[derive(Clone, Copy, Debug)]
    enum CallOutcome {
        Submitted,
        Busy,
        Result,
        Deadline,
        Limit,
        Denied,
        OtherError,
    }
    #[derive(Clone, Copy, Debug)]
    struct Call {
        start_us: u128,
        end_us: u128,
        outcome: CallOutcome,
        diagnostic: Option<OperationSnapshot>,
    }
    fn error_outcome(stage: tls::Stage) -> CallOutcome {
        match stage {
            tls::Stage::Deadline => CallOutcome::Deadline,
            tls::Stage::Limit => CallOutcome::Limit,
            tls::Stage::Denied => CallOutcome::Denied,
            _ => CallOutcome::OtherError,
        }
    }
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
    fn observe() -> Result<(), String> {
        let h = Homes::new(true);
        let stores = [h.empty(0), h.empty(1)];
        let ciphertext = [37; 17];
        let mut binding = h.pair.binding(0);
        binding.id = sha256(&ciphertext);
        binding.size = ciphertext.len() as u64;
        let setup = [
            Request::Begin(binding.clone()),
            Request::PutChunk {
                id: binding.id,
                offset: 0,
                bytes: ciphertext.to_vec(),
            },
            Request::Commit { id: binding.id },
        ];
        let get = Request::GetChunk {
            id: binding.id,
            offset: 0,
        };
        capture_carrier_optional(&h, "small-get-early");
        for i in 0..2 {
            retain_synthetic_fixture(
                &format!("small-get-policy{i}.snapshot"),
                &fs::read(
                    h.pair
                        .root
                        .join(format!("policy{i}/provider-policy.snapshot")),
                )
                .map_err(|_| "initial policy capture unavailable")?,
            );
            retain_synthetic_fixture(&format!("small-get-initial-store{i}.snapshot"), &h.bytes(i));
        }
        for (i, request) in setup.iter().chain(std::iter::once(&get)).enumerate() {
            retain_synthetic_fixture(
                &format!("small-get-request-{i}.bin"),
                &faction_files::private_blobs::encode_request(request)
                    .map_err(|_| "request encoding unavailable")?,
            );
        }
        retain_synthetic_fixture("small-get-ciphertext.bin", &ciphertext);
        retain_synthetic_fixture("small-get-private-input-coverage.txt",b"actual_issued_token_retained=false\nreason=no_existing_bounded_private_token_witness\nin_memory_policy_clock_state_captured=false\n");
        let server = |i: usize| {
            let home = &h.pair.homes[i];
            let foreign = &h.pair.homes[1 - i];
            Server::new_composed(
                &home.server.0,
                &home.server.1,
                &[home.ca.0.clone(), foreign.ca.0.clone()],
            )
            .map_err(|_| "composed server configuration failed")
        };
        let servers = [server(0)?, server(1)?];
        let profile = |i: usize| {
            PublicationWireProfile::new(h.pair.homes[i].name.into(), vec!["observer".into()])
                .map_err(|_| "publication profile failed")
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
            .map_err(|_| "relay routes failed")
        };
        let routes = [routes(0)?, routes(1)?];
        let holder = Arc::new(OnceLock::<Arc<Trace>>::new());
        let hooks: [_; 2] = std::array::from_fn(|_| {
            let holder = holder.clone();
            composed_loop::Hooks {
                lifecycle: None,
                attach: Some(Box::new(move |delivery| {
                    if let Some(trace) = holder.get() {
                        delivery.test_attach_collection_trace(trace.clone());
                    }
                })),
            }
        });
        let cancel = Arc::new(AtomicBool::new(false));
        let mut calls: [Option<Call>; 7] = [None; 7];
        let mut fetched = None;
        let mut attempts = 0;
        let mut diagnostic = None;
        let mut collect_frame = None;
        let mut stores_changed = [false; 2];
        let (attempted, loops, drain) = thread::scope(|scope| {
            // This guard cancels before the scope's implicit joins on unwind.
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
            let attempted = std::panic::catch_unwind(std::panic::AssertUnwindSafe(
                || -> Result<(), String> {
                    let sender = &h.pair.homes[0];
                    let endpoint = carrier_endpoint(
                        sender,
                        &sender.device,
                        sender.callback.local_addr().unwrap(),
                    );
                    for (number, request) in setup.into_iter().enumerate() {
                        let mut operation = endpoint.operation(cancel.clone());
                        let submitted = operation.submit_publication(binding.clone(), request);
                        if submitted.is_ok() {
                            metadata_capture_inputs(
                                &h,
                                &operation,
                                &format!("small-get-setup-{number}"),
                            );
                        }
                        submitted.map_err(|stage| format!("setup submit {number}: {stage:?}"))?;
                        let mut response = None;
                        let mut error = None;
                        let mut collects = 0;
                        for _ in 0..6 {
                            collects += 1;
                            match operation.collect_paced() {
                                Ok(Collection::Complete(value)) => {
                                    response = Some(value);
                                    break;
                                }
                                Ok(Collection::Busy) => (),
                                Err(stage) => {
                                    error = Some(stage);
                                    break;
                                }
                            }
                        }
                        if let Some(value) = &response {
                            retain_synthetic_fixture(
                                &format!("small-get-setup-response-{number}.bin"),
                                &faction_files::private_blobs::encode_response(value)
                                    .map_err(|_| "setup response encoding failed")?,
                            );
                        }
                        retain_synthetic_fixture(&format!("small-get-setup-outcome-{number}.txt"),format!("response_present={} error={error:?} collects={collects} attempts={} diagnostic={:?}\n",response.is_some(),operation.test_attempts(),operation.diagnostic_snapshot()).as_bytes());
                        capture_carrier_optional(&h, &format!("small-get-setup-after-{number}"));
                        if error.is_some()
                            || response
                                != Some(Response::Progress {
                                    binding: binding.clone(),
                                    offset: if number == 0 { 0 } else { 17 },
                                    committed: number == 2,
                                })
                            || operation.test_attempts() > 8
                        {
                            return Err(format!(
                                "exact setup response or original bound failed at {number}"
                            ));
                        }
                    }
                    // Exact setup responses and Store bytes precede arming. No idle
                    // wait: accept_composed may reserve active while listening.
                    let before = [h.bytes(0), h.bytes(1)];
                    for (i, bytes) in before.iter().enumerate() {
                        retain_synthetic_fixture(
                            &format!("small-get-committed-store{i}.snapshot"),
                            bytes,
                        );
                    }
                    let recipient = &h.pair.homes[1];
                    let endpoint = carrier_endpoint(
                        recipient,
                        &recipient.device,
                        recipient.callback.local_addr().unwrap(),
                    );
                    let t0 = Instant::now();
                    let trace = Trace::new(t0);
                    trace.arm();
                    holder
                        .set(trace)
                        .map_err(|_| "trace origin initialized twice")?;
                    let mut operation = endpoint
                        .operation_until(t0 + Duration::from_secs(60), cancel.clone())
                        .map_err(|stage| format!("measured operation construction: {stage:?}"))?;
                    let start = t0.elapsed();
                    let submitted = operation.submit_own_home(binding.clone(), get);
                    calls[0] = Some(Call {
                        start_us: start.as_micros(),
                        end_us: t0.elapsed().as_micros(),
                        outcome: match submitted {
                            Ok(()) => CallOutcome::Submitted,
                            Err(stage) => error_outcome(stage),
                        },
                        diagnostic: operation.diagnostic_snapshot(),
                    });
                    let mut result = submitted.map(|()| None);
                    if result.is_ok() {
                        // Retain the actual source/correlation frame in this bounded
                        // private input slot; flush it after measured calls finish.
                        collect_frame = Some(operation.test_collect_frame());
                        for slot in &mut calls[1..] {
                            let start = t0.elapsed();
                            let collected = operation.collect_paced();
                            let end = t0.elapsed();
                            let outcome = match &collected {
                                Ok(Collection::Busy) => CallOutcome::Busy,
                                Ok(Collection::Complete(_)) => CallOutcome::Result,
                                Err(stage) => error_outcome(*stage),
                            };
                            *slot = Some(Call {
                                start_us: start.as_micros(),
                                end_us: end.as_micros(),
                                outcome,
                                diagnostic: operation.diagnostic_snapshot(),
                            });
                            match collected {
                                Ok(Collection::Busy) => (),
                                Ok(Collection::Complete(value)) => {
                                    result = Ok(Some(value));
                                    break;
                                }
                                Err(stage) => {
                                    result = Err(stage);
                                    break;
                                }
                            }
                        }
                    }
                    attempts = operation.test_attempts();
                    diagnostic = operation.diagnostic_snapshot();
                    fetched = Some(result);
                    for i in 0..2 {
                        stores_changed[i] = sha256(&before[i]) != sha256(&h.bytes(i));
                    }
                    Ok(())
                },
            ));
            // Capture even after a setup/measurement error, before judgment/Drop.
            if let Some(bytes) = &collect_frame {
                retain_synthetic_fixture("small-get-actual-outer-collect-frame.bin", bytes);
            }
            if let Some(Ok(Some(value))) = &fetched {
                retain_synthetic_fixture(
                    "small-get-actual-response.bin",
                    &faction_files::private_blobs::encode_response(value).unwrap(),
                );
            }
            capture_carrier_optional(&h, "small-get-after-calls");
            let stopped = Instant::now(); // includes cancellation itself in drain.
            drop(stop);
            let [left, right] = loops;
            let loops = [left.join(), right.join()];
            (attempted, loops, stopped.elapsed())
        });
        let snapshot = holder.get().map(|trace| trace.snapshot()); // ONLY after joins.
        let loops_ok = loops
            .into_iter()
            .all(|result| matches!(result, Ok(Ok(Ok(())))));
        // stop invalidates logical/token slots; charged_len preserves a live
        // physical weak reservation, so zero now proves physical lease release.
        let physical_zero = h.pair.homes.iter().all(|home| home.charged() == 0);
        let inactive = h
            .pair
            .homes
            .iter()
            .all(|home| !home.runtime().state.active.load(Ordering::SeqCst));
        let exact = matches!(&fetched,Some(Ok(Some(Response::Chunk {offset,bytes,total}))) if *offset==0 && *total==17 && bytes.as_slice()==ciphertext.as_slice());
        let mut role_counts = [(0, 0, 0); 2];
        let mut expected_kinds = true;
        if let Some(snapshot) = &snapshot {
            for point in &snapshot.points {
                let r = point.role as usize;
                match point.event {
                    TraceEvent::Validated => {
                        role_counts[r].0 += 1;
                        expected_kinds &= point.kind
                            == if point.role == Role::Outer {
                                WireKind::Collect
                            } else {
                                WireKind::Wait
                            };
                    }
                    TraceEvent::FinishBefore => role_counts[r].1 += 1,
                    TraceEvent::FinishAfter => role_counts[r].2 += 1,
                    _ => (),
                }
            }
        }
        let both_roles = role_counts.iter().all(|&(collects, before, after)| {
            collects > 0 && collects <= 8 && before == 1 && after == 1
        });
        let mut overlap = false;
        let mut busy_before = 0;
        let mut busy_after = 0;
        let mut busy_count = 0;
        if let Some(snapshot) = &snapshot {
            let before = snapshot
                .points
                .iter()
                .find(|p| p.role == Role::Outer && p.event == TraceEvent::FinishBefore);
            let after = snapshot
                .points
                .iter()
                .find(|p| p.role == Role::Outer && p.event == TraceEvent::FinishAfter);
            for outcome in snapshot.points.iter().filter(|p| {
                p.role == Role::Outer
                    && p.event == TraceEvent::Outcome
                    && p.outcome == crate::delivery::collection_trace::Outcome::Busy
            }) {
                busy_count += 1;
                let start = snapshot.points.iter().find(|p| {
                    p.role == Role::Outer
                        && p.attempt == outcome.attempt
                        && p.event == TraceEvent::Validated
                });
                match (start, before, after) {
                    (Some(_), Some(before), Some(_)) if outcome.micros < before.micros => {
                        busy_before += 1
                    }
                    (Some(start), Some(_), Some(after)) if start.micros > after.micros => {
                        busy_after += 1
                    }
                    _ => overlap = true,
                }
            }
        }
        let trace_conclusive = snapshot.as_ref().is_some_and(|s| !s.inconclusive)
            && both_roles
            && expected_kinds
            && !overlap;
        let calls_ok = calls
            .iter()
            .flatten()
            .all(|call| call.start_us <= call.end_us && call.diagnostic.is_some())
            && calls[0].is_some_and(|call| matches!(call.outcome, CallOutcome::Submitted));
        retain_synthetic_fixture(
            "small-get-calls.txt",
            format!("{calls:?}\nfinal_diagnostic={diagnostic:?}\n").as_bytes(),
        );
        retain_synthetic_fixture(
            "small-get-postjoin-trace.txt",
            format!("{snapshot:?}\n").as_bytes(),
        );
        let transport = fetched.as_ref().map(|result| {
            result
                .as_ref()
                .map(|value| value.is_some())
                .map_err(|stage| *stage)
        });
        let setup_ok = matches!(&attempted, Ok(Ok(())));
        retain_synthetic_fixture("small-get-postjoin-status.txt",format!("setup_and_measurement_returned={setup_ok} transport={transport:?} exact={exact} attempts={attempts} role_counts={role_counts:?} expected_kinds={expected_kinds} trace_conclusive={trace_conclusive} timestamp_overlap={overlap} busy_count={busy_count} busy_before_finish={busy_before} busy_after_finish={busy_after} stores_changed={stores_changed:?} loops_ok={loops_ok} physical_zero={physical_zero} inactive={inactive} drain_us={} actual_issued_token_retained=false\n",drain.as_micros()).as_bytes());
        capture_carrier_optional(&h, "small-get-drained");
        drop(routes);
        drop(servers);
        drop(stores);
        let root = h.pair.root.clone();
        drop(h);
        let removed = !root.exists();
        retain_synthetic_fixture(
            "small-get-owned-cleanup.txt",
            format!("owned_runtime_root_removed={removed}\n").as_bytes(),
        );
        // Publish only explicit numeric fields after measurement, joins and
        // owned-root cleanup. Never forward the private Debug/raw capture files.
        if let Some(snapshot) = &snapshot {
            for point in &snapshot.points {
                eprintln!("{}", small_get_point_line(point));
            }
        }
        for (index, call) in calls.iter().flatten().enumerate() {
            eprintln!(
                "{}",
                small_get_call_line(
                    index,
                    call.start_us,
                    call.end_us,
                    call.outcome as u8,
                    call.diagnostic
                )
            );
        }
        let (response_bytes, response_offset, response_total) = match &fetched {
            Some(Ok(Some(Response::Chunk {
                bytes,
                offset,
                total,
            }))) => (bytes.len(), *offset, *total),
            _ => (0, 0, 0),
        };
        let transport_ok = matches!(&fetched, Some(Ok(Some(_))));
        eprintln!(
            "small_get_summary schema=1 points={} calls={} setup_ok={} transport_ok={} exact={} response_bytes={response_bytes} response_offset={response_offset} response_total={response_total} trace_conclusive={} timestamp_overlap={} busy_count={busy_count} busy_before_finish={busy_before} busy_after_finish={busy_after} loops_ok={} physical_zero={} inactive={} drain_us={} root_removed={} token_retained=0 clock_state_retained=0 attempts={attempts} connections={} outer_collects={} inner_collects={} outer_finish_before={} outer_finish_after={} inner_finish_before={} inner_finish_after={} expected_kinds={} calls_ok={}",
            snapshot.as_ref().map_or(0, |s| s.points.len()),
            calls.iter().flatten().count(),
            u8::from(setup_ok),
            u8::from(transport_ok),
            u8::from(exact),
            u8::from(trace_conclusive),
            u8::from(overlap),
            u8::from(loops_ok),
            u8::from(physical_zero),
            u8::from(inactive),
            drain.as_micros(),
            u8::from(removed),
            diagnostic.map_or(0, |d| d.connections),
            role_counts[0].0,
            role_counts[1].0,
            role_counts[0].1,
            role_counts[0].2,
            role_counts[1].1,
            role_counts[1].2,
            u8::from(expected_kinds),
            u8::from(calls_ok)
        );
        if !loops_ok || !physical_zero || !inactive || drain >= Duration::from_secs(5) || !removed {
            return Err("loop/worker/physical/owned-fixture cleanup failed".into());
        }
        match attempted {
            Ok(result) => result?,
            Err(_) => {
                return Err(
                    "setup or measured caller panicked; captures and drain completed".into(),
                );
            }
        }
        if !exact
            || sha256(&ciphertext) != binding.id
            || attempts > 8
            || !calls_ok
            || !diagnostic.is_some_and(|d| d.attempted as usize == attempts && d.connections <= 8)
        {
            return Err("exact source/response/transport/original operation bound failed".into());
        }
        if !trace_conclusive {
            return Err("missing role/event/request kind or conservative timestamp overlap; characterization inconclusive".into());
        }
        Ok(())
    }
    let observed = observe();
    assert!(
        observed.is_ok(),
        "small Get characterization stopped: {observed:?}"
    );
}

#[test]
fn small_get_numeric_formatter_contract() {
    use crate::delivery::collection_trace::{Event, Outcome, Point, Role, WireKind};
    let point = Point {
        sequence: 7,
        micros: 19,
        role: Role::Inner,
        kind: WireKind::Wait,
        attempt: 0,
        event: Event::Outcome,
        outcome: Outcome::Result,
        first_us: 0,
        last_us: 0,
    };
    let line = small_get_point_line(&point);
    assert_eq!(
        line,
        "small_get_point schema=1 sequence=7 micros=19 role=1 kind=6 attempt=0 event=4 outcome=2 first_us=0 last_us=0"
    );
    let snapshot = crate::delivery::OperationSnapshot {
        phase: crate::delivery::DiagnosticPhase::Idle,
        attempted: 1,
        connections: 1,
        ..Default::default()
    };
    let call = small_get_call_line(0, 1, 2, 0, Some(snapshot));
    assert_eq!(
        call,
        "small_get_call schema=1 index=0 start_us=1 end_us=2 outcome=0 diagnostic=1 phase=6 attempted=1 connections=1 busy=0 planned_wait_us=0 actual_wait_us=0"
    );
    for secret in [
        "PRIVATE_PKI_SENTINEL",
        "PRIVATE_TOKEN_SENTINEL",
        "PRIVATE_REQUEST_SENTINEL",
    ] {
        assert!(!line.contains(secret));
        assert!(!call.contains(secret));
    }
    assert!(line.split_whitespace().skip(1).all(|field| {
        field
            .split_once('=')
            .unwrap()
            .1
            .bytes()
            .all(|b| b.is_ascii_digit())
    }));
}

fn small_get_point_line(point: &crate::delivery::collection_trace::Point) -> String {
    use crate::delivery::collection_trace::WireKind;
    let kind = match point.kind {
        WireKind::None => 0,
        WireKind::Collect => 1,
        WireKind::Wait => 6,
    };
    format!(
        "small_get_point schema=1 sequence={} micros={} role={} kind={kind} attempt={} event={} outcome={} first_us={} last_us={}",
        point.sequence,
        point.micros,
        point.role as u8,
        point.attempt,
        point.event as u8,
        point.outcome as u8,
        point.first_us,
        point.last_us
    )
}
fn small_get_call_line(
    index: usize,
    start_us: u128,
    end_us: u128,
    outcome: u8,
    diagnostic: Option<crate::delivery::OperationSnapshot>,
) -> String {
    let present = diagnostic.is_some();
    let d = diagnostic.unwrap_or_default();
    format!(
        "small_get_call schema=1 index={index} start_us={start_us} end_us={end_us} outcome={outcome} diagnostic={} phase={} attempted={} connections={} busy={} planned_wait_us={} actual_wait_us={}",
        u8::from(present),
        d.phase as u8,
        d.attempted,
        d.connections,
        d.busy,
        d.timings.planned_wait_us,
        d.timings.actual_wait_us
    )
}

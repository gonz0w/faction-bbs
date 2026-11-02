use super::*;

#[test]
fn setup_trace_shared_connector_distinguishes_cold_hit_and_unpublished_work() {
    let trace = Trace::new(Instant::now());
    trace.arm();
    {
        let _scope = trace.scope(Role::OwnHome);
        for kind in [Kind::Binary, Kind::Http] {
            for outcome in [
                SetupOutcome::ColdPublished,
                SetupOutcome::Hit,
                SetupOutcome::ColdUnpublished,
            ] {
                let c = Connection::begin(kind).unwrap();
                if kind == Kind::Binary {
                    c.interval(Phase::Tcp, 0).finish();
                }
                c.interval(Phase::Lookup, 0)
                    .finish_as(if outcome == SetupOutcome::Hit {
                        outcome
                    } else {
                        SetupOutcome::None
                    });
                if outcome != SetupOutcome::Hit {
                    c.interval(Phase::Builder, 0).finish();
                    c.interval(Phase::Immutable, 0).finish();
                }
                if kind == Kind::Http {
                    c.interval(Phase::Tcp, 0).finish();
                }
                if outcome != SetupOutcome::Hit {
                    c.interval(Phase::Immutable, 1).finish();
                    c.interval(Phase::Publication, 0).finish_as(outcome);
                }
                c.interval(Phase::Ssl, 0).finish();
                c.interval(Phase::Postcheck, 0).finish();
            }
        }
    }
    let snapshot = trace.snapshot();
    assert!(!snapshot.inconclusive);
    assert_eq!(
        snapshot
            .points
            .iter()
            .filter(|p| p.outcome == SetupOutcome::Hit)
            .count(),
        2
    );
    assert_eq!(
        snapshot
            .points
            .iter()
            .filter(|p| p.phase == Phase::Builder)
            .count(),
        4
    );
    assert_eq!(
        snapshot
            .points
            .iter()
            .filter(|p| p.outcome == SetupOutcome::ColdPublished)
            .count(),
        2
    );
    assert_eq!(
        snapshot
            .points
            .iter()
            .filter(|p| p.outcome == SetupOutcome::ColdUnpublished)
            .count(),
        2
    );
}

fn binary() {
    let c = Connection::begin(Kind::Binary).unwrap();
    for phase in [Phase::Tcp, Phase::Lookup, Phase::Builder, Phase::Immutable] {
        Interval::finish_optional(Some(c.interval(phase, 0)));
    }
    c.interval(Phase::Immutable, 1).finish();
    c.interval(Phase::Publication, 0)
        .finish_as(SetupOutcome::ColdPublished);
    c.interval(Phase::Ssl, 0).finish();
    c.interval(Phase::Postcheck, 0).finish();
}

#[test]
fn setup_trace_shared_connector_rejects_contradictory_missing_and_incomplete_work() {
    for case in 0..8 {
        let trace = Trace::new(Instant::now());
        trace.arm();
        {
            let _scope = trace.scope(Role::OwnHome);
            let c = Connection::begin(Kind::Binary).unwrap();
            if case != 6 {
                c.interval(Phase::Tcp, 0).finish();
            }
            if case != 0 {
                let lookup = c.interval(Phase::Lookup, 0);
                if case == 5 {
                    drop(lookup);
                } else {
                    lookup.finish_as(if case == 1 {
                        SetupOutcome::Hit
                    } else {
                        SetupOutcome::None
                    });
                }
            }
            if case == 6 {
                c.interval(Phase::Tcp, 0).finish();
            }
            c.interval(Phase::Builder, 0).finish();
            c.interval(Phase::Immutable, 0).finish();
            if case != 2 {
                c.interval(Phase::Immutable, 1).finish();
            }
            if case != 4 {
                c.interval(Phase::Publication, 0).finish_as(if case == 3 {
                    SetupOutcome::Hit
                } else {
                    SetupOutcome::ColdPublished
                });
            }
            c.interval(Phase::Ssl, 0).finish_as(if case == 7 {
                SetupOutcome::ColdUnpublished
            } else {
                SetupOutcome::None
            });
            c.interval(Phase::Postcheck, 0).finish();
        }
        assert!(
            trace.snapshot().inconclusive,
            "invalid shared setup case {case}"
        );
    }
}
fn http() {
    let c = Connection::begin(Kind::Http).unwrap();
    for (phase, part) in [
        (Phase::Lookup, 0),
        (Phase::Builder, 0),
        (Phase::Immutable, 0),
        (Phase::Tcp, 0),
        (Phase::Immutable, 1),
    ] {
        c.interval(phase, part).finish();
    }
    c.interval(Phase::Publication, 0)
        .finish_as(SetupOutcome::ColdPublished);
    c.interval(Phase::Ssl, 0).finish();
    c.interval(Phase::Postcheck, 0).finish();
}
#[test]
fn setup_trace_disabled_scopes_cleanup_and_numeric_privacy() {
    assert!(Connection::begin(Kind::Binary).is_none());
    assert!(scope_is_clear());
    let trace = Trace::new(Instant::now());
    {
        let _scope = trace.scope(Role::Outer);
        assert!(Connection::begin(Kind::Binary).is_none());
    }
    assert!(trace.snapshot().points.is_empty());
    trace.arm();
    {
        let _scope = trace.scope(Role::Outer);
        binary();
    }
    assert!(Connection::begin(Kind::Binary).is_none());
    let snapshot = trace.snapshot();
    assert!(!snapshot.inconclusive);
    assert_eq!(snapshot.points.len(), 8);
    assert!(
        snapshot
            .points
            .iter()
            .all(|p| p.ordinal == 0 && p.start_us <= p.end_us && p.complete)
    );
    for point in snapshot.points {
        let line = point.line();
        assert!(line.split_whitespace().skip(1).all(|f| {
            f.split_once('=')
                .unwrap()
                .1
                .bytes()
                .all(|b| b.is_ascii_digit())
        }));
        for secret in [
            "PRIVATE_TOKEN_SENTINEL",
            "PRIVATE_ENDPOINT_SENTINEL",
            "PRIVATE_KEY_SENTINEL",
        ] {
            assert!(!line.contains(secret));
        }
    }
    let weak = Arc::downgrade(&trace);
    drop(trace);
    assert!(weak.upgrade().is_none());
}
#[test]
fn setup_trace_roles_order_ordinals_and_concurrent_recording() {
    let trace = Trace::new(Instant::now());
    trace.arm();
    std::thread::scope(|threads| {
        for role in [Role::Outer, Role::OwnHome, Role::Resource] {
            let trace = &trace;
            threads.spawn(move || {
                let _scope = trace.scope(role);
                match role {
                    Role::Outer => {
                        for _ in 0..2 {
                            binary();
                            Connection::begin(Kind::Paced)
                                .unwrap()
                                .interval(Phase::PacedWait, 0)
                                .finish();
                        }
                    }
                    Role::OwnHome => {
                        http();
                        binary();
                    }
                    Role::Resource => http(),
                }
            });
        }
    });
    let snapshot = trace.snapshot();
    assert!(!snapshot.inconclusive);
    assert_eq!(snapshot.points.len(), 42);
    assert!(
        snapshot
            .points
            .iter()
            .any(|p| p.role == Role::Outer && p.ordinal == 1)
    );
    assert!(scope_is_clear());
    let weak = Arc::downgrade(&trace);
    drop(trace);
    assert!(weak.upgrade().is_none());
}
#[test]
fn setup_trace_incomplete_order_role_and_ordinal_overflow_are_inconclusive() {
    for case in 0..5 {
        let trace = Trace::new(Instant::now());
        trace.arm();
        {
            let _scope = trace.scope(Role::Outer);
            match case {
                0 => {
                    let c = Connection::begin(Kind::Binary).unwrap();
                    drop(c.interval(Phase::Builder, 0));
                }
                1 => {
                    let c = Connection::begin(Kind::Binary).unwrap();
                    c.interval(Phase::Builder, 0).finish();
                }
                2 => {
                    assert!(Connection::begin(Kind::Http).is_none());
                }
                3 => {
                    for _ in 0..8 {
                        binary();
                    }
                    assert!(Connection::begin(Kind::Binary).is_none());
                }
                4 => {
                    let c = Connection::begin(Kind::Binary).unwrap();
                    for _ in 0..129 {
                        c.interval(Phase::Tcp, 0).finish();
                    }
                }
                _ => unreachable!(),
            }
        }
        let snapshot = trace.snapshot();
        assert!(snapshot.inconclusive, "case {case}");
        assert!(snapshot.points.len() <= CAPACITY);
    }
}
#[test]
fn setup_trace_empty_connection_and_nested_scope_fail_closed() {
    let trace = Trace::new(Instant::now());
    trace.arm();
    {
        let _scope = trace.scope(Role::Outer);
        drop(Connection::begin(Kind::Binary).unwrap());
    }
    assert!(trace.snapshot().inconclusive);
    let trace = Trace::new(Instant::now());
    trace.arm();
    {
        let _scope = trace.scope(Role::Outer);
        {
            let _nested = trace.scope(Role::Resource);
            http();
        }
        binary();
    }
    assert!(Connection::begin(Kind::Binary).is_none());
    assert!(trace.snapshot().inconclusive);
}

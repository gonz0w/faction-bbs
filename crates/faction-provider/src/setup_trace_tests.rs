use super::*;

fn binary() {
    let c = Connection::begin(Kind::Binary).unwrap();
    for phase in [
        Phase::Tcp,
        Phase::Builder,
        Phase::Immutable,
        Phase::Ssl,
        Phase::Postcheck,
    ] {
        Interval::finish_optional(Some(c.interval(phase, 0)));
    }
}
fn http() {
    let c = Connection::begin(Kind::Http).unwrap();
    for (phase, part) in [
        (Phase::Builder, 0),
        (Phase::Immutable, 0),
        (Phase::Tcp, 0),
        (Phase::Immutable, 1),
        (Phase::Ssl, 0),
        (Phase::Postcheck, 0),
    ] {
        c.interval(phase, part).finish();
    }
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
    assert_eq!(snapshot.points.len(), 5);
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
    assert_eq!(snapshot.points.len(), 29);
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

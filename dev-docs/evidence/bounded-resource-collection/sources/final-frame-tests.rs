//! Private exact-byte framing controls; no TCP packetization claim.
use super::{MAX_WIRE, Stage, write_frame};
use std::io::{self, Write};

struct Probe {
    attempted: Vec<Vec<u8>>,
    wire: Vec<u8>,
    max_write: usize,
    fail_after: Option<usize>,
    interrupt_once: bool,
    flush_error: bool,
    flushes: usize,
}
impl Probe {
    fn healthy() -> Self {
        Self {
            attempted: Vec::new(),
            wire: Vec::new(),
            max_write: usize::MAX,
            fail_after: None,
            interrupt_once: false,
            flush_error: false,
            flushes: 0,
        }
    }
}
impl Write for Probe {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.attempted.push(bytes.to_vec());
        if self.interrupt_once {
            self.interrupt_once = false;
            return Err(io::ErrorKind::Interrupted.into());
        }
        let remaining = match self.fail_after {
            Some(limit) if self.wire.len() >= limit => return Err(io::ErrorKind::BrokenPipe.into()),
            Some(limit) => limit - self.wire.len(),
            None => usize::MAX,
        };
        let accepted = bytes.len().min(self.max_write).min(remaining);
        self.wire.extend_from_slice(&bytes[..accepted]);
        Ok(accepted)
    }
    fn flush(&mut self) -> io::Result<()> {
        self.flushes += 1;
        if self.flush_error {
            Err(io::ErrorKind::BrokenPipe.into())
        } else {
            Ok(())
        }
    }
}
fn expected(payload: &[u8]) -> Vec<u8> {
    let mut wire = (payload.len() as u32).to_be_bytes().to_vec();
    wire.extend_from_slice(payload);
    wire
}
#[test]
fn exact_prefix_payload_and_final_flush_preserve_existing_wire_bytes() {
    for payload in [
        vec![0xa5],
        vec![0, 255, 3, 128, 9, 0, 42],
        vec![0x3c; MAX_WIRE],
    ] {
        let mut probe = Probe::healthy();
        assert_eq!(write_frame(&mut probe, &payload), Ok(()));
        assert_eq!(probe.wire, expected(&payload));
        assert_eq!(&probe.wire[..4], &(payload.len() as u32).to_be_bytes());
        assert_eq!(probe.flushes, 1);
        assert_eq!(probe.wire.len(), payload.len() + 4);
        assert!(probe.wire.len() <= 8196);
    }
}
#[test]
fn empty_and_oversize_payloads_fail_before_any_write_or_flush() {
    for payload in [Vec::new(), vec![7; MAX_WIRE + 1]] {
        let mut probe = Probe::healthy();
        assert_eq!(write_frame(&mut probe, &payload), Err(Stage::Frame));
        assert!(probe.attempted.is_empty());
        assert!(probe.wire.is_empty());
        assert_eq!(probe.flushes, 0);
    }
}
#[test]
fn partial_and_interrupted_writes_complete_the_exact_existing_frame() {
    let payload: Vec<_> = (0..19).collect();
    let mut probe = Probe::healthy();
    probe.max_write = 3;
    probe.interrupt_once = true;
    assert_eq!(write_frame(&mut probe, &payload), Ok(()));
    assert_eq!(probe.wire, expected(&payload));
    assert!(probe.attempted.len() > 2);
    assert_eq!(probe.flushes, 1);
}
#[test]
fn write_errors_and_write_zero_propagate_without_flush() {
    let payload = [1, 3, 5, 7, 9, 11, 13];
    for accepted in [0, 2, 6] {
        let mut probe = Probe::healthy();
        probe.fail_after = Some(accepted);
        assert_eq!(write_frame(&mut probe, &payload), Err(Stage::Frame));
        assert_eq!(probe.wire, expected(&payload)[..accepted]);
        assert_eq!(probe.flushes, 0);
    }
    let mut zero = Probe::healthy();
    zero.max_write = 0;
    assert_eq!(write_frame(&mut zero, &payload), Err(Stage::Frame));
    assert!(zero.wire.is_empty());
    assert_eq!(zero.flushes, 0);
}
#[test]
fn flush_error_propagates_after_complete_wire_bytes() {
    let payload = [0x12, 0x34, 0x56];
    let mut probe = Probe::healthy();
    probe.flush_error = true;
    assert_eq!(write_frame(&mut probe, &payload), Err(Stage::Frame));
    assert_eq!(probe.wire, expected(&payload));
    assert_eq!(probe.flushes, 1);
}

#[test]
fn healthy_writer_receives_one_contiguous_existing_frame() {
    for payload in [vec![0xa5], vec![0x3c; MAX_WIRE]] {
        let mut probe = Probe::healthy();
        assert_eq!(write_frame(&mut probe, &payload), Ok(()));
        assert_eq!(
            probe.attempted.len(),
            1,
            "healthy writer received a split frame"
        );
        assert_eq!(probe.attempted[0], expected(&payload));
        assert_eq!(probe.wire, expected(&payload));
        assert_eq!(probe.flushes, 1);
    }
}

#[test]
fn frame_observation_scope_refuses_nested_and_clears_on_drop() {
    let initial = super::FrameObservation::current();
    frame_observer_save("scope-initial.txt", format!("{initial:?}\n").as_bytes());
    let scope = super::FrameObservation::start().expect("fresh frame observation");
    let before = scope.snapshot();
    let refused = super::FrameObservation::start().is_none();
    let after = scope.snapshot();
    drop(scope);
    let cleared = super::FrameObservation::current();
    frame_observer_save(
        "scope-lifecycle.txt",
        format!("before={before:?} nested_refused={refused} after={after:?} cleared={cleared:?}\n")
            .as_bytes(),
    );
    assert!(initial.is_none());
    assert!(refused);
    assert_eq!(after, before);
    assert!(cleared.is_none());
}

fn frame_observer_save(label: &str, bytes: &[u8]) {
    use std::{
        fs,
        io::Write,
        os::unix::fs::{OpenOptionsExt, PermissionsExt},
        sync::atomic::{AtomicUsize, Ordering},
    };
    let Some(root) = std::env::var_os("FACTION318_CAPTURE_DIR") else {
        return;
    };
    let root = std::path::PathBuf::from(root);
    assert_eq!(
        fs::metadata(&root).unwrap().permissions().mode() & 0o7777,
        0o700
    );
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let name = format!(
        "frame-observer-{}-{}-{label}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::SeqCst)
    );
    fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .mode(0o600)
        .open(root.join(name))
        .unwrap()
        .write_all(bytes)
        .unwrap();
}

#[test]
fn frame_observation_writer_controls_keep_exact_bytes_and_results() {
    use super::{
        FrameObservation as O, FrameOutcome as Outcome, FramePart as Part, FramePhase as P,
    };
    let payload = b"synthetic framed bytes";
    for case in 0..5 {
        let mut probe = Probe::healthy();
        match case {
            1 => {
                probe.max_write = 3;
                probe.interrupt_once = true;
            }
            2 => probe.fail_after = Some(2),
            3 => probe.flush_error = true,
            4 => probe.max_write = 0,
            _ => (),
        }
        let scope = O::start().unwrap();
        O::phase(P::ResponseWrite);
        let result = write_frame(&mut probe, payload);
        let snapshot = scope.snapshot();
        drop(scope);
        frame_observer_save("writer-input.bin", payload);
        frame_observer_save("writer-wire.bin", &probe.wire);
        frame_observer_save(
            "writer-observation.txt",
            format!(
                "case={case} result={result:?} snapshot={snapshot:?} flushes={}\n",
                probe.flushes
            )
            .as_bytes(),
        );
        assert_eq!(snapshot.phase, P::ResponseWrite);
        if case < 2 {
            assert_eq!(result, Ok(()));
            assert_eq!(probe.wire, expected(payload));
            assert_eq!(probe.flushes, 1);
            assert_eq!(snapshot.part, Some(Part::Flush));
            assert_eq!(snapshot.outcome, Outcome::Complete);
            assert_eq!(snapshot.error_kind, None);
        } else {
            assert_eq!(result, Err(Stage::Frame));
            assert_eq!(snapshot.outcome, Outcome::IoError);
            assert_eq!(
                snapshot.part,
                Some(if case == 3 {
                    Part::Flush
                } else {
                    Part::WriteAll
                })
            );
            assert_eq!(
                snapshot.error_kind,
                Some(if case == 4 {
                    io::ErrorKind::WriteZero
                } else {
                    io::ErrorKind::BrokenPipe
                })
            );
            let count = if case == 3 {
                expected(payload).len()
            } else if case == 2 {
                2
            } else {
                0
            };
            assert_eq!(probe.wire, expected(payload)[..count]);
            assert_eq!(probe.flushes, usize::from(case == 3));
        }
        assert!(O::current().is_none());
    }
    for payload in [Vec::new(), vec![7; MAX_WIRE + 1]] {
        let scope = O::start().unwrap();
        O::phase(P::ResponseWrite);
        let mut probe = Probe::healthy();
        let result = write_frame(&mut probe, &payload);
        let snapshot = scope.snapshot();
        drop(scope);
        frame_observer_save(
            "writer-invalid-observation.txt",
            format!("result={result:?} snapshot={snapshot:?}\n").as_bytes(),
        );
        assert_eq!(result, Err(Stage::Frame));
        assert_eq!(snapshot.part, Some(Part::LengthValidation));
        assert_eq!(snapshot.outcome, Outcome::InvalidLength);
        assert_eq!(snapshot.error_kind, None);
        assert!(probe.wire.is_empty());
        assert_eq!(probe.flushes, 0);
    }
}

#[test]
fn frame_observation_phase_entries_and_panic_scopes_clear_without_leaking_state() {
    use super::{FrameObservation as O, FrameOutcome as Outcome, FramePhase as P};
    let scope = O::start().unwrap();
    for phase in [
        P::InitialRead,
        P::DecodeDispatch,
        P::ResponseWrite,
        P::NextRead,
    ] {
        O::phase(phase);
        let snapshot = scope.snapshot();
        frame_observer_save("phase-entry.txt", format!("{snapshot:?}\n").as_bytes());
        assert_eq!(snapshot.phase, phase);
        assert_eq!(snapshot.part, None);
        assert_eq!(snapshot.outcome, Outcome::Entered);
        assert_eq!(snapshot.error_kind, None);
    }
    drop(scope);
    let panicked = std::panic::catch_unwind(|| {
        let scope = O::start().unwrap();
        O::phase(P::NextRead);
        let snapshot = scope.snapshot();
        frame_observer_save("before-panic.txt", format!("{snapshot:?}\n").as_bytes());
        panic!("fixed synthetic frame observation unwind");
    });
    let after = O::current();
    frame_observer_save(
        "after-panic.txt",
        format!("panicked={} snapshot={after:?}\n", panicked.is_err()).as_bytes(),
    );
    assert!(panicked.is_err());
    assert!(after.is_none());
    let scope = O::start().unwrap();
    let fresh = scope.snapshot();
    drop(scope);
    frame_observer_save("fresh-scope.txt", format!("{fresh:?}\n").as_bytes());
    assert_eq!(fresh.phase, P::InitialRead);
    assert_eq!(fresh.part, None);
    assert_eq!(fresh.outcome, Outcome::Entered);
    assert_eq!(fresh.error_kind, None);
    assert!(O::current().is_none());
}

#[test]
fn frame_observation_complete_representation_never_contains_payload_or_error_message() {
    use super::{
        FrameObservation as O, FrameOutcome as Outcome, FramePart as Part, FramePhase as P,
    };
    const PAYLOAD: &[u8] = b"PRIVATE_PAYLOAD_SENTINEL_326";
    const MESSAGE: &str = "PRIVATE_ERROR_SENTINEL_326";
    struct PrivateFailure;
    impl Write for PrivateFailure {
        fn write(&mut self, _bytes: &[u8]) -> io::Result<usize> {
            Err(io::Error::other(MESSAGE))
        }
        fn flush(&mut self) -> io::Result<()> {
            panic!("write failure unexpectedly flushed")
        }
    }
    let scope = O::start().unwrap();
    O::phase(P::ResponseWrite);
    let result = write_frame(&mut PrivateFailure, PAYLOAD);
    let snapshot = scope.snapshot();
    let representation = format!("{snapshot:?}");
    drop(scope);
    frame_observer_save("privacy-payload.bin", PAYLOAD);
    frame_observer_save("privacy-error.txt", MESSAGE.as_bytes());
    frame_observer_save("privacy-fixed-snapshot.txt", representation.as_bytes());
    assert_eq!(result, Err(Stage::Frame));
    assert_eq!(snapshot.part, Some(Part::WriteAll));
    assert_eq!(snapshot.outcome, Outcome::IoError);
    assert_eq!(snapshot.error_kind, Some(io::ErrorKind::Other));
    assert!(!representation.contains(MESSAGE));
    assert!(!representation.contains(std::str::from_utf8(PAYLOAD).unwrap()));
}

#[cfg(feature = "delegation-issuer")]
type ObservedTlsFrameRead = (
    Result<Option<Vec<u8>>, Stage>,
    super::FrameSnapshot,
    (Result<(), io::ErrorKind>, Option<bool>),
);

#[cfg(feature = "delegation-issuer")]
fn observed_tls_frame_read(
    wire: &[u8],
    close_notify: bool,
    close_refusal: Option<Stage>,
) -> ObservedTlsFrameRead {
    use super::{
        Arc, AtomicBool, CYCLE_LIMIT, Client, DeadlineIo, FrameObservation, FramePhase,
        HANDSHAKE_LIMIT, Instant, Server, TcpListener, sha256, test_certificate, wall_now,
    };
    let t = wall_now() as i64;
    let ca = test_certificate(None, "frame-observer-ca", 401, t - 60, t + 3600);
    let server_identity =
        test_certificate(Some((&ca.0, &ca.1)), "provider.test", 402, t - 60, t + 3600);
    let client_identity = test_certificate(
        Some((&ca.0, &ca.1)),
        "frame-observer-client",
        403,
        t - 60,
        t + 3600,
    );
    for (label, identity) in [
        ("ca", &ca),
        ("server", &server_identity),
        ("client", &client_identity),
    ] {
        frame_observer_save(&format!("tls-{label}.pem"), &identity.0.to_pem().unwrap());
        frame_observer_save(
            &format!("tls-{label}.key"),
            &identity.1.private_key_to_pem_pkcs8().unwrap(),
        );
    }
    frame_observer_save("tls-raw-input.bin", wire);
    let server = Server::new(&server_identity.0, &server_identity.1, &ca.0).unwrap();
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    listener.set_nonblocking(true).unwrap();
    let start = Instant::now();
    let handshake = start + HANDSHAKE_LIMIT;
    let deadline = start + CYCLE_LIMIT;
    let (result, snapshot, sender) = std::thread::scope(|scope| {
        let peer = scope.spawn(|| {
            let mut client = Client::connect(
                address,
                "provider.test",
                sha256(&server_identity.0.to_der().unwrap()),
                &ca.0,
                Some((&client_identity.0, Some(&client_identity.1))),
                Arc::new(AtomicBool::new(false)),
            )
            .expect("fixed TLS frame fixture client admission");
            let written = client
                .stream
                .write_all(wire)
                .and_then(|()| client.stream.flush())
                .map_err(|error| error.kind());
            let closed = close_notify.then(|| {
                match close_refusal {
                    Some(Stage::Cancelled) => client.cancel.store(true, std::sync::atomic::Ordering::SeqCst),
                    Some(Stage::Deadline) => client.deadline = client.deadline.min(Instant::now()),
                    None => (),
                    _ => panic!("unsupported fixed close-notify guard control"),
                }
                let original_deadline = client.deadline;
                let original_attempts = client.attempted;
                let result: Result<(), Stage> = client.test_close_notify();
                let deadline_unchanged = client.deadline == original_deadline;
                let attempts_unchanged = client.attempted == original_attempts;
                frame_observer_save("close-notify-control.txt", format!("expected_refusal={close_refusal:?} result={result:?} deadline_unchanged={deadline_unchanged} attempts_unchanged={attempts_unchanged}\n").as_bytes());
                assert!(deadline_unchanged);
                assert!(attempts_unchanged);
                if let Some(expected) = close_refusal { assert_eq!(result, Err(expected)); }
                result.is_ok()
            });
            (written, closed)
        });
        let socket = loop {
            match listener.accept() {
                Ok((socket, _)) => break socket,
                Err(error)
                    if error.kind() == io::ErrorKind::WouldBlock && Instant::now() < handshake =>
                {
                    std::thread::sleep(std::time::Duration::from_millis(1))
                }
                _ => panic!("fixed TLS frame fixture accept failed"),
            }
        };
        let io = DeadlineIo::new(socket, handshake, Arc::new(AtomicBool::new(false)));
        let mut stream = server
            .acceptor
            .accept(io)
            .unwrap_or_else(|_| panic!("fixed TLS frame fixture handshake failed"));
        stream.get_mut().deadline = deadline;
        let observed = FrameObservation::start().unwrap();
        FrameObservation::phase(FramePhase::InitialRead);
        let result = super::read_frame(&mut stream);
        let snapshot = observed.snapshot();
        drop(observed);
        drop(stream);
        let sender = peer.join().unwrap();
        (result, snapshot, sender)
    });
    let result_shape = result.as_ref().map(|bytes| bytes.as_ref().map(Vec::len));
    frame_observer_save(
        "tls-fixed-observation.txt",
        format!("result={result_shape:?} snapshot={snapshot:?} sender={sender:?}\n").as_bytes(),
    );
    if let Ok(Some(bytes)) = &result {
        frame_observer_save("tls-decoded-output.bin", bytes);
    }
    // The sender has no observation activation; it cannot mutate this thread's
    // slot. No Runtime, provider Session or dispatch authority exists here.
    (result, snapshot, sender)
}

#[cfg(feature = "delegation-issuer")]
#[test]
fn frame_observation_real_tls_read_controls_keep_valid_invalid_and_truncated_results() {
    use super::{FrameOutcome as Outcome, FramePart as Part, FramePhase as P};
    let payload = b"real bounded TLS bytes";
    let (result, snapshot, sender) = observed_tls_frame_read(&expected(payload), true, None);
    assert!(sender.0.is_ok());
    assert_eq!(result, Ok(Some(payload.to_vec())));
    assert_eq!(snapshot.phase, P::InitialRead);
    assert_eq!(snapshot.part, Some(Part::Body));
    assert_eq!(snapshot.outcome, Outcome::Complete);
    assert_eq!(snapshot.error_kind, None);
    for length in [0u32, (MAX_WIRE + 1) as u32] {
        let (result, snapshot, sender) = observed_tls_frame_read(&length.to_be_bytes(), true, None);
        assert!(sender.0.is_ok());
        assert_eq!(result, Err(Stage::Frame));
        assert_eq!(snapshot.part, Some(Part::LengthValidation));
        assert_eq!(snapshot.outcome, Outcome::InvalidLength);
        assert_eq!(snapshot.error_kind, None);
    }
    for (wire, part) in [
        (vec![0, 0], Part::PrefixRest),
        (vec![0, 0, 0, 8, b'a', b'b', b'c'], Part::Body),
    ] {
        let (result, snapshot, sender) = observed_tls_frame_read(&wire, true, None);
        assert!(sender.0.is_ok());
        assert_eq!(result, Err(Stage::Frame));
        assert_eq!(snapshot.part, Some(part));
        assert_eq!(snapshot.outcome, Outcome::IoError);
        assert!(snapshot.error_kind.is_some());
    }
}

#[cfg(feature = "delegation-issuer")]
#[test]
fn frame_observation_empty_tls_boundary_distinguishes_clean_eof_and_closure_error() {
    use super::{FrameOutcome as Outcome, FramePart as Part};
    for close_notify in [true, false] {
        let (result, snapshot, sender) = observed_tls_frame_read(&[], close_notify, None);
        assert!(sender.0.is_ok());
        assert_eq!(snapshot.part, Some(Part::PrefixFirst));
        if close_notify {
            assert_eq!(result, Ok(None));
            assert_eq!(sender.1, Some(true));
        } else {
            assert_eq!(result, Err(Stage::Frame));
        }
        match result {
            Ok(None) => {
                assert_eq!(snapshot.outcome, Outcome::CleanEof);
                assert_eq!(snapshot.error_kind, None);
            }
            Err(Stage::Frame) => {
                assert_eq!(snapshot.outcome, Outcome::IoError);
                assert!(snapshot.error_kind.is_some());
            }
            other => panic!(
                "unexpected empty-boundary result shape: {:?}",
                other.map(|bytes| bytes.map(|data| data.len()))
            ),
        }
    }
}

#[cfg(feature = "delegation-issuer")]
#[test]
fn frame_observation_close_notify_succeeds_once_and_refuses_original_cancel_or_expiry() {
    use super::{FrameOutcome as Outcome, FramePart as Part};
    // Empty wire makes the real peer wait for close_notify, not close first.
    let (result, snapshot, sender) = observed_tls_frame_read(&[], true, None);
    assert!(sender.0.is_ok());
    assert_eq!(sender.1, Some(true));
    assert_eq!(result, Ok(None));
    assert_eq!(snapshot.part, Some(Part::PrefixFirst));
    assert_eq!(snapshot.outcome, Outcome::CleanEof);
    for guard in [Stage::Cancelled, Stage::Deadline] {
        let (result, snapshot, sender) = observed_tls_frame_read(&[], true, Some(guard));
        assert!(sender.0.is_ok());
        assert_eq!(sender.1, Some(false));
        assert_eq!(result, Err(Stage::Frame));
        assert_eq!(snapshot.part, Some(Part::PrefixFirst));
        assert_eq!(snapshot.outcome, Outcome::IoError);
        assert!(snapshot.error_kind.is_some());
    }
}

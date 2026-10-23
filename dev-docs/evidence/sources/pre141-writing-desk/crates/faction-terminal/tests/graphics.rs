use faction_terminal::graphics::*;
use std::time::{Duration, Instant};
#[test]
fn chunks_are_inline_bounded_and_exact_rgb_without_a_frame_cache() {
    let chunks = (0..CHUNKS).map(|i| chunk(i).unwrap()).collect::<Vec<_>>();
    assert_eq!(RGB.len(), 38400);
    assert_eq!(CHUNKS, 13);
    assert!(chunk(CHUNKS).is_none());
    let mut encoded = 0;
    for (i, c) in chunks.iter().enumerate() {
        assert!(c.len() <= 4096);
        assert_eq!(c.capacity(), 4096);
        assert!(c.is_ascii() && c.starts_with("\x1b_G") && c.ends_with("\x1b\\"));
        let (header, data) = c.split_once(';').unwrap();
        assert!(header.contains(if i + 1 == CHUNKS { "m=0" } else { "m=1" }));
        assert!(header.contains("q=2"));
        let data = &data[..data.len() - 2];
        assert!(data.len() <= 4000 && data.len() % 4 == 0);
        assert!(
            data.bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"+/".contains(&b))
        );
        encoded += data.len();
    }
    assert_eq!(encoded, 51200);
    assert!(chunks[0].starts_with("\x1b_Ga=T,t=d,f=24,s=160,v=80,i=17391,c=20,r=5,q=2,m=1;"));
}
#[test]
fn graphics_probe_owns_fragmented_unknown_refused_malformed_and_late_reports() {
    let now = Instant::now();
    for (bytes, expected) in [
        (b"".as_slice(), Some(false)),
        (b"\x1b_Gi=17392;OK\x1b\\", Some(true)),
        (b"\x1b_Gi=17392;ENOTSUP:unsupported\x1b\\", Some(false)),
        (b"\x1b_Gi=17392;OK", None),
    ] {
        let mut p = Probe::default();
        p.begin(now);
        assert!(!p.expired(now + Duration::from_millis(299)));
        for &b in bytes {
            p.feed(b).unwrap();
        }
        assert_eq!(p.result().ok(), expected);
        assert!(p.expired(now + Duration::from_millis(300)));
        p.begin(now + Duration::from_secs(5));
        assert!(p.expired(now + Duration::from_millis(300)));
    }
    for bytes in [
        b"\x1b_Gi=17392;OK\x1b\\p\n".as_slice(),
        b"\x1b_Gi=1;OK\x1b\\",
        b"\x1b]52;c;secret\x07",
        b"\x1b_Gi=17392;\n",
        &[b"\x1b_Gi=17392;".as_slice(), &[b'a'; 129]].concat(),
    ] {
        let mut p = Probe::default();
        assert!(bytes.iter().any(|b| p.feed(*b).is_err()));
    }
}
#[test]
fn native_gate_is_local_opt_in_ordered_fixed_art_and_rate_limited() {
    let now = Instant::now();
    let mut g = Gate::default();
    assert!(g.accept(QUERY, now).unwrap().is_none());
    g.enabled = true;
    assert_eq!(g.accept(QUERY, now).unwrap().unwrap(), QUERY);
    assert!(g.accept(&chunk(1).unwrap(), now).is_err());
    for i in 0..CHUNKS {
        assert_eq!(
            g.accept(&chunk(i).unwrap(), now).unwrap().unwrap(),
            chunk(i).unwrap()
        );
    }
    assert!(g.accept(&chunk(0).unwrap(), now).is_err());
    assert_eq!(g.accept(DELETE, now).unwrap().unwrap(), DELETE);
    assert!(g.accept(&chunk(0).unwrap(), now).is_err());
    assert!(g.accept(&chunk(0).unwrap(), now + INTERVAL).is_ok());
    assert!(g.expired(now + INTERVAL + TRANSFER_TIMEOUT));
    assert!(
        g.accept(&chunk(1).unwrap(), now + INTERVAL + TRANSFER_TIMEOUT)
            .is_err()
    );
    assert_eq!(g.accept(DELETE, now).unwrap().unwrap(), DELETE);
    for malicious in [
        "\x1b_Ga=T,t=f;L2V0Yy9wYXNzd2Q=\x1b\\",
        "\x1b_Gf=100;AAAA\x1b\\",
        "\x1b_Gt=s;AAAA\x1b\\",
        "\x1b_Ga=d,d=A;\x1b\\",
    ] {
        assert!(g.accept(malicious, now + INTERVAL).is_err());
    }
}

#[test]
fn tty_output_deadline_restores_flags_after_a_blocked_peer() {
    use rustix::fs::{OFlags, fcntl_getfl, fcntl_setfl};
    use std::os::fd::OwnedFd;
    use std::{fs::File, io::Write, os::unix::net::UnixStream};
    let (writer, _reader) = UnixStream::pair().unwrap();
    let mut file = File::from(OwnedFd::from(writer));
    let original = fcntl_getfl(&file).unwrap();
    fcntl_setfl(&file, original | OFlags::NONBLOCK).unwrap();
    let data = [0u8; 4096];
    loop {
        match file.write(&data) {
            Ok(_) => (),
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
            Err(e) => panic!("{e}"),
        }
    }
    fcntl_setfl(&file, original).unwrap();
    let now = Instant::now();
    assert!(write_tty(&mut file, &data, now + Duration::from_millis(20)).is_err());
    assert!(now.elapsed() < Duration::from_secs(1));
    assert_eq!(
        fcntl_getfl(&file).unwrap() & OFlags::NONBLOCK,
        original & OFlags::NONBLOCK
    );
}

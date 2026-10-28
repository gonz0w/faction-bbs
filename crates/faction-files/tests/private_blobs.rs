use faction_files::private_blobs::{
    Access, Action, Binding, CHUNK, Error, Principal, Request, Response, Store,
};
use sha2::{Digest, Sha256};
use std::{cell::Cell, fs, os::unix::fs::DirBuilderExt, path::PathBuf};
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let p = std::env::temp_dir().canonicalize().unwrap().join(format!(
            "faction260-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::DirBuilder::new().mode(0o700).create(&p).unwrap();
        fs::write(p.join("OWNER"), b"Owned issue260 fixture").unwrap();
        fs::DirBuilder::new()
            .mode(0o700)
            .create(p.join("store"))
            .unwrap();
        Self(p)
    }
    fn store(&self) -> PathBuf {
        self.0.join("store")
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        assert_eq!(
            fs::read(self.0.join("OWNER")).unwrap(),
            b"Owned issue260 fixture"
        );
        fs::remove_dir_all(&self.0).unwrap();
        assert!(!self.0.exists());
    }
}
struct Policy(Cell<bool>);
impl Access for Policy {
    fn require(&self, _: &Principal, _: &Binding, _: Action, _: u64) -> Result<(), Error> {
        if self.0.get() {
            Ok(())
        } else {
            Err(Error::Denied)
        }
    }
}
fn actor(name: &str, device: u32) -> Principal {
    Principal {
        account: format!("{name}@home.test"),
        device,
        identity: [device as u8; 32],
    }
}
fn binding(data: &[u8]) -> Binding {
    Binding {
        sender: actor("alice", 1),
        recipient: actor("bob", 2),
        id: Sha256::digest(data).into(),
        size: data.len() as u64,
        created: 100,
        expires: 200,
    }
}
#[test]
fn partial_ciphertext_survives_reopen_and_recipient_only_reads_committed_exact_bytes() {
    let f = Fixture::new();
    let policy = Policy(Cell::new(true));
    let data = vec![0xa5; CHUNK + 17];
    let b = binding(&data);
    let id = b.id;
    let mut s = Store::open(&f.store()).unwrap();
    assert!(matches!(
        s.dispatch(&b.sender, &policy, Request::Begin(b.clone()), 100)
            .unwrap(),
        Response::Progress {
            offset: 0,
            committed: false,
            ..
        }
    ));
    s.dispatch(
        &b.sender,
        &policy,
        Request::PutChunk {
            id,
            offset: 0,
            bytes: data[..CHUNK].to_vec(),
        },
        101,
    )
    .unwrap();
    assert_eq!(
        s.dispatch(
            &b.recipient,
            &policy,
            Request::GetChunk { id, offset: 0 },
            101
        ),
        Err(Error::Denied)
    );
    assert_eq!(
        s.dispatch(&b.recipient, &policy, Request::Status { id }, 101),
        Err(Error::Denied)
    );
    drop(s);
    let mut s = Store::open(&f.store()).unwrap();
    assert!(matches!(
        s.dispatch(&b.sender, &policy, Request::Status { id }, 101)
            .unwrap(),
        Response::Progress {
            offset: 4096,
            committed: false,
            ..
        }
    ));
    s.dispatch(
        &b.sender,
        &policy,
        Request::PutChunk {
            id,
            offset: CHUNK as u64,
            bytes: data[CHUNK..].to_vec(),
        },
        102,
    )
    .unwrap();
    s.dispatch(&b.sender, &policy, Request::Commit { id }, 102)
        .unwrap();
    assert_eq!(
        s.dispatch(
            &b.recipient,
            &policy,
            Request::GetChunk { id, offset: 0 },
            102
        )
        .unwrap(),
        Response::Chunk {
            offset: 0,
            bytes: data[..CHUNK].to_vec(),
            total: data.len() as u64
        }
    );
    policy.0.set(false);
    assert_eq!(
        s.dispatch(
            &b.recipient,
            &policy,
            Request::GetChunk { id, offset: 0 },
            103
        ),
        Err(Error::Denied)
    );
}
#[test]
fn every_current_policy_gate_and_exact_principal_binding_precede_disclosure_or_mutation() {
    let f = Fixture::new();
    let policy = Policy(Cell::new(true));
    let data = vec![7; 32];
    let b = binding(&data);
    let id = b.id;
    let mut s = Store::open(&f.store()).unwrap();
    s.dispatch(&b.sender, &policy, Request::Begin(b.clone()), 100)
        .unwrap();
    let requests = [
        Request::Begin(b.clone()),
        Request::Status { id },
        Request::PutChunk {
            id,
            offset: 0,
            bytes: data.clone(),
        },
        Request::Commit { id },
        Request::GetChunk { id, offset: 0 },
        Request::Delete { id },
    ];
    let before = fs::read(f.store().join("private-blobs.snapshot")).unwrap();
    policy.0.set(false);
    for r in &requests {
        assert_eq!(
            s.dispatch(&b.sender, &policy, r.clone(), 101),
            Err(Error::Denied)
        );
    }
    assert_eq!(
        fs::read(f.store().join("private-blobs.snapshot")).unwrap(),
        before
    );
    policy.0.set(true);
    for changed in [
        Principal {
            account: "mallory@home.test".into(),
            ..b.sender.clone()
        },
        Principal {
            device: 3,
            ..b.sender.clone()
        },
        Principal {
            identity: [3; 32],
            ..b.sender.clone()
        },
    ] {
        for r in &requests {
            assert_eq!(
                s.dispatch(&changed, &policy, r.clone(), 101),
                Err(Error::Denied)
            );
        }
    }
    s.dispatch(
        &b.sender,
        &policy,
        Request::PutChunk {
            id,
            offset: 0,
            bytes: data,
        },
        101,
    )
    .unwrap();
    s.dispatch(&b.sender, &policy, Request::Commit { id }, 101)
        .unwrap();
    for changed in [
        Principal {
            account: "mallory@home.test".into(),
            ..b.recipient.clone()
        },
        Principal {
            device: 3,
            ..b.recipient.clone()
        },
        Principal {
            identity: [3; 32],
            ..b.recipient.clone()
        },
    ] {
        assert_eq!(
            s.dispatch(&changed, &policy, Request::GetChunk { id, offset: 0 }, 102),
            Err(Error::Denied)
        );
        assert_eq!(
            s.dispatch(&changed, &policy, Request::Status { id }, 102),
            Err(Error::Denied)
        );
    }
    assert_eq!(
        s.dispatch(&b.recipient, &policy, Request::Delete { id }, 102),
        Err(Error::Denied)
    );
    policy.0.set(false);
    assert_eq!(
        s.dispatch(
            &b.recipient,
            &policy,
            Request::GetChunk { id, offset: 0 },
            102
        ),
        Err(Error::Denied)
    );
}
#[test]
fn exact_retries_are_immutable_and_digest_length_offsets_and_chunks_fail_closed() {
    let f = Fixture::new();
    let policy = Policy(Cell::new(true));
    let data = vec![8; 32];
    let b = binding(&data);
    let id = b.id;
    let mut s = Store::open(&f.store()).unwrap();
    s.dispatch(&b.sender, &policy, Request::Begin(b.clone()), 100)
        .unwrap();
    assert_eq!(
        s.dispatch(&b.sender, &policy, Request::Commit { id }, 100),
        Err(Error::Invalid)
    );
    for (offset, bytes) in [
        (0, vec![]),
        (0, vec![0; CHUNK + 1]),
        (1, vec![8]),
        (31, vec![8; 2]),
        (u64::MAX, vec![8]),
    ] {
        assert_eq!(
            s.dispatch(
                &b.sender,
                &policy,
                Request::PutChunk { id, offset, bytes },
                100
            ),
            Err(Error::Invalid)
        );
    }
    s.dispatch(
        &b.sender,
        &policy,
        Request::PutChunk {
            id,
            offset: 0,
            bytes: data.clone(),
        },
        100,
    )
    .unwrap();
    let progress = s
        .dispatch(
            &b.sender,
            &policy,
            Request::PutChunk {
                id,
                offset: 0,
                bytes: data.clone(),
            },
            100,
        )
        .unwrap();
    assert!(matches!(
        progress,
        Response::Progress {
            offset: 32,
            committed: false,
            ..
        }
    ));
    assert_eq!(
        s.dispatch(
            &b.sender,
            &policy,
            Request::PutChunk {
                id,
                offset: 0,
                bytes: vec![9; 32]
            },
            100
        ),
        Err(Error::Invalid)
    );
    let mut altered = b.clone();
    altered.expires += 1;
    assert_eq!(
        s.dispatch(&b.sender, &policy, Request::Begin(altered), 100),
        Err(Error::Invalid)
    );
    let mut altered = b.clone();
    altered.recipient.identity = [3; 32];
    assert_eq!(
        s.dispatch(&b.sender, &policy, Request::Begin(altered), 100),
        Err(Error::Invalid)
    );
    s.dispatch(&b.sender, &policy, Request::Commit { id }, 100)
        .unwrap();
    assert!(matches!(
        s.dispatch(&b.sender, &policy, Request::Commit { id }, 100)
            .unwrap(),
        Response::Progress {
            committed: true,
            ..
        }
    ));
    s.dispatch(
        &b.sender,
        &policy,
        Request::PutChunk {
            id,
            offset: 0,
            bytes: data,
        },
        100,
    )
    .unwrap();
    assert_eq!(
        s.dispatch(
            &b.recipient,
            &policy,
            Request::GetChunk { id, offset: 33 },
            100
        ),
        Err(Error::Invalid)
    );
    assert_eq!(
        s.dispatch(
            &b.recipient,
            &policy,
            Request::GetChunk { id, offset: 32 },
            100
        )
        .unwrap(),
        Response::Chunk {
            offset: 32,
            bytes: vec![],
            total: 32
        }
    );
    let wrong = Binding {
        id: [5; 32],
        ..b.clone()
    };
    s.dispatch(&b.sender, &policy, Request::Begin(wrong.clone()), 100)
        .unwrap();
    s.dispatch(
        &b.sender,
        &policy,
        Request::PutChunk {
            id: wrong.id,
            offset: 0,
            bytes: vec![8; 32],
        },
        100,
    )
    .unwrap();
    assert_eq!(
        s.dispatch(&b.sender, &policy, Request::Commit { id: wrong.id }, 100),
        Err(Error::Invalid)
    );
}
#[test]
fn incomplete_reservations_count_expiry_never_renews_and_explicit_owner_cleanup_is_authorized() {
    use faction_files::private_blobs::{MAX_CIPHERTEXT, MAX_OBJECTS, MAX_RETENTION};
    let f = Fixture::new();
    let policy = Policy(Cell::new(true));
    let mut s = Store::open(&f.store()).unwrap();
    let base = binding(&[0; 32]);
    for i in 0..MAX_OBJECTS {
        let b = Binding {
            id: [i as u8; 32],
            size: MAX_CIPHERTEXT as u64,
            ..base.clone()
        };
        s.dispatch(&b.sender, &policy, Request::Begin(b.clone()), 100)
            .unwrap();
    }
    assert_eq!(
        s.dispatch(&base.sender, &policy, Request::Begin(base.clone()), 100),
        Err(Error::Quota)
    );
    let id = [0; 32];
    assert_eq!(
        s.dispatch(&base.sender, &policy, Request::Status { id }, 200),
        Err(Error::Expired)
    );
    let longer = Binding {
        id,
        size: MAX_CIPHERTEXT as u64,
        expires: 201,
        ..base.clone()
    };
    assert_eq!(
        s.dispatch(&base.sender, &policy, Request::Begin(longer.clone()), 200),
        Err(Error::Expired)
    );
    policy.0.set(false);
    assert_eq!(
        s.dispatch(&base.sender, &policy, Request::Delete { id }, 200),
        Err(Error::Denied)
    );
    policy.0.set(true);
    s.dispatch(&base.sender, &policy, Request::Delete { id }, 200)
        .unwrap();
    let recreated = Binding {
        created: 200,
        expires: 201,
        ..longer
    };
    s.dispatch(&base.sender, &policy, Request::Begin(recreated), 200)
        .unwrap();
    for b in [
        Binding {
            size: 0,
            ..base.clone()
        },
        Binding {
            size: MAX_CIPHERTEXT as u64 + 1,
            ..base.clone()
        },
        Binding {
            expires: 100 + MAX_RETENTION + 1,
            ..base.clone()
        },
        Binding {
            expires: 100,
            ..base.clone()
        },
    ] {
        assert_eq!(
            s.dispatch(&base.sender, &policy, Request::Begin(b), 200),
            Err(Error::Invalid)
        );
    }
}
#[test]
fn successful_read_highwater_is_durable_and_rollback_denials_do_not_change_storage() {
    let f = Fixture::new();
    let policy = Policy(Cell::new(true));
    let b = binding(&[0; 32]);
    let mut s = Store::open(&f.store()).unwrap();
    assert_eq!(
        s.dispatch(&b.sender, &policy, Request::Begin(b.clone()), 99),
        Err(Error::Clock)
    );
    s.dispatch(&b.sender, &policy, Request::Begin(b.clone()), 100)
        .unwrap();
    s.dispatch(&b.sender, &policy, Request::Status { id: b.id }, 150)
        .unwrap();
    drop(s);
    let mut s = Store::open(&f.store()).unwrap();
    let before = fs::read(f.store().join("private-blobs.snapshot")).unwrap();
    assert_eq!(
        s.dispatch(&b.sender, &policy, Request::Status { id: b.id }, 149),
        Err(Error::Clock)
    );
    assert_eq!(
        fs::read(f.store().join("private-blobs.snapshot")).unwrap(),
        before
    );
    assert!(
        s.dispatch(&b.sender, &policy, Request::Status { id: b.id }, 199)
            .is_ok()
    );
    assert_eq!(
        s.dispatch(&b.sender, &policy, Request::Status { id: b.id }, 200),
        Err(Error::Expired)
    );
}
#[test]
fn maximum_complete_ciphertext_reopens_verified_with_bounded_regular_file_bytes() {
    use faction_files::private_blobs::{MAX_CIPHERTEXT, MAX_DISK_BYTES};
    let started = std::time::Instant::now();
    let f = Fixture::new();
    let policy = Policy(Cell::new(true));
    let data = vec![0x5a; MAX_CIPHERTEXT];
    let b = binding(&data);
    let id = b.id;
    let mut s = Store::open(&f.store()).unwrap();
    s.dispatch(&b.sender, &policy, Request::Begin(b.clone()), 100)
        .unwrap();
    for (i, chunk) in data.chunks(CHUNK).enumerate() {
        s.dispatch(
            &b.sender,
            &policy,
            Request::PutChunk {
                id,
                offset: (i * CHUNK) as u64,
                bytes: chunk.to_vec(),
            },
            100,
        )
        .unwrap();
    }
    s.dispatch(&b.sender, &policy, Request::Commit { id }, 100)
        .unwrap();
    drop(s);
    let mut s = Store::open(&f.store()).unwrap();
    let mut actual = Vec::new();
    while actual.len() < data.len() {
        match s
            .dispatch(
                &b.recipient,
                &policy,
                Request::GetChunk {
                    id,
                    offset: actual.len() as u64,
                },
                100,
            )
            .unwrap()
        {
            Response::Chunk { bytes, total, .. } => {
                assert_eq!(total, data.len() as u64);
                actual.extend(bytes)
            }
            _ => panic!("wrong response"),
        }
    }
    assert_eq!(actual, data);
    let lengths = fs::read_dir(f.store())
        .unwrap()
        .map(|p| p.unwrap().metadata().unwrap().len())
        .collect::<Vec<_>>();
    assert_eq!(lengths.len(), 2);
    assert!(lengths.iter().sum::<u64>() <= MAX_DISK_BYTES as u64);
    println!(
        "one maximum ciphertext store upload/commit/reopen/download elapsed_ms={} regular_bytes={}",
        started.elapsed().as_millis(),
        lengths.iter().sum::<u64>()
    );
}
#[test]
fn canonical_codecs_reject_unknown_truncated_trailing_invalid_and_oversized_input() {
    use faction_files::private_blobs::{
        MAX_WIRE, decode_request, decode_response, encode_request, encode_response,
    };
    let b = binding(&[1; 32]);
    let id = b.id;
    for request in [
        Request::Begin(b.clone()),
        Request::Status { id },
        Request::PutChunk {
            id,
            offset: 0,
            bytes: vec![8; CHUNK],
        },
        Request::Commit { id },
        Request::GetChunk { id, offset: 0 },
        Request::Delete { id },
    ] {
        let raw = encode_request(&request).unwrap();
        assert_eq!(decode_request(&raw).unwrap(), request);
        for n in 0..raw.len() {
            assert!(decode_request(&raw[..n]).is_err());
        }
        let mut extra = raw.clone();
        extra.push(0);
        assert!(decode_request(&extra).is_err());
        let mut unknown = raw;
        unknown[8] = 255;
        assert!(decode_request(&unknown).is_err());
    }
    for response in [
        Response::Progress {
            binding: b.clone(),
            offset: 0,
            committed: false,
        },
        Response::Chunk {
            offset: 0,
            bytes: vec![9; 32],
            total: 32,
        },
        Response::Deleted,
    ] {
        let raw = encode_response(&response).unwrap();
        assert_eq!(decode_response(&raw).unwrap(), response);
        for n in 0..raw.len() {
            assert!(decode_response(&raw[..n]).is_err());
        }
        let mut extra = raw;
        extra.push(0);
        assert!(decode_response(&extra).is_err());
    }
    assert!(decode_request(&vec![0; MAX_WIRE + 1]).is_err());
    assert!(decode_response(&vec![0; MAX_WIRE + 1]).is_err());
    assert!(
        encode_request(&Request::PutChunk {
            id,
            offset: 0,
            bytes: vec![0; CHUNK + 1]
        })
        .is_err()
    );
    assert!(
        encode_response(&Response::Chunk {
            offset: u64::MAX,
            bytes: vec![0; 32],
            total: 32
        })
        .is_err()
    );
    assert!(
        encode_response(&Response::Progress {
            binding: b,
            offset: 0,
            committed: true
        })
        .is_err()
    );
}
#[test]
fn filesystem_refuses_links_permissions_corruption_and_live_lock_without_unlink() {
    use std::os::unix::fs::{PermissionsExt, symlink};
    let f = Fixture::new();
    let store = f.store();
    let s = Store::open(&store).unwrap();
    let lock = store.join("private-blobs.lock");
    let inode = std::os::unix::fs::MetadataExt::ino(&fs::metadata(&lock).unwrap());
    assert!(matches!(Store::open(&store), Err(Error::Busy)));
    assert_eq!(
        std::os::unix::fs::MetadataExt::ino(&fs::metadata(&lock).unwrap()),
        inode
    );
    drop(s);
    fs::hard_link(&lock, f.0.join("other-lock")).unwrap();
    assert!(Store::open(&store).is_err());
    fs::remove_file(f.0.join("other-lock")).unwrap();
    fs::set_permissions(&store, fs::Permissions::from_mode(0o750)).unwrap();
    assert!(Store::open(&store).is_err());
    fs::set_permissions(&store, fs::Permissions::from_mode(0o700)).unwrap();
    let snapshot = store.join("private-blobs.snapshot");
    let valid = fs::read(&snapshot).unwrap();
    fs::set_permissions(&snapshot, fs::Permissions::from_mode(0o644)).unwrap();
    assert!(Store::open(&store).is_err());
    fs::set_permissions(&snapshot, fs::Permissions::from_mode(0o600)).unwrap();
    fs::hard_link(&snapshot, f.0.join("other-snapshot")).unwrap();
    assert!(Store::open(&store).is_err());
    fs::remove_file(f.0.join("other-snapshot")).unwrap();
    fs::remove_file(&snapshot).unwrap();
    symlink(f.0.join("outside"), &snapshot).unwrap();
    assert!(Store::open(&store).is_err());
    fs::remove_file(&snapshot).unwrap();
    fs::write(&snapshot, b"malformed snapshot").unwrap();
    fs::set_permissions(&snapshot, fs::Permissions::from_mode(0o600)).unwrap();
    assert!(Store::open(&store).is_err());
    fs::write(&snapshot, &valid).unwrap();
    let link = f.0.join("link");
    symlink(&store, &link).unwrap();
    assert!(Store::open(&link).is_err());
    fs::write(store.join("unrelated"), b"unrecognized").unwrap();
    assert!(Store::open(&store).is_err());
    fs::remove_file(store.join("unrelated")).unwrap();
    assert!(Store::open(&store).is_ok());
}

#[test]
fn an_authorized_expiry_observation_survives_reopen_and_cannot_revive_by_rollback() {
    let f = Fixture::new();
    let policy = Policy(Cell::new(true));
    let data = vec![0x93; 32];
    let b = binding(&data);
    let id = b.id;
    let mut s = Store::open(&f.store()).unwrap();
    s.dispatch(&b.sender, &policy, Request::Begin(b.clone()), 100)
        .unwrap();
    s.dispatch(
        &b.sender,
        &policy,
        Request::PutChunk {
            id,
            offset: 0,
            bytes: data,
        },
        100,
    )
    .unwrap();
    s.dispatch(&b.sender, &policy, Request::Commit { id }, 100)
        .unwrap();
    assert_eq!(
        s.dispatch(
            &b.recipient,
            &policy,
            Request::GetChunk { id, offset: 0 },
            200
        ),
        Err(Error::Expired)
    );
    drop(s);
    let mut reopened = Store::open(&f.store()).unwrap();
    assert_eq!(
        reopened.dispatch(
            &b.recipient,
            &policy,
            Request::GetChunk { id, offset: 0 },
            199
        ),
        Err(Error::Clock)
    );
    assert_eq!(
        reopened.dispatch(
            &b.recipient,
            &policy,
            Request::GetChunk { id, offset: 0 },
            200
        ),
        Err(Error::Expired)
    );
}

#[test]
fn an_unauthorized_future_request_neither_advances_clock_nor_discloses_expiry() {
    let f = Fixture::new();
    let policy = Policy(Cell::new(true));
    let b = binding(&[0; 32]);
    let mut s = Store::open(&f.store()).unwrap();
    s.dispatch(&b.sender, &policy, Request::Begin(b.clone()), 100)
        .unwrap();
    let before = fs::read(f.store().join("private-blobs.snapshot")).unwrap();
    policy.0.set(false);
    assert_eq!(
        s.dispatch(&b.sender, &policy, Request::Status { id: b.id }, 200),
        Err(Error::Denied)
    );
    assert_eq!(
        fs::read(f.store().join("private-blobs.snapshot")).unwrap(),
        before
    );
    drop(s);
    policy.0.set(true);
    let mut reopened = Store::open(&f.store()).unwrap();
    assert!(
        reopened
            .dispatch(&b.sender, &policy, Request::Status { id: b.id }, 199)
            .is_ok()
    );
}

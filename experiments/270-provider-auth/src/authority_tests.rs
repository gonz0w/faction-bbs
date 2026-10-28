//! Deterministic fake verified-certificate observations; these are not TLS proofs.
use crate::authority::*;
use faction_files::private_blobs::{Binding, Error, Principal, Request, Response, Store};
use openssl::sha::sha256;
use std::{
    fs,
    io::Write,
    os::unix::fs::{DirBuilderExt, OpenOptionsExt},
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

static NEXT: AtomicU64 = AtomicU64::new(0);
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let root = std::env::temp_dir().canonicalize().unwrap().join(format!(
            "faction270-owned-{}-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::DirBuilder::new().mode(0o700).create(&root).unwrap();
        let mut marker = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(root.join("OWNED"))
            .unwrap();
        marker
            .write_all(b"issue270 synthetic store fixture")
            .unwrap();
        fs::DirBuilder::new()
            .mode(0o700)
            .create(root.join("store"))
            .unwrap();
        Self(root)
    }
    fn store(&self) -> Store {
        Store::open(&self.0.join("store")).unwrap()
    }
    fn bytes(&self) -> Vec<u8> {
        fs::read(self.0.join("store/private-blobs.snapshot")).unwrap()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        assert_eq!(
            fs::read(self.0.join("OWNED")).unwrap(),
            b"issue270 synthetic store fixture"
        );
        fs::remove_dir_all(&self.0).unwrap();
        assert!(!self.0.exists());
    }
}
fn principal(n: u8) -> Principal {
    Principal {
        account: format!("user{n}@home{n}.test"),
        device: n.into(),
        identity: [n; 32],
    }
}
fn entry(n: u8) -> CredentialBinding {
    CredentialBinding {
        certificate_sha256: [n; 32],
        principal: principal(n),
        valid_from: 100,
        valid_until: 1000,
        approved: true,
        account_enabled: true,
        device_enabled: true,
        generation: 1,
    }
}
fn registry(generation: u64, reset_epoch: u64, mut entries: Vec<CredentialBinding>) -> Registry {
    for e in &mut entries {
        e.generation = generation;
    }
    Registry::new(
        generation,
        reset_epoch,
        entries
            .iter()
            .map(|e| e.principal.account.split_once('@').unwrap().1.to_owned())
            .collect(),
        entries,
    )
    .unwrap()
}
fn authority() -> Authority {
    Authority::new(Some(registry(1, 0, vec![entry(1), entry(2)])))
}
fn fake(a: &mut Authority, c: &Connection, n: u8, wall: u64, mono: u64) -> Handle {
    a.authenticate(c, [n; 32], wall, mono).unwrap()
}
fn binding(bytes: &[u8]) -> Binding {
    Binding {
        sender: principal(1),
        recipient: principal(2),
        id: sha256(bytes),
        size: bytes.len() as u64,
        created: 100,
        expires: 200,
    }
}
fn committed(a: &Authority, c: &Connection, h: &Handle, s: &mut Store, b: &Binding, data: &[u8]) {
    a.dispatch(c, h, s, Request::Begin(b.clone()), 100, 0)
        .unwrap();
    a.dispatch(
        c,
        h,
        s,
        Request::PutChunk {
            id: b.id,
            offset: 0,
            bytes: data.to_vec(),
        },
        100,
        0,
    )
    .unwrap();
    a.dispatch(c, h, s, Request::Commit { id: b.id }, 100, 0)
        .unwrap();
}

#[test]
fn exact_internal_sender_upload_and_committed_recipient_read_preserve_store_retries() {
    let f = Fixture::new();
    let mut s = f.store();
    let mut a = authority();
    let cs = a.open_connection();
    let cr = a.open_connection();
    let hs = fake(&mut a, &cs, 1, 100, 0);
    let hr = fake(&mut a, &cr, 2, 100, 0);
    let data = vec![9; 32];
    let b = binding(&data);
    a.dispatch(&cs, &hs, &mut s, Request::Begin(b.clone()), 100, 0)
        .unwrap();
    assert_eq!(
        a.dispatch(&cr, &hr, &mut s, Request::Status { id: b.id }, 100, 0),
        Err(Error::Denied)
    );
    assert_eq!(
        a.dispatch(
            &cr,
            &hr,
            &mut s,
            Request::GetChunk {
                id: b.id,
                offset: 0
            },
            100,
            0
        ),
        Err(Error::Denied)
    );
    for request in [
        Request::Begin(b.clone()),
        Request::PutChunk {
            id: b.id,
            offset: 0,
            bytes: data.clone(),
        },
        Request::PutChunk {
            id: b.id,
            offset: 0,
            bytes: data.clone(),
        },
        Request::Commit { id: b.id },
        Request::Commit { id: b.id },
    ] {
        a.dispatch(&cs, &hs, &mut s, request, 100, 0).unwrap();
    }
    assert_eq!(
        a.dispatch(
            &cr,
            &hr,
            &mut s,
            Request::GetChunk {
                id: b.id,
                offset: 0
            },
            100,
            0
        ),
        Ok(Response::Chunk {
            offset: 0,
            bytes: data,
            total: 32
        })
    );
    assert!(matches!(
        a.dispatch(&cr, &hr, &mut s, Request::Status { id: b.id }, 100, 0),
        Ok(Response::Progress {
            committed: true,
            ..
        })
    ));
    for request in [
        Request::Begin(b.clone()),
        Request::PutChunk {
            id: b.id,
            offset: 0,
            bytes: vec![9; 32],
        },
        Request::Commit { id: b.id },
        Request::Delete { id: b.id },
    ] {
        assert_eq!(
            a.dispatch(&cr, &hr, &mut s, request, 100, 0),
            Err(Error::Denied)
        );
    }
    assert_eq!(
        a.dispatch(
            &cs,
            &hs,
            &mut s,
            Request::GetChunk {
                id: b.id,
                offset: 0
            },
            100,
            0
        ),
        Err(Error::Denied)
    );
}
#[test]
fn registry_shape_caps_duplicates_and_ambiguous_implicit_mappings_fail() {
    assert!(Registry::new(0, 0, vec![], vec![]).is_err());
    for mutate in 0..9 {
        let mut e = entry(1);
        match mutate {
            0 => e.principal.account = "Bad@home.test".into(),
            1 => e.principal.device = 0,
            2 => e.principal.identity = [0; 32],
            3 => e.valid_until = e.valid_from,
            4 => e.valid_until = e.valid_from - 1,
            5 => e.certificate_sha256 = [0; 32],
            6 => e.generation = 0,
            7 => e.generation = 2,
            _ => e.principal.account = "u@invalid".into(),
        }
        assert!(Registry::new(1, 0, vec!["home1.test".into()], vec![e]).is_err());
    }
    let mut duplicate = entry(1);
    duplicate.principal = principal(2);
    assert!(Registry::new(1, 0, vec!["home1.test".into()], vec![entry(1), duplicate]).is_err());
    let mut ambiguous = entry(1);
    ambiguous.certificate_sha256 = [3; 32];
    ambiguous.principal.identity = [3; 32];
    assert!(Registry::new(1, 0, vec!["home1.test".into()], vec![entry(1), ambiguous]).is_err());
    assert!(Registry::new(1, 0, vec!["home1.test".into(); 2], vec![]).is_err());
    for home in [
        "",
        "Bad.test",
        "-bad.test",
        "u@home.test",
        "localhost",
        "bad..test",
    ] {
        assert!(Registry::new(1, 0, vec![home.into()], vec![]).is_err());
    }
    let entries = (1..=32).map(entry).collect::<Vec<_>>();
    assert!(Registry::new(1, 0, vec!["home1.test".into()], entries.clone()).is_ok());
    let mut excess = entries;
    excess.push(entry(33));
    assert!(Registry::new(1, 0, vec![], excess).is_err());
    assert!(
        Registry::new(
            1,
            0,
            (1..=32).map(|n| format!("home{n}.test")).collect(),
            vec![]
        )
        .is_ok()
    );
    assert!(
        Registry::new(
            1,
            0,
            (1..=33).map(|n| format!("home{n}.test")).collect(),
            vec![]
        )
        .is_err()
    );
}
#[test]
fn unknown_unapproved_disabled_and_credential_validity_boundaries_deny() {
    for variant in 0..6 {
        let mut e = entry(1);
        let wall = match variant {
            0 => 99,
            1 => 1000,
            _ => 100,
        };
        if variant == 2 {
            e.approved = false
        }
        if variant == 3 {
            e.account_enabled = false
        }
        if variant == 4 {
            e.device_enabled = false
        }
        let mut a = Authority::new(Some(registry(1, 0, vec![e])));
        let c = a.open_connection();
        assert!(
            a.authenticate(&c, if variant == 5 { [99; 32] } else { [1; 32] }, wall, 0)
                .is_err()
        );
    }
    for wall in [100, 999] {
        let mut a = authority();
        let c = a.open_connection();
        assert!(a.authenticate(&c, [1; 32], wall, 0).is_ok());
    }
}
#[test]
fn handles_are_bounded_closed_expired_and_bound_to_authority_and_connection() {
    let f = Fixture::new();
    let mut s = f.store();
    let b = binding(&[9; 32]);
    let mut a = authority();
    let c = a.open_connection();
    let h = fake(&mut a, &c, 1, 100, 0);
    let other = a.open_connection();
    assert_eq!(
        a.dispatch(&other, &h, &mut s, Request::Begin(b.clone()), 100, 0),
        Err(Error::Denied)
    );
    let fresh = authority();
    assert_eq!(
        fresh.dispatch(&c, &h, &mut s, Request::Begin(b.clone()), 100, 0),
        Err(Error::Denied)
    );
    assert!(
        a.dispatch(&c, &h, &mut s, Request::Begin(b.clone()), 100, 9999)
            .is_ok()
    );
    assert_eq!(
        a.dispatch(&c, &h, &mut s, Request::Status { id: b.id }, 100, 10000),
        Err(Error::Denied)
    );
    assert_eq!(
        a.dispatch(&c, &h, &mut s, Request::Status { id: b.id }, 100, 0),
        Err(Error::Denied)
    );
    a.close_connection(&c);
    assert!(a.authenticate(&c, [1; 32], 100, 0).is_err());
    assert_eq!(
        a.dispatch(&c, &h, &mut s, Request::Status { id: b.id }, 100, 0),
        Err(Error::Denied)
    );
    let mut a = authority();
    let mut held = Vec::new();
    for _ in 0..8 {
        let c = a.open_connection();
        let h = fake(&mut a, &c, 1, 100, 0);
        held.push((c, h));
    }
    let ninth = a.open_connection();
    assert!(a.authenticate(&ninth, [1; 32], 100, 0).is_err());
    a.close_connection(&held[0].0);
    assert!(a.authenticate(&ninth, [1; 32], 100, 0).is_ok());
}
#[test]
fn forged_binding_account_device_and_full_identity_never_select_actor_or_mutate_store() {
    let f = Fixture::new();
    let mut s = f.store();
    let mut a = authority();
    let c = a.open_connection();
    let h = fake(&mut a, &c, 1, 100, 0);
    let original = binding(&[9; 32]);
    let before = f.bytes();
    for side in [false, true] {
        for field in 0..3 {
            let mut b = original.clone();
            let p = if side {
                &mut b.recipient
            } else {
                &mut b.sender
            };
            match field {
                0 => p.account = "wrong@home1.test".into(),
                1 => p.device = 99,
                _ => p.identity[31] ^= 1,
            }
            assert_eq!(
                a.dispatch(&c, &h, &mut s, Request::Begin(b), 100, 0),
                Err(Error::Denied)
            );
            assert_eq!(f.bytes(), before);
        }
    }
}
#[test]
fn live_account_device_approval_peer_and_identity_revocation_denies_next_operation() {
    for variant in 0..7 {
        let f = Fixture::new();
        let mut s = f.store();
        let mut a = authority();
        let c = a.open_connection();
        let h = fake(&mut a, &c, 1, 100, 0);
        let b = binding(&[9; 32]);
        a.dispatch(&c, &h, &mut s, Request::Begin(b.clone()), 100, 0)
            .unwrap();
        let before = f.bytes();
        let mut entries = vec![entry(1), entry(2)];
        match variant {
            0 => entries[0].account_enabled = false,
            1 => entries[0].device_enabled = false,
            2 => entries[0].approved = false,
            3 => entries[1].account_enabled = false,
            4 => entries[1].device_enabled = false,
            5 => entries[1].approved = false,
            _ => {
                entries.remove(1);
            }
        }
        a.replace_registry(registry(2, 0, entries)).unwrap();
        assert_eq!(
            a.dispatch(
                &c,
                &h,
                &mut s,
                Request::PutChunk {
                    id: b.id,
                    offset: 0,
                    bytes: vec![9; 32]
                },
                101,
                1
            ),
            Err(Error::Denied)
        );
        if variant >= 3 {
            let c2 = a.open_connection();
            let h2 = fake(&mut a, &c2, 1, 101, 1);
            assert_eq!(
                a.dispatch(&c2, &h2, &mut s, Request::Status { id: b.id }, 101, 1),
                Err(Error::Denied)
            );
        }
        assert_eq!(f.bytes(), before);
    }
}
#[test]
fn home_removal_credential_expiry_and_missing_policy_deny_without_clock_mutation() {
    for variant in 0..4 {
        let f = Fixture::new();
        let mut s = f.store();
        let mut a = authority();
        let c = a.open_connection();
        let h = fake(&mut a, &c, 1, 100, 0);
        let b = binding(&[9; 32]);
        a.dispatch(&c, &h, &mut s, Request::Begin(b.clone()), 100, 0)
            .unwrap();
        let before = f.bytes();
        let (wall, mono) = if variant == 0 { (1000, 1) } else { (101, 1) };
        if variant == 1 {
            a.unavailable();
        }
        if variant == 2 {
            a.replace_registry(
                Registry::new(
                    2,
                    0,
                    vec!["home1.test".into()],
                    vec![
                        CredentialBinding {
                            generation: 2,
                            ..entry(1)
                        },
                        CredentialBinding {
                            generation: 2,
                            ..entry(2)
                        },
                    ],
                )
                .unwrap(),
            )
            .unwrap();
        }
        if variant == 3 {
            a = Authority::new(None);
        }
        assert_eq!(
            a.dispatch(&c, &h, &mut s, Request::Status { id: b.id }, wall, mono),
            Err(Error::Denied)
        );
        if variant == 2 {
            let c2 = a.open_connection();
            let h2 = fake(&mut a, &c2, 1, 101, 1);
            assert_eq!(
                a.dispatch(&c2, &h2, &mut s, Request::Status { id: b.id }, 101, 1),
                Err(Error::Denied)
            );
        }
        assert_eq!(f.bytes(), before);
    }
}
#[test]
fn replacement_requires_increasing_generation_exact_existing_mapping_and_explicit_reset_rotation() {
    let f = Fixture::new();
    let mut s = f.store();
    let mut a = authority();
    let c = a.open_connection();
    let h = fake(&mut a, &c, 1, 100, 0);
    assert!(
        a.replace_registry(registry(1, 0, vec![entry(1), entry(2)]))
            .is_err()
    );
    let mut remap = entry(1);
    remap.principal.identity[31] ^= 1;
    assert!(
        a.replace_registry(registry(2, 0, vec![remap, entry(2)]))
            .is_err()
    );
    let mut rotated = entry(1);
    rotated.certificate_sha256 = [3; 32];
    a.replace_registry(registry(2, 0, vec![rotated, entry(2)]))
        .unwrap();
    assert_eq!(
        a.dispatch(&c, &h, &mut s, Request::Begin(binding(&[9; 32])), 100, 0),
        Err(Error::Denied)
    );
    let c2 = a.open_connection();
    assert!(a.authenticate(&c2, [1; 32], 100, 0).is_err());
    let h2 = a.authenticate(&c2, [3; 32], 100, 0).unwrap();
    a.reset(1).unwrap();
    assert_eq!(
        a.dispatch(&c2, &h2, &mut s, Request::Begin(binding(&[9; 32])), 100, 0),
        Err(Error::Denied)
    );
    let c3 = a.open_connection();
    assert!(a.authenticate(&c3, [3; 32], 100, 0).is_err());
    assert!(a.reset(1).is_err());
    assert!(a.replace_registry(registry(4, 0, vec![entry(1)])).is_err());
    // New mail identity needs a new explicitly approved certificate; old history is not remapped.
    let mut new_identity = entry(1);
    new_identity.certificate_sha256 = [4; 32];
    new_identity.principal.identity = [4; 32];
    a.replace_registry(registry(4, 1, vec![new_identity, entry(2)]))
        .unwrap();
    let c4 = a.open_connection();
    let h4 = a.authenticate(&c4, [4; 32], 100, 0).unwrap();
    assert_eq!(
        a.dispatch(&c4, &h4, &mut s, Request::Begin(binding(&[9; 32])), 100, 0),
        Err(Error::Denied)
    );
}
#[test]
fn authorized_store_expiry_semantics_quota_and_dedup_remain_unchanged() {
    let f = Fixture::new();
    let mut s = f.store();
    let mut a = authority();
    let c = a.open_connection();
    let h = fake(&mut a, &c, 1, 100, 0);
    let b = binding(&[9; 32]);
    committed(&a, &c, &h, &mut s, &b, &[9; 32]);
    let before = f.bytes();
    assert_eq!(
        a.dispatch(&c, &h, &mut s, Request::Status { id: b.id }, 200, 1),
        Err(Error::Expired)
    );
    assert_ne!(f.bytes(), before);
    assert_eq!(
        a.dispatch(&c, &h, &mut s, Request::Status { id: b.id }, 199, 2),
        Err(Error::Clock)
    );
    a.dispatch(&c, &h, &mut s, Request::Delete { id: b.id }, 200, 2)
        .unwrap();
    for n in 0..8 {
        let next = Binding {
            id: [n; 32],
            created: 200,
            expires: 300,
            size: 1_048_592,
            ..b.clone()
        };
        a.dispatch(&c, &h, &mut s, Request::Begin(next), 200, 2)
            .unwrap();
    }
    let ninth = Binding {
        id: [99; 32],
        created: 200,
        expires: 300,
        ..b
    };
    assert_eq!(
        a.dispatch(&c, &h, &mut s, Request::Begin(ninth), 200, 2),
        Err(Error::Quota)
    );
}

#[test]
fn retired_digests_never_remap_after_removal_or_reset_and_lifetime_history_is_bounded() {
    let mut a = authority();
    a.replace_registry(registry(2, 0, vec![entry(2)])).unwrap();
    let mut changed = entry(1);
    changed.principal.identity[31] ^= 1;
    assert!(
        a.replace_registry(registry(3, 0, vec![changed.clone(), entry(2)]))
            .is_err()
    );
    a.reset(1).unwrap();
    assert!(
        a.replace_registry(registry(4, 1, vec![changed, entry(2)]))
            .is_err()
    );
    // Explicit reapproval of exactly the same digest/principal remains possible.
    a.replace_registry(registry(4, 1, vec![entry(1), entry(2)]))
        .unwrap();
    let c = a.open_connection();
    assert!(a.authenticate(&c, [1; 32], 100, 0).is_ok());
    let mut a = Authority::new(Some(registry(1, 0, (1..=32).map(entry).collect())));
    a.replace_registry(registry(2, 0, vec![])).unwrap();
    assert!(a.replace_registry(registry(3, 0, vec![entry(33)])).is_err());
    a.replace_registry(registry(3, 0, vec![entry(1)])).unwrap();
}

#[test]
fn contradictory_account_status_across_distinct_devices_is_invalid_policy() {
    let first = entry(1);
    let mut second = entry(2);
    second.principal.account = first.principal.account.clone();
    second.account_enabled = false;
    assert!(Registry::new(1, 0, vec!["home1.test".into()], vec![first, second]).is_err());
}
#[test]
fn an_observed_credential_expiry_is_terminal_even_after_wall_clock_rollback() {
    let f = Fixture::new();
    let mut s = f.store();
    let mut a = authority();
    let c = a.open_connection();
    let h = fake(&mut a, &c, 1, 100, 0);
    let b = binding(&[9; 32]);
    a.dispatch(&c, &h, &mut s, Request::Begin(b.clone()), 100, 0)
        .unwrap();
    let before = f.bytes();
    assert_eq!(
        a.dispatch(&c, &h, &mut s, Request::Status { id: b.id }, 1000, 1),
        Err(Error::Denied)
    );
    assert_eq!(
        a.dispatch(&c, &h, &mut s, Request::Status { id: b.id }, 101, 2),
        Err(Error::Denied)
    );
    assert_eq!(f.bytes(), before);
}

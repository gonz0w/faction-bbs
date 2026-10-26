use faction_core::{Board, Error, RoamingAdmission, RoamingRegistry, Status, request_id};
use std::{fs, path::PathBuf};
struct Fixture {
    path: PathBuf,
    board: Board,
    secret: String,
}
impl Fixture {
    fn new() -> Self {
        let path =
            std::env::temp_dir().join(format!("faction223-core-OWNED-{}", request_id().unwrap()));
        let mut board = Board::open(&path).unwrap();
        let secret = request_id().unwrap();
        board.bootstrap("sysop", &secret).unwrap();
        board.register("member", &secret).unwrap();
        let root = board.authenticate("sysop", &secret).unwrap();
        board.set_status(&root, "member", Status::Approved).unwrap();
        Self {
            path,
            board,
            secret,
        }
    }
    fn binding(&mut self, uid: u32) -> i64 {
        let issuer = self
            .board
            .roaming_add_issuer(
                "home-one",
                "SHA256:AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
            )
            .unwrap();
        let issuer = self
            .board
            .roaming_set_issuer_enabled(issuer.id, issuer.generation, true)
            .unwrap();
        let member = self.board.roaming_owner_member_id("member").unwrap();
        let b = self
            .board
            .roaming_link_member(
                issuer.id,
                issuer.generation,
                "stable-subject",
                "board-one:home-one:stable-subject",
                member,
                uid,
                "local-consent-223",
            )
            .unwrap();
        self.board
            .roaming_set_binding_enabled(b.id, b.generation, true)
            .unwrap()
            .id
    }
    fn install(&mut self) {
        self.board = Board::open(&self.path).unwrap();
        let s = self.board.roaming_owner_state().unwrap();
        self.board
            .configure_roaming_owner(Some(RoamingRegistry {
                board_identity: s.board_identity,
                entries: s
                    .bindings
                    .iter()
                    .filter(|b| b.enabled)
                    .map(|b| RoamingAdmission {
                        uid: b.uid,
                        binding_id: b.id,
                        binding_generation: b.generation,
                        issuer_generation: s
                            .issuers
                            .iter()
                            .find(|i| i.id == b.issuer_id)
                            .unwrap()
                            .generation,
                    })
                    .collect(),
            }))
            .unwrap();
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}
#[test]
fn owner_registry_authenticates_real_sessions_and_revokes_exact_provenance() {
    let mut f = Fixture::new();
    assert!(f.board.authenticate_roaming_uid(42001).is_err());
    let id = f.binding(42001);
    f.install();
    let roaming = f.board.authenticate_roaming_uid(42001).unwrap();
    let password = f.board.authenticate("member", &f.secret).unwrap();
    assert_eq!(f.board.whoami(&roaming).unwrap().handle, "member");
    assert_eq!(f.board.rooms(&roaming).unwrap().len(), 1);
    assert!(f.board.authenticate_roaming_uid(42002).is_err());
    let mut run = f.board.game_start(&roaming, "fixture").unwrap();
    let mut ordinary = f.board.game_start(&password, "fixture").unwrap();
    let lease = f.board.chat_open(&roaming).unwrap();
    let plain_lease = f.board.chat_open(&password).unwrap();
    let b = f
        .board
        .roaming_owner_state()
        .unwrap()
        .bindings
        .into_iter()
        .find(|b| b.id == id)
        .unwrap();
    f.board
        .roaming_set_binding_enabled(id, b.generation, false)
        .unwrap();
    assert!(f.board.whoami(&roaming).is_err());
    assert!(f.board.rooms(&roaming).is_err());
    assert!(f.board.game_finish(&password, &mut run, 3).is_err());
    assert!(f.board.chat_heartbeat(&lease).is_err());
    f.board.chat_heartbeat(&plain_lease).unwrap();
    f.board.game_finish(&password, &mut ordinary, 3).unwrap();
    assert!(f.board.rooms(&password).is_ok());
    let b = f.board.roaming_owner_state().unwrap().bindings.remove(0);
    f.board
        .roaming_set_binding_enabled(id, b.generation, true)
        .unwrap();
    assert!(f.board.authenticate_roaming_uid(42001).is_err());
    assert!(f.board.whoami(&roaming).is_err());
}
#[test]
fn owner_registry_is_once_before_auth_and_rejects_wrong_board_or_generation() {
    let mut f = Fixture::new();
    f.binding(42001);
    let state = f.board.roaming_owner_state().unwrap();
    assert!(
        f.board
            .configure_roaming_owner(Some(RoamingRegistry {
                board_identity: [0; 32],
                entries: vec![]
            }))
            .is_err()
    );
    // Setup has password-authenticated; use a fresh owner handle before installing.
    let mut fresh = Board::open(&f.path).unwrap();
    let b = &state.bindings[0];
    let i = &state.issuers[0];
    assert!(
        fresh
            .configure_roaming_owner(Some(RoamingRegistry {
                board_identity: state.board_identity,
                entries: vec![RoamingAdmission {
                    uid: b.uid,
                    binding_id: b.id,
                    binding_generation: b.generation + 1,
                    issuer_generation: i.generation
                }]
            }))
            .is_err()
    );
    f.board = Board::open(&f.path).unwrap();
    f.install();
    assert!(f.board.configure_roaming_owner(None).is_err());
    let s = f.board.authenticate_roaming_uid(42001).unwrap();
    let root = f.board.authenticate("sysop", &f.secret).unwrap();
    f.board
        .set_status(&root, "member", Status::Revoked)
        .unwrap();
    assert!(matches!(
        f.board.whoami(&s),
        Err(Error::StaleSession) | Err(Error::Denied)
    ));
    assert!(f.board.authenticate_roaming_uid(42001).is_err());
}
#[test]
fn issuer_revoke_disables_bindings_and_export_carries_no_enabled_authority() {
    let mut f = Fixture::new();
    f.binding(42001);
    f.install();
    let old = f.board.roaming_owner_state().unwrap();
    let export = f.path.join("disabled-authority-export.sqlite3");
    f.board.export_local(&export).unwrap();
    let db = rusqlite::Connection::open(&export).unwrap();
    assert_eq!(
        db.query_row(
            "SELECT count(*) FROM roaming_bindings WHERE enabled=1",
            [],
            |r| r.get::<_, i64>(0)
        )
        .unwrap(),
        0
    );
    assert_eq!(
        db.query_row(
            "SELECT count(*) FROM roaming_issuers WHERE enabled=1",
            [],
            |r| r.get::<_, i64>(0)
        )
        .unwrap(),
        0
    );
    let issuer = &old.issuers[0];
    f.board
        .roaming_set_issuer_enabled(issuer.id, issuer.generation, false)
        .unwrap();
    let next = f.board.roaming_owner_state().unwrap();
    assert!(!next.bindings[0].enabled);
    assert!(next.bindings[0].generation > old.bindings[0].generation);
    let issuer = &next.issuers[0];
    f.board
        .roaming_set_issuer_enabled(issuer.id, issuer.generation, true)
        .unwrap();
    assert!(!f.board.roaming_owner_state().unwrap().bindings[0].enabled);
}
#[test]
fn issuer_disablement_rotates_every_binding_before_reenable() {
    let mut f = Fixture::new();
    f.binding(42001);
    let old = f.board.roaming_owner_state().unwrap();
    let i = &old.issuers[0];
    f.board
        .roaming_set_issuer_enabled(i.id, i.generation, false)
        .unwrap();
    let next = f.board.roaming_owner_state().unwrap();
    assert!(!next.bindings[0].enabled);
    assert!(next.bindings[0].generation > old.bindings[0].generation);
}
fn transfer_key(seed: u8) -> String {
    use base64::{Engine, engine::general_purpose::STANDARD};
    let mut bytes = b"\0\0\0\x0bssh-ed25519\0\0\0\x20".to_vec();
    bytes.extend([seed; 32]);
    format!("ssh-ed25519 {}", STANDARD.encode(bytes))
}
#[test]
fn bound_transfer_grants_recheck_after_disable_without_revoking_password_grants() {
    let mut f = Fixture::new();
    f.binding(42001);
    f.install();
    let roaming = f.board.authenticate_roaming_uid(42001).unwrap();
    let password = f.board.authenticate("member", &f.secret).unwrap();
    let root = f.board.authenticate("sysop", &f.secret).unwrap();
    let area = f
        .board
        .file_create_area(&root, "uploads", "Uploads", false, 1000)
        .unwrap();
    let bound = f
        .board
        .file_upload(&roaming, area, "bound.bin", "bound", 3, &transfer_key(1))
        .unwrap();
    let bound_grant = f.board.file_claim(&bound).unwrap();
    let ordinary = f
        .board
        .file_upload(
            &password,
            area,
            "ordinary.bin",
            "ordinary",
            3,
            &transfer_key(2),
        )
        .unwrap();
    let ordinary_grant = f.board.file_claim(&ordinary).unwrap();
    f.board
        .file_chunk(&bound, &bound_grant.lease, 0, 1)
        .unwrap();
    let b = f.board.roaming_owner_state().unwrap().bindings.remove(0);
    f.board
        .roaming_set_binding_enabled(b.id, b.generation, false)
        .unwrap();
    assert!(f.board.file_check(&bound, &bound_grant.lease).is_err());
    assert!(
        f.board
            .file_chunk(&bound, &bound_grant.lease, 1, 2)
            .is_err()
    );
    assert!(
        f.board
            .file_complete(&bound, &bound_grant.lease, &"a".repeat(64))
            .is_err()
    );
    f.board
        .file_chunk(&ordinary, &ordinary_grant.lease, 0, 3)
        .unwrap();
    f.board
        .file_complete(&ordinary, &ordinary_grant.lease, &"a".repeat(64))
        .unwrap();
    let db = rusqlite::Connection::open(f.path.join("faction.sqlite3")).unwrap();
    assert_eq!(
        db.query_row(
            "SELECT state FROM file_transfers WHERE id=?1",
            [bound],
            |r| r.get::<_, String>(0)
        )
        .unwrap(),
        "failed"
    );
}
#[test]
fn issuer_revocation_removes_only_exact_bound_leases_and_ready_grants() {
    let mut f = Fixture::new();
    f.binding(42001);
    f.install();
    let s = f.board.authenticate_roaming_uid(42001).unwrap();
    let ordinary = f.board.authenticate("member", &f.secret).unwrap();
    let root = f.board.authenticate("sysop", &f.secret).unwrap();
    let area = f
        .board
        .file_create_area(&root, "uploads", "Uploads", false, 1000)
        .unwrap();
    let bound = f
        .board
        .file_upload(&s, area, "bound.bin", "bound", 3, &transfer_key(3))
        .unwrap();
    let plain = f
        .board
        .file_upload(&ordinary, area, "plain.bin", "plain", 3, &transfer_key(4))
        .unwrap();
    let l = f.board.chat_open(&s).unwrap();
    let plain_lease = f.board.chat_open(&ordinary).unwrap();
    let mut game = f.board.game_start(&s, "fixture").unwrap();
    let i = f.board.roaming_owner_state().unwrap().issuers.remove(0);
    f.board
        .roaming_set_issuer_enabled(i.id, i.generation, false)
        .unwrap();
    assert!(f.board.whoami(&s).is_err());
    assert!(f.board.file_claim(&bound).is_err());
    assert_eq!(
        f.board
            .file_lookup_key(transfer_key(3).split_whitespace().nth(1).unwrap())
            .unwrap(),
        None
    );
    f.board.file_claim(&plain).unwrap();
    assert!(f.board.chat_heartbeat(&l).is_err());
    f.board.chat_heartbeat(&plain_lease).unwrap();
    assert!(f.board.game_finish(&ordinary, &mut game, 10).is_err());
    let db = rusqlite::Connection::open(f.path.join("faction.sqlite3")).unwrap();
    assert_eq!(
        db.query_row("SELECT count(*) FROM chat_sessions", [], |r| r
            .get::<_, i64>(0))
            .unwrap(),
        1
    );
    assert_eq!(
        db.query_row("SELECT count(*) FROM roaming_chat", [], |r| r
            .get::<_, i64>(0))
            .unwrap(),
        0
    );
}
#[test]
fn owner_association_needs_approval_consent_exact_generation_and_permanent_unused_uid() {
    let mut f = Fixture::new();
    let i = f
        .board
        .roaming_add_issuer(
            "home-one",
            "SHA256:AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
        )
        .unwrap();
    assert!(!i.enabled);
    let member = f.board.roaming_owner_member_id("member").unwrap();
    assert!(
        f.board
            .roaming_link_member(
                i.id,
                i.generation,
                "stable",
                "board:home:stable",
                member,
                42001,
                "consent"
            )
            .is_err()
    );
    assert!(
        f.board
            .roaming_set_issuer_enabled(i.id, i.generation + 1, true)
            .is_err()
    );
    let i = f
        .board
        .roaming_set_issuer_enabled(i.id, i.generation, true)
        .unwrap();
    assert!(
        f.board
            .roaming_link_member(
                i.id,
                i.generation,
                "stable",
                "board:home:stable",
                member,
                42001,
                ""
            )
            .is_err()
    );
    assert!(
        f.board
            .roaming_link_member(
                i.id,
                i.generation,
                "stable",
                "board:home:stable",
                member,
                0,
                "consent"
            )
            .is_err()
    );
    f.board.register("pending", &f.secret).unwrap();
    let pending = f.board.roaming_owner_member_id("pending").unwrap();
    assert!(
        f.board
            .roaming_link_member(
                i.id,
                i.generation,
                "pending",
                "board:home:pending",
                pending,
                42002,
                "consent"
            )
            .is_err()
    );
    let b = f
        .board
        .roaming_link_member(
            i.id,
            i.generation,
            "stable",
            "board:home:stable",
            member,
            42001,
            "consent",
        )
        .unwrap();
    assert!(!b.enabled);
    let enabled = f
        .board
        .roaming_set_binding_enabled(b.id, b.generation, true)
        .unwrap();
    assert!(
        f.board
            .roaming_set_binding_enabled(b.id, b.generation, false)
            .is_err()
    );
    let disabled = f
        .board
        .roaming_set_binding_enabled(b.id, enabled.generation, false)
        .unwrap();
    assert!(disabled.generation > enabled.generation);
    assert!(
        f.board
            .roaming_link_member(
                i.id,
                i.generation,
                "other",
                "board:home:other",
                member,
                42001,
                "new-consent"
            )
            .is_err()
    );
    // A fresh disabled record may retain a retired identity/principal. It has
    // its own immutable UID/consent and cannot be enabled alongside a conflict.
    let relink = f
        .board
        .roaming_link_member(
            i.id,
            i.generation,
            "stable",
            "board:home:other",
            member,
            42003,
            "new-consent",
        )
        .unwrap();
    let other = f
        .board
        .roaming_link_member(
            i.id,
            i.generation,
            "other",
            "board:home:stable",
            member,
            42004,
            "new-consent",
        )
        .unwrap();
    assert!(!relink.enabled);
    assert!(!other.enabled);
    let state = f.board.roaming_owner_state().unwrap();
    assert_eq!(state.bindings.len(), 3);
    assert_eq!(state.bindings[0].subject, "stable");
    assert_eq!(state.bindings[0].uid, 42001);
}
#[test]
fn bounded_owner_registry_and_uid_tombstones_survive_restart() {
    let mut f = Fixture::new();
    let i = f
        .board
        .roaming_add_issuer(
            "home-one",
            "SHA256:AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
        )
        .unwrap();
    let i = f
        .board
        .roaming_set_issuer_enabled(i.id, i.generation, true)
        .unwrap();
    f.board
        .roaming_add_issuer(
            "home-two",
            "SHA256:BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB",
        )
        .unwrap();
    assert!(
        f.board
            .roaming_add_issuer(
                "third",
                "SHA256:CCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCC"
            )
            .is_err()
    );
    let member = f.board.roaming_owner_member_id("member").unwrap();
    for n in 0..32 {
        let b = f
            .board
            .roaming_link_member(
                i.id,
                i.generation,
                &format!("subject-{n}"),
                &format!("board:home:{n}"),
                member,
                42000 + n,
                "consent",
            )
            .unwrap();
        if n < 4 {
            f.board
                .roaming_set_binding_enabled(b.id, b.generation, true)
                .unwrap();
        } else if n == 4 {
            assert!(
                f.board
                    .roaming_set_binding_enabled(b.id, b.generation, true)
                    .is_err()
            );
        }
    }
    assert!(
        f.board
            .roaming_link_member(
                i.id,
                i.generation,
                "subject-33",
                "board:home:33",
                member,
                43000,
                "consent"
            )
            .is_err()
    );
    f.board = Board::open(&f.path).unwrap();
    let s = f.board.roaming_owner_state().unwrap();
    assert_eq!(s.bindings.len(), 32);
    assert_eq!(s.bindings.iter().filter(|b| b.enabled).count(), 4);
    let b = &s.bindings[0];
    f.board
        .roaming_set_binding_enabled(b.id, b.generation, false)
        .unwrap();
    assert!(
        f.board
            .roaming_link_member(
                i.id,
                i.generation,
                "replacement",
                "board:home:replacement",
                member,
                b.uid,
                "consent"
            )
            .is_err()
    );
}
#[test]
fn trusted_registry_is_default_off_late_locked_and_checks_all_entry_fields() {
    let mut f = Fixture::new();
    f.binding(42001);
    assert!(f.board.configure_roaming_owner(None).is_err());
    let s = f.board.roaming_owner_state().unwrap();
    let b = &s.bindings[0];
    let i = &s.issuers[0];
    let entry = RoamingAdmission {
        uid: b.uid,
        binding_id: b.id,
        binding_generation: b.generation,
        issuer_generation: i.generation,
    };
    for entries in [
        vec![RoamingAdmission {
            uid: b.uid + 1,
            ..entry.clone()
        }],
        vec![RoamingAdmission {
            binding_id: b.id + 1,
            ..entry.clone()
        }],
        vec![RoamingAdmission {
            issuer_generation: i.generation + 1,
            ..entry.clone()
        }],
        vec![entry.clone(), entry.clone()],
        vec![entry.clone(); 5],
    ] {
        let mut owner = Board::open(&f.path).unwrap();
        assert!(
            owner
                .configure_roaming_owner(Some(RoamingRegistry {
                    board_identity: s.board_identity,
                    entries
                }))
                .is_err()
        );
    }
    let mut off = Board::open(&f.path).unwrap();
    off.configure_roaming_owner(None).unwrap();
    assert!(off.authenticate_roaming_uid(b.uid).is_err());
    assert!(
        off.configure_roaming_owner(Some(RoamingRegistry {
            board_identity: s.board_identity,
            entries: vec![entry]
        }))
        .is_err()
    );
    f.install();
    let session = f.board.authenticate_roaming_uid(b.uid).unwrap();
    assert_eq!(f.board.rooms(&session).unwrap().len(), 1);
    let mut independent = Fixture::new();
    assert!(independent.board.whoami(&session).is_err());
}
#[test]
fn restart_and_restore_require_manual_reconciliation_and_fresh_board_identity() {
    use faction_core::{restore_local, verify_export};
    let mut f = Fixture::new();
    f.binding(42001);
    f.install();
    let old = f.board.roaming_owner_state().unwrap();
    let s = f.board.authenticate_roaming_uid(42001).unwrap();
    let lease = f.board.chat_open(&s).unwrap();
    let mut game = f.board.game_start(&s, "fixture").unwrap();
    f.board = Board::open(&f.path).unwrap();
    assert!(f.board.authenticate_roaming_uid(42001).is_err());
    f.install();
    let fresh = f.board.authenticate_roaming_uid(42001).unwrap();
    assert!(f.board.rooms(&fresh).is_ok());
    let export = f.path.join("export.sqlite3");
    f.board.export_local(&export).unwrap();
    verify_export(&export).unwrap();
    let target = f.path.join("restored");
    restore_local(&export, &target).unwrap();
    let mut restored = Board::open(&target).unwrap();
    let state = restored.roaming_owner_state().unwrap();
    assert_ne!(state.board_identity, old.board_identity);
    assert!(!state.bindings[0].enabled);
    assert!(!state.issuers[0].enabled);
    assert_eq!(state.bindings[0].subject, old.bindings[0].subject);
    assert_eq!(state.bindings[0].principal, old.bindings[0].principal);
    assert_eq!(state.bindings[0].uid, old.bindings[0].uid);
    assert!(state.bindings[0].generation > old.bindings[0].generation);
    assert!(state.issuers[0].generation > old.issuers[0].generation);
    assert!(restored.whoami(&s).is_err());
    assert!(restored.chat_heartbeat(&lease).is_err());
    assert!(restored.game_finish(&s, &mut game, 5).is_err());
    assert!(
        restored
            .configure_roaming_owner(Some(RoamingRegistry {
                board_identity: old.board_identity,
                entries: vec![]
            }))
            .is_err()
    );
    assert!(restored.authenticate_roaming_uid(42001).is_err());
    let i = restored
        .roaming_set_issuer_enabled(state.issuers[0].id, state.issuers[0].generation, true)
        .unwrap();
    let b = restored
        .roaming_set_binding_enabled(state.bindings[0].id, state.bindings[0].generation, true)
        .unwrap();
    let member = restored.roaming_owner_member_id("member").unwrap();
    assert!(
        restored
            .roaming_link_member(
                i.id,
                i.generation,
                "newsubject",
                "newprincipal",
                member,
                b.uid,
                "newconsent"
            )
            .is_err()
    );
    let mut reconciled = Board::open(&target).unwrap();
    reconciled
        .configure_roaming_owner(Some(RoamingRegistry {
            board_identity: state.board_identity,
            entries: vec![RoamingAdmission {
                uid: b.uid,
                binding_id: b.id,
                binding_generation: b.generation,
                issuer_generation: i.generation,
            }],
        }))
        .unwrap();
    let fresh = reconciled.authenticate_roaming_uid(42001).unwrap();
    assert!(reconciled.rooms(&fresh).is_ok());
    assert!(reconciled.rooms(&s).is_err());
    let db = rusqlite::Connection::open(export).unwrap();
    assert_eq!(
        db.query_row("SELECT count(*) FROM roaming_chat", [], |r| r
            .get::<_, i64>(0))
            .unwrap(),
        0
    );
}
#[test]
fn snapshot_rejects_malformed_identity_metadata_authority_and_catalog() {
    use faction_core::verify_export;
    let f = Fixture::new();
    let mut f = f;
    f.binding(42001);
    f.install();
    for (n,sql) in ["UPDATE roaming_issuers SET ca_fingerprint='not-a-fingerprint'","UPDATE roaming_bindings SET consent=''","UPDATE roaming_bindings SET subject=char(27)","UPDATE roaming_bindings SET principal='space principal'","UPDATE roaming_issuers SET enabled=1","UPDATE roaming_bindings SET enabled=1","PRAGMA ignore_check_constraints=ON;UPDATE roaming_bindings SET generation=0",
        "PRAGMA ignore_check_constraints=ON;UPDATE roaming_issuers SET generation=0",
        "PRAGMA ignore_check_constraints=ON;UPDATE roaming_bindings SET enabled=2",
        "PRAGMA ignore_check_constraints=ON;UPDATE roaming_issuers SET enabled=2",
        "PRAGMA ignore_check_constraints=ON;UPDATE roaming_bindings SET uid=0","PRAGMA foreign_keys=OFF;UPDATE roaming_bindings SET issuer_id=999","CREATE TABLE extra_roaming(x)","INSERT INTO roaming_issuers VALUES(2,'two','SHA256:BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB',1,0);INSERT INTO roaming_issuers VALUES(3,'three','SHA256:CCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCC',1,0)"].iter().enumerate(){let export=f.path.join(format!("invalid-{n}.sqlite3"));f.board.export_local(&export).unwrap();let db=rusqlite::Connection::open(&export).unwrap();db.execute_batch(sql).unwrap();drop(db);assert!(verify_export(&export).is_err(),"accepted {sql}");}
}
#[test]
fn generation_exhaustion_is_atomic_and_never_wraps_or_exports() {
    let mut f = Fixture::new();
    f.binding(42001);
    let db = rusqlite::Connection::open(f.path.join("faction.sqlite3")).unwrap();
    db.execute("UPDATE roaming_bindings SET generation=?1", [i64::MAX])
        .unwrap();
    let before = f.board.roaming_owner_state().unwrap();
    let b = &before.bindings[0];
    let i = &before.issuers[0];
    assert!(
        f.board
            .roaming_set_binding_enabled(b.id, i64::MAX, false)
            .is_err()
    );
    assert!(
        f.board
            .roaming_set_issuer_enabled(i.id, i.generation, false)
            .is_err()
    );
    assert!(
        f.board
            .export_local(&f.path.join("max-generation.sqlite3"))
            .is_err()
    );
    let after = f.board.roaming_owner_state().unwrap();
    assert_eq!(after.bindings[0].generation, i64::MAX);
    assert!(after.bindings[0].enabled);
    assert!(after.issuers[0].enabled);
    assert_eq!(after.issuers[0].generation, i.generation);
}
#[test]
fn schema19_stays_readonly_on_export_and_migrates_atomically_to20() {
    use faction_core::{export_local, restore_local, verify_export};
    let f = Fixture::new();
    let path = f.path.clone();
    let db = rusqlite::Connection::open(path.join("faction.sqlite3")).unwrap();
    db.execute_batch("DROP TABLE roaming_chat;DROP TABLE roaming_transfers;DROP TABLE roaming_bindings;DROP TABLE roaming_issuers;PRAGMA user_version=19").unwrap();
    let export = path.join("legacy19.sqlite3");
    export_local(&path, &export).unwrap();
    verify_export(&export).unwrap();
    assert_eq!(
        db.query_row("PRAGMA user_version", [], |r| r.get::<_, i64>(0))
            .unwrap(),
        19
    );
    let restored = path.join("restored19");
    restore_local(&export, &restored).unwrap();
    let mut b = Board::open(&restored).unwrap();
    let s = b.authenticate("member", &f.secret).unwrap();
    assert_eq!(b.rooms(&s).unwrap().len(), 1);
    db.execute_batch("CREATE TABLE roaming_issuers(unrelated TEXT)")
        .unwrap();
    assert!(Board::open(&path).is_err());
    assert_eq!(
        db.query_row("PRAGMA user_version", [], |r| r.get::<_, i64>(0))
            .unwrap(),
        19
    );
    db.execute_batch("DROP TABLE roaming_issuers").unwrap();
    let migrated = Board::open(&path).unwrap();
    assert!(migrated.roaming_owner_state().unwrap().bindings.is_empty());
    assert_eq!(
        db.query_row("PRAGMA user_version", [], |r| r.get::<_, i64>(0))
            .unwrap(),
        20
    );
}
#[test]
fn independent_issuers_keep_same_member_password_and_other_home_authority() {
    let mut f = Fixture::new();
    f.binding(42001);
    let i = f
        .board
        .roaming_add_issuer(
            "home-two",
            "SHA256:BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB",
        )
        .unwrap();
    let i = f
        .board
        .roaming_set_issuer_enabled(i.id, i.generation, true)
        .unwrap();
    let member = f.board.roaming_owner_member_id("member").unwrap();
    let b = f
        .board
        .roaming_link_member(
            i.id,
            i.generation,
            "same-display-handle",
            "board:home-two:stable-subject",
            member,
            42002,
            "independent-local-consent",
        )
        .unwrap();
    f.board
        .roaming_set_binding_enabled(b.id, b.generation, true)
        .unwrap();
    f.install();
    let first = f.board.authenticate_roaming_uid(42001).unwrap();
    let second = f.board.authenticate_roaming_uid(42002).unwrap();
    let password = f.board.authenticate("member", &f.secret).unwrap();
    let a = f.board.chat_open(&first).unwrap();
    let b = f.board.chat_open(&second).unwrap();
    let c = f.board.chat_open(&password).unwrap();
    let first_issuer = f.board.roaming_owner_state().unwrap().issuers.remove(0);
    f.board
        .roaming_set_issuer_enabled(first_issuer.id, first_issuer.generation, false)
        .unwrap();
    assert!(f.board.rooms(&first).is_err());
    assert!(f.board.chat_heartbeat(&a).is_err());
    assert!(f.board.rooms(&second).is_ok());
    assert!(f.board.rooms(&password).is_ok());
    f.board.chat_heartbeat(&b).unwrap();
    f.board.chat_heartbeat(&c).unwrap();
    let db = rusqlite::Connection::open(f.path.join("faction.sqlite3")).unwrap();
    assert_eq!(
        db.query_row("SELECT count(*) FROM chat_sessions", [], |r| r
            .get::<_, i64>(0))
            .unwrap(),
        2
    );
}
#[test]
fn disabled_identity_relinks_with_fresh_uid_consent_and_never_reuses_old_uid() {
    let mut f = Fixture::new();
    f.binding(42001);
    f.install();
    let old_session = f.board.authenticate_roaming_uid(42001).unwrap();
    let state = f.board.roaming_owner_state().unwrap();
    let old = &state.bindings[0];
    let issuer = &state.issuers[0];
    f.board
        .roaming_set_binding_enabled(old.id, old.generation, false)
        .unwrap();
    let new = f
        .board
        .roaming_link_member(
            issuer.id,
            issuer.generation,
            &old.subject,
            &old.principal,
            old.member_id,
            42002,
            "fresh-consent-223",
        )
        .unwrap();
    assert_ne!(new.id, old.id);
    assert!(!new.enabled);
    let new = f
        .board
        .roaming_set_binding_enabled(new.id, new.generation, true)
        .unwrap();
    assert!(f.board.whoami(&old_session).is_err());
    assert!(f.board.authenticate_roaming_uid(42001).is_err());
    assert!(
        f.board
            .roaming_link_member(
                issuer.id,
                issuer.generation,
                "other-subject",
                "other-principal",
                old.member_id,
                42001,
                "other-consent"
            )
            .is_err()
    );
    let old_current = f
        .board
        .roaming_owner_state()
        .unwrap()
        .bindings
        .into_iter()
        .find(|b| b.id == old.id)
        .unwrap();
    assert_eq!(old_current.uid, 42001);
    assert_eq!(old_current.subject, old.subject);
    assert_eq!(old_current.consent, old.consent);
    assert!(!old_current.enabled);
    assert!(
        f.board
            .roaming_set_binding_enabled(old.id, old_current.generation, true)
            .is_err()
    );
    f.board = Board::open(&f.path).unwrap();
    f.board
        .configure_roaming_owner(Some(RoamingRegistry {
            board_identity: state.board_identity,
            entries: vec![RoamingAdmission {
                uid: new.uid,
                binding_id: new.id,
                binding_generation: new.generation,
                issuer_generation: issuer.generation,
            }],
        }))
        .unwrap();
    let session = f.board.authenticate_roaming_uid(42002).unwrap();
    assert_eq!(f.board.rooms(&session).unwrap().len(), 1);
    assert!(f.board.authenticate_roaming_uid(42001).is_err());
    assert!(f.board.whoami(&old_session).is_err());
}

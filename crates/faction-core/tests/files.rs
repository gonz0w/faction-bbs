use base64::{Engine, engine::general_purpose::STANDARD};
use faction_core::{
    Board, Error, FILE_BYTES, Session, Status, request_id, restore_local, verify_export,
};
use std::{fs, os::unix::fs::DirBuilderExt, path::PathBuf};
fn key(seed: u8) -> String {
    let mut b = b"\0\0\0\x0bssh-ed25519\0\0\0\x20".to_vec();
    b.extend([seed; 32]);
    format!("ssh-ed25519 {} test", STANDARD.encode(b))
}
struct F {
    root: PathBuf,
    board: Board,
    sysop: Session,
    alice: Session,
    bob: Session,
    pending: Session,
    secret: String,
}
impl F {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!("faction-files-{}", request_id().unwrap()));
        fs::DirBuilder::new().mode(0o700).create(&root).unwrap();
        let mut b = Board::open(&root.join("board")).unwrap();
        let secret = request_id().unwrap();
        b.bootstrap("sysop", &secret).unwrap();
        let sysop = b.authenticate("sysop", &secret).unwrap();
        let mut sessions = Vec::new();
        for name in ["alice", "bobby", "pending"] {
            b.register(name, &secret).unwrap();
            if name != "pending" {
                b.set_status(&sysop, name, Status::Approved).unwrap()
            }
            sessions.push(b.authenticate(name, &secret).unwrap())
        }
        let mut it = sessions.into_iter();
        Self {
            root,
            board: b,
            sysop,
            alice: it.next().unwrap(),
            bob: it.next().unwrap(),
            pending: it.next().unwrap(),
            secret,
        }
    }
    fn area(&mut self, private: bool, quota: u32) -> i64 {
        self.board
            .file_create_area(&self.sysop, "downloads", "Downloads", private, quota)
            .unwrap()
    }
    fn complete(&mut self, id: &str, bytes: u32) -> String {
        let g = self.board.file_claim(id).unwrap();
        self.board.file_chunk(id, &g.lease, 0, bytes).unwrap();
        self.board
            .file_complete(id, &g.lease, &"a".repeat(64))
            .unwrap();
        g.object
    }
}
impl Drop for F {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}
#[test]
fn approval_private_scope_review_and_search_are_local() {
    let mut f = F::new();
    let a = f.area(true, 100);
    assert!(f.board.file_areas(&f.pending, 0).is_err());
    assert!(f.board.file_areas(&f.alice, 0).unwrap().is_empty());
    assert!(
        f.board
            .file_upload(&f.alice, a, "x.bin", "description", 3, &key(1))
            .is_err()
    );
    f.board.file_allow(&f.sysop, a, "alice", true).unwrap();
    let id = f
        .board
        .file_upload(&f.alice, a, "x.bin", "a useful artifact", 3, &key(1))
        .unwrap();
    assert!(
        f.board
            .file_list(&f.alice, a, "", "", false)
            .unwrap()
            .is_empty()
    );
    let o = f.complete(&id, 3);
    assert!(f.board.file_info(&f.alice, &o).is_err());
    assert!(f.board.file_download(&f.alice, &o, &key(12)).is_err());
    assert_eq!(f.board.file_info(&f.sysop, &o).unwrap().state, "pending");
    let review = f.board.file_download(&f.sysop, &o, &key(13)).unwrap();
    let reviewing = f.board.file_claim(&review).unwrap();
    f.board.file_check(&review, &reviewing.lease).unwrap();
    f.board.file_cancel(&review, &reviewing.lease).unwrap();
    assert!(f.board.file_review(&f.alice, &o, true).is_err());
    f.board.file_review(&f.sysop, &o, true).unwrap();
    assert_eq!(
        f.board
            .file_list(&f.alice, a, "", "useful", false)
            .unwrap()
            .len(),
        1
    );
    assert!(f.board.file_download(&f.bob, &o, &key(2)).is_err());
    assert!(f.board.file_list(&f.bob, a, "", "", true).is_err());
    assert_eq!(
        f.board.file_info(&f.alice, &o).unwrap().sha256,
        "a".repeat(64)
    );
}
#[test]
fn transfers_are_key_bound_one_shot_and_recheck_approval() {
    let mut f = F::new();
    let a = f.area(false, 100);
    let id = f
        .board
        .file_upload(&f.alice, a, "x.bin", "description", 3, &key(1))
        .unwrap();
    let blob = key(1).split_whitespace().nth(1).unwrap().to_string();
    assert_eq!(f.board.file_lookup_key(&blob).unwrap(), Some(id.clone()));
    assert_eq!(
        f.board
            .file_lookup_key(key(2).split_whitespace().nth(1).unwrap())
            .unwrap(),
        None
    );
    assert!(f.board.file_claim(&"0".repeat(32)).is_err());
    let g = f.board.file_claim(&id).unwrap();
    assert!(f.board.file_claim(&id).is_err());
    assert!(f.board.file_check(&id, &"0".repeat(32)).is_err());
    assert!(f.board.file_chunk(&id, &g.lease, 1, 1).is_err());
    assert!(f.board.file_chunk(&id, &g.lease, 0, 4).is_err());
    f.board
        .set_status(&f.sysop, "alice", Status::Revoked)
        .unwrap();
    assert!(f.board.file_check(&id, &g.lease).is_err());
    f.board
        .set_status(&f.sysop, "alice", Status::Approved)
        .unwrap();
    assert!(f.board.file_check(&id, &g.lease).is_err());
}
#[test]
fn private_deny_regrant_does_not_revive_a_transfer() {
    let mut f = F::new();
    let a = f.area(true, 100);
    f.board.file_allow(&f.sysop, a, "alice", true).unwrap();
    let id = f
        .board
        .file_upload(&f.alice, a, "x.bin", "description", 3, &key(1))
        .unwrap();
    let g = f.board.file_claim(&id).unwrap();
    f.board.file_allow(&f.sysop, a, "alice", false).unwrap();
    assert!(f.board.file_check(&id, &g.lease).is_err());
    f.board.file_allow(&f.sysop, a, "alice", true).unwrap();
    assert!(f.board.file_check(&id, &g.lease).is_err());
}
#[test]
fn quotas_remain_reserved_after_failure_and_remove_until_reclaimed() {
    let mut f = F::new();
    let a = f.area(false, 5);
    let id = f
        .board
        .file_upload(&f.alice, a, "x.bin", "description", 4, &key(1))
        .unwrap();
    let g = f.board.file_claim(&id).unwrap();
    f.board.file_cancel(&id, &g.lease).unwrap();
    assert!(
        f.board
            .file_upload(&f.bob, a, "y.bin", "description", 2, &key(2))
            .is_err()
    );
    f.board.file_review(&f.sysop, &g.object, false).unwrap();
    assert!(
        f.board
            .file_upload(&f.bob, a, "y.bin", "description", 2, &key(2))
            .is_err()
    );
    f.board.file_reclaimed(&g.object).unwrap();
    assert!(
        f.board
            .file_upload(&f.bob, a, "y.bin", "description", 2, &key(2))
            .is_ok()
    );
}
#[test]
fn unsafe_names_keys_sizes_and_metadata_are_rejected() {
    let mut f = F::new();
    let a = f.area(false, 100);
    for n in [
        "../x",
        "/etc/passwd",
        ".hidden",
        "a/b",
        "a\\b",
        "x\x1b[0m",
        "two names",
    ] {
        assert!(
            f.board
                .file_upload(&f.alice, a, n, "description", 1, &key(1))
                .is_err(),
            "{n}"
        )
    }
    for k in [
        "command=bad ssh-ed25519 AAAA",
        "ssh-rsa AAAA",
        "ssh-ed25519 AAAA",
    ] {
        assert!(
            f.board
                .file_upload(&f.alice, a, "x.bin", "description", 1, k)
                .is_err()
        )
    }
    assert!(
        f.board
            .file_upload(&f.alice, a, "x.bin", "description", FILE_BYTES + 1, &key(1))
            .is_err()
    );
    assert!(
        f.board
            .file_upload(&f.alice, a, "x.bin", "bad\x1b", 1, &key(1))
            .is_err()
    );
    assert!(f.board.file_info(&f.alice, "../x").is_err());
    assert!(f.board.file_list(&f.alice, a, "bad", "", false).is_err());
}
#[test]
fn snapshot_keeps_catalog_and_scores_but_discards_transfer_authority() {
    let mut f = F::new();
    let a = f.area(false, 100);
    let id = f
        .board
        .file_upload(&f.alice, a, "x.bin", "description", 3, &key(1))
        .unwrap();
    let o = f.complete(&id, 3);
    f.board.file_review(&f.sysop, &o, true).unwrap();
    let download = f.board.file_download(&f.alice, &o, &key(2)).unwrap();
    let export = f.root.join("backup.sqlite3");
    f.board.export_local(&export).unwrap();
    verify_export(&export).unwrap();
    let dest = f.root.join("restored");
    restore_local(&export, &dest).unwrap();
    let mut b = Board::open(&dest).unwrap();
    let alice = b.authenticate("alice", &f.secret).unwrap();
    assert_eq!(b.file_info(&alice, &o).unwrap().filename, "x.bin");
    assert!(b.file_claim(&download).is_err());
    assert!(matches!(b.file_info(&f.alice, &o), Err(Error::Denied)));
}
#[test]
fn concurrent_reservations_cannot_overspend_one_area() {
    let mut f = F::new();
    let a = f.area(false, 5);
    let path = f.root.join("board");
    let secret = f.secret.clone();
    let work = std::thread::spawn(move || {
        let mut b = Board::open(&path).unwrap();
        let s = b.authenticate("bobby", &secret).unwrap();
        b.file_upload(&s, a, "b.bin", "description", 4, &key(2))
            .is_ok()
    });
    let first = f
        .board
        .file_upload(&f.alice, a, "a.bin", "description", 4, &key(1))
        .is_ok();
    let second = work.join().unwrap();
    assert_ne!(first, second);
}

#[test]
fn expired_grant_denies_lookup_and_claim_but_does_not_free_partial_quota() {
    let mut f = F::new();
    let a = f.area(false, 5);
    let id = f
        .board
        .file_upload(&f.alice, a, "old.bin", "description", 4, &key(1))
        .unwrap();
    let db = rusqlite::Connection::open(f.root.join("board/faction.sqlite3")).unwrap();
    db.execute(
        "UPDATE file_transfers SET created=1,expires=301 WHERE id=?1",
        [&id],
    )
    .unwrap();
    let key = key(1);
    assert!(
        f.board
            .file_lookup_key(key.split_whitespace().nth(1).unwrap())
            .unwrap()
            .is_none()
    );
    assert!(f.board.file_claim(&id).is_err());
    assert!(
        f.board
            .file_upload(&f.bob, a, "new.bin", "description", 2, &key)
            .is_err()
    );
}

#[test]
fn room_and_chat_links_do_not_grant_file_access_or_disclose_private_metadata() {
    use faction_core::ChatKind;
    let mut f = F::new();
    let a = f.area(true, 100);
    f.board.file_allow(&f.sysop, a, "alice", true).unwrap();
    let id = f
        .board
        .file_upload(&f.alice, a, "secret.bin", "Private release", 3, &key(1))
        .unwrap();
    let object = f.complete(&id, 3);
    assert!(f.board.file_reference(&f.alice, &object).is_err());
    f.board.file_review(&f.sysop, &object, true).unwrap();
    let link = f.board.file_reference(&f.alice, &object).unwrap();
    assert!(f.board.file_resolve(&f.bob, &link).is_err());
    f.board
        .post(
            &f.alice,
            1,
            &request_id().unwrap(),
            &format!("Scene release: {link}"),
            None,
        )
        .unwrap();
    let allowed = f.board.posts(&f.alice, 1, 0).unwrap();
    assert!(allowed[0].body.contains("secret.bin"));
    assert!(allowed[0].body.contains(&link));
    let denied = f.board.posts(&f.bob, 1, 0).unwrap();
    assert!(!denied[0].body.contains("secret.bin"));
    assert!(!denied[0].body.contains(&link));
    assert!(denied[0].body.contains("[file unavailable]"));
    assert!(
        f.board
            .post(&f.bob, 1, &request_id().unwrap(), &link, None)
            .is_err()
    );
    let alice = f.board.chat_open(&f.alice).unwrap();
    let bob = f.board.chat_open(&f.bob).unwrap();
    f.board.chat_join(&alice, "#lobby").unwrap();
    f.board.chat_join(&bob, "#lobby").unwrap();
    f.board
        .chat_send(&alice, "#lobby", 1, &link, ChatKind::Say)
        .unwrap();
    assert!(
        f.board
            .chat_send(&bob, "#lobby", 1, &link, ChatKind::Say)
            .is_err()
    );
    assert!(
        f.board.chat_read(&bob, "#lobby", 0).unwrap().messages[0]
            .body
            .contains("[file unavailable]")
    );
    f.board.file_allow(&f.sysop, a, "alice", false).unwrap();
    assert!(
        !f.board.posts(&f.alice, 1, 0).unwrap()[0]
            .body
            .contains("secret.bin")
    );
    assert!(f.board.file_resolve(&f.alice, &link).is_err());
    f.board.file_allow(&f.sysop, a, "bobby", true).unwrap();
    assert!(
        f.board.chat_read(&bob, "#lobby", 0).unwrap().messages[0]
            .body
            .contains("secret.bin")
    );
    f.board.file_review(&f.sysop, &object, false).unwrap();
    assert!(
        !f.board.chat_read(&bob, "#lobby", 0).unwrap().messages[0]
            .body
            .contains("secret.bin")
    );
}

#[test]
fn public_file_namespace_and_references_survive_restore_but_sessions_do_not() {
    let mut f = F::new();
    let a = f.area(false, 100);
    let id = f
        .board
        .file_upload(&f.alice, a, "release.bin", "description", 3, &key(1))
        .unwrap();
    let object = f.complete(&id, 3);
    f.board.file_review(&f.sysop, &object, true).unwrap();
    let link = f.board.file_reference(&f.alice, &object).unwrap();
    f.board
        .post(&f.alice, 1, &request_id().unwrap(), &link, None)
        .unwrap();
    let export = f.root.join("links.sqlite3");
    f.board.export_local(&export).unwrap();
    let dest = f.root.join("restored");
    restore_local(&export, &dest).unwrap();
    let mut b = Board::open(&dest).unwrap();
    let s = b.authenticate("alice", &f.secret).unwrap();
    assert_eq!(b.file_reference(&s, &object).unwrap(), link);
    assert!(b.posts(&s, 1, 0).unwrap()[0].body.contains(&link));
    assert!(b.file_resolve(&f.alice, &link).is_err());
    assert_eq!(b.file_resolve(&s, &link).unwrap().filename, "release.bin");
    let foreign = format!("faction-file:{}/{}", "0".repeat(32), object);
    assert!(b.file_resolve(&s, &foreign).is_err());
    assert!(
        b.post(&s, 1, &request_id().unwrap(), &foreign, None)
            .is_err()
    );
}

#[test]
fn ordinary_text_about_file_link_syntax_remains_readable() {
    let mut f = F::new();
    let body = "A link starts with faction-file: and has board/object IDs.";
    f.board
        .post(&f.alice, 1, &request_id().unwrap(), body, None)
        .unwrap();
    assert_eq!(f.board.posts(&f.alice, 1, 0).unwrap()[0].body, body);
}

#[test]
fn member_can_abandon_own_ready_or_active_grant_without_releasing_reserved_bytes() {
    let mut f = F::new();
    let a = f.area(false, 5);
    let id = f
        .board
        .file_upload(&f.alice, a, "abandon.bin", "description", 4, &key(1))
        .unwrap();
    assert!(f.board.file_abandon(&f.bob, &id).is_err());
    f.board.file_abandon(&f.alice, &id).unwrap();
    assert!(f.board.file_claim(&id).is_err());
    assert!(
        f.board
            .file_upload(&f.alice, a, "other.bin", "description", 2, &key(1))
            .is_err()
    );
    let other = f
        .board
        .file_upload(&f.bob, a, "other.bin", "description", 1, &key(2))
        .unwrap();
    let active = f.board.file_claim(&other).unwrap();
    f.board.file_abandon(&f.bob, &other).unwrap();
    assert!(f.board.file_check(&other, &active.lease).is_err());
}

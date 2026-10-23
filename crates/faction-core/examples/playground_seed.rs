//! Explicitly new, local demo data. Never installed on the member entry point.
#![forbid(unsafe_code)]
use faction_core::{Board, PostDraft, Status, request_id};
use std::{
    fs,
    io::{self, Read},
    os::unix::fs::MetadataExt,
    path::PathBuf,
};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() != 2 || args[0] != "--new-private-state" {
        return Err(
            "usage: playground_seed --new-private-state PARENT/new-board < private-password".into(),
        );
    }
    let path = PathBuf::from(&args[1]);
    let parent = path.parent().ok_or("private parent required")?;
    let info = fs::symlink_metadata(parent)?;
    if !info.is_dir()
        || info.uid() != rustix::process::geteuid().as_raw()
        || info.mode() & 0o077 != 0
        || !matches!(fs::symlink_metadata(&path), Err(e) if e.kind() == io::ErrorKind::NotFound)
    {
        return Err(
            "new state inside an owned private directory required; existing data preserved".into(),
        );
    }
    let mut password = String::new();
    io::stdin().take(130).read_to_string(&mut password)?;
    let password = password.strip_suffix('\n').unwrap_or(&password);
    if !(12..=128).contains(&password.len()) || password.contains(['\n', '\r']) {
        return Err("provide one 12-128 byte demo password on stdin".into());
    }
    let mut board = Board::open(&path)?;
    board.bootstrap("sysop", password)?;
    let sysop = board.authenticate("sysop", password)?;
    for handle in ["cam", "nightowl"] {
        board.register(handle, password)?;
        board.set_status(&sysop, handle, Status::Approved)?;
    }
    let mut wall = board.wall_config(&sysop)?;
    wall.enabled = true;
    wall.name = "The graffiti wall".into();
    board.configure_wall(&sysop, &wall)?;
    let mut pages = board.page_config(&sysop)?;
    pages.enabled = true;
    pages.name = "People & projects".into();
    board.configure_pages(&sysop, &pages)?;
    for (slug, title, subject, body) in [
        (
            "signals",
            "Introductions & signals",
            "Carrier detected",
            "Welcome to Faction BBS. This is your private local playground.\nPick a room with R, use arrows and Enter, or its displayed hotkey.\nPost a reply, leave a wall note, and make this place your own.",
        ),
        (
            "workshop",
            "The BSD workshop",
            "Tiny machines, loud ideas",
            "NetBSD, Rust, small boards and odd hardware belong here.\nAt the message prompt, /edit opens the writing desk.\nSaved room drafts are plaintext board data, visible to the operator.",
        ),
        (
            "artscene",
            "ANSI & the artscene",
            "Afterhours frequency",
            "Original Nightwire artwork welcomes you at login.\nT opens terminal settings. Classic ANSI remains the default.\nTry native text and the bounded graphics sample in Ghostty.",
        ),
        (
            "afterhours",
            "Afterhours hangout",
            "What are you building?",
            "Leave a project update on your member page with M.\nW opens the local graffiti wall. C opens live local chat.\nThese rooms, profiles and wall notes stay on this board.",
        ),
    ] {
        let room = board.create_room(&sysop, slug, title)?;
        board.post_article(
            &sysop,
            room,
            &request_id()?,
            PostDraft {
                subject,
                body,
                reply_to: None,
            },
        )?;
    }
    board.post_article(&sysop, 1, &request_id()?, PostDraft {
        subject: "Start here / Ghostty test flight",
        body: "Use arrows + Enter or hotkeys. R:rooms, W:wall, M:member pages.\nS:sysop controls, T:terminal samples, Q:log out.\nOnly this private preview uses shared demo passwords.\nUse sysop for local BBS settings; cam is an approved ordinary member.",
        reply_to: None,
    })?;
    for (handle, bio, interests, project) in [
        (
            "sysop",
            "Keeper of the late-night carrier.",
            "NetBSD / ANSI / independent communities",
            "Building Faction BBS",
        ),
        (
            "cam",
            "Back on the boards. Make yourself at home.",
            "Old-school BBS culture / tiny systems",
            "Exploring the Ghostty playground",
        ),
        (
            "nightowl",
            "Synthetic demo neighbor; no live person or bot behind this page.",
            "Radio / art / terminal experiments",
            "Making the next great hangout",
        ),
    ] {
        let session = board.authenticate(handle, password)?;
        let mut page = board.member_page(&session, handle)?;
        page.fields = [
            bio.into(),
            interests.into(),
            "--[ carrier / connected ]--".into(),
            project.into(),
        ];
        board.save_member_page(&session, &page)?;
        board.add_member_plan(
            &session,
            &request_id()?,
            "The board is open. Leave a signal and come back after dark.",
            true,
        )?;
        let edition = board.wall_page(&session, None, 0)?.edition;
        board.wall_post(&session, edition, &request_id()?, match handle {
            "sysop" => "Welcome back to the underground.\nYour local notes and room posts survive preview restarts.",
            "cam" => "640K of attitude. Modern terminal energy.",
            _ => "This is a seeded demo note, not an online member.",
        })?;
    }
    board.chat_create(&sysop, "#afterhours", false)?;
    board.chat_create(&sysop, "#bsd", false)?;
    println!(
        "Seeded five rooms, three approved demo accounts/pages/plan updates/wall notes and two local chat channels."
    );
    println!(
        "Demo handles: sysop (BBS sysop), cam and nightowl (ordinary members). Password never printed."
    );
    Ok(())
}

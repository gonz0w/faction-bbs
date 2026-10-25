//! A local notebook, reachable only between sessions or via explicit local CLI.
use crate::{
    Terminal,
    launcher::Catalog,
    passport_data::{self, Data, Entry, Seal},
    passport_store::Store,
    profile,
};
use faction_files::invalid;
use std::{io, path::Path};
fn page(t: &mut Terminal, lines: &[String], number: &mut usize, footer: &str) -> io::Result<usize> {
    let size = rustix::termios::tcgetwinsize(&t.input)?;
    if !(40..=240).contains(&size.ws_col) || !(18..=100).contains(&size.ws_row) {
        t.print("\x1b[2J\x1b[HResize to 40x18.\r\nQ: back\r\n")?;
        return Ok(0);
    }
    let width = usize::from(size.ws_col) - 1;
    let height = usize::from(size.ws_row) - 8;
    let mut rows = Vec::new();
    for line in lines {
        if line.is_empty() {
            rows.push(String::new());
        } else {
            rows.extend(
                line.as_bytes()
                    .chunks(width)
                    .map(|b| String::from_utf8_lossy(b).into_owned()),
            );
        }
    }
    let count = rows.len().div_ceil(height).max(1);
    *number = (*number).min(count - 1);
    t.print(&format!(
        "\x1b[2J\x1b[HLOCAL PASSPORT / on your device\r\nPage {} / {}\r\n\r\n{}\r\n\r\n{}\r\n",
        *number + 1,
        count,
        rows.into_iter()
            .skip(*number * height)
            .take(height)
            .collect::<Vec<_>>()
            .join("\r\n"),
        footer
    ))?;
    Ok(count)
}
fn review(t: &mut Terminal, lines: &[String], word: &str) -> io::Result<bool> {
    let mut at = 0;
    loop {
        let count = page(t, lines, &mut at, "N/P:page Q:cancel")?;
        let prompt = if count > 0 && at + 1 == count {
            format!("Type {word} to confirm: ")
        } else {
            "Review> ".into()
        };
        let key = t.line(&prompt, 8)?;
        match key.as_str() {
            "q" | "Q" => return Ok(false),
            "n" | "N" => at = at.saturating_add(1),
            "p" | "P" => at = at.saturating_sub(1),
            s if s == word && count > 0 && at + 1 == count => return Ok(true),
            _ => (),
        }
    }
}
fn current<'a>(catalog: &'a Catalog, id: &str) -> Option<&'a crate::launcher::Board> {
    catalog.boards.iter().find(|b| b.id == id)
}
fn fresh(catalog: &Catalog, id: &str) -> io::Result<Entry> {
    let b = current(catalog, id).ok_or_else(invalid)?;
    Ok(Entry {
        id: id.into(),
        label: b.label.clone(),
        binding: passport_data::binding(&b.profile)?,
        notes: String::new(),
        favorites: String::new(),
        seal: None,
    })
}
fn status(catalog: &Catalog, e: &Entry) -> &'static str {
    match current(catalog, &e.id) {
        None => "Contact removed from this catalog.",
        Some(b) => match passport_data::binding(&b.profile) {
            Ok(h) if h == e.binding => "Local profile snapshot unchanged.",
            Ok(_) => "Profile/pins changed: review association.",
            Err(_) => "Local profile snapshot unavailable.",
        },
    }
}
fn display(catalog: &Catalog, id: &str, data: &Data) -> Vec<String> {
    let mut out = vec![
        "Private plaintext notes; not mail.".into(),
        "No visit log or membership proof.".into(),
        format!("Local ID: {id}"),
    ];
    if let Some(e) = data.entries.iter().find(|e| e.id == id) {
        out.extend([
            format!("Label: {}", e.label),
            status(catalog, e).into(),
            format!("Note: {}", e.notes),
            format!("Favorite rooms: {}", e.favorites),
        ]);
        if let Some(s) = &e.seal {
            out.extend(s.lines());
        } else {
            out.push("No seal collected.".into());
        }
    } else {
        out.push("Nothing saved for this board.".into());
        out.push("E: create note  I: import local seal".into());
    }
    out
}
fn persist(store: &Store, data: &mut Data, entry: Entry) -> io::Result<()> {
    let mut next = data.clone();
    next.put(entry)?;
    store.write(&next, false)?;
    *data = next;
    Ok(())
}
pub fn run(t: &mut Terminal, path: &Path, catalog: &Catalog, last: usize) -> io::Result<()> {
    let store = Store::at(path, false)?;
    let mut data = store.read()?;
    let mut id = catalog.boards[last].id.clone();
    let mut at = 0;
    let mut notice = String::new();
    rustix::termios::tcflush(&t.input, rustix::termios::QueueSelector::IFlush)?;
    loop {
        let mut lines = display(catalog, &id, &data);
        if !notice.is_empty() {
            lines.insert(0, notice.clone());
        }
        page(t, &lines, &mut at, "E:note I:seal S:share L:list D:delete")?;
        t.print("N/P:page B:rebind Q:back\r\n")?;
        let input = t.line("Passport> ", 1)?.to_ascii_lowercase();
        notice.clear();
        let result = (|| -> io::Result<()> {
            match input.as_str() {
                "q" => return Ok(()),
                "n" => at = at.saturating_add(1),
                "p" => at = at.saturating_sub(1),
                "e" => {
                    let mut e = match data.entries.iter().find(|e| e.id == id) {
                        Some(e) => e.clone(),
                        None => fresh(catalog, &id)?,
                    };
                    let notes = t.line("Note (empty keeps, - clears): ", 512)?;
                    let favorites = t.line("Rooms (empty keeps, - clears): ", 96)?;
                    if !notes.is_empty() {
                        e.notes = if notes == "-" { String::new() } else { notes };
                    }
                    if !favorites.is_empty() {
                        e.favorites = if favorites == "-" {
                            String::new()
                        } else {
                            favorites
                        };
                    }
                    e.check()?;
                    let review_lines = vec![
                        "PRIVATE PLAINTEXT / local only".into(),
                        format!("Note: {}", e.notes),
                        format!("Favorite rooms: {}", e.favorites),
                        "No timestamp or visit saved.".into(),
                    ];
                    if review(t, &review_lines, "SAVE")? {
                        persist(&store, &mut data, e)?;
                        notice = "Notebook saved locally.".into();
                    }
                    at = 0;
                }
                "i" => {
                    let snapshot = fresh(catalog, &id)?;
                    let mut e = data
                        .entries
                        .iter()
                        .find(|e| e.id == id)
                        .cloned()
                        .unwrap_or(snapshot.clone());
                    if e.binding != snapshot.binding {
                        notice = "Profile changed; review then B:rebind.".into();
                        return Ok(());
                    }
                    let local = t.line("Local seal file (no remote import): ", 512)?;
                    let seal = Seal::load(&profile::path(local)?)?;
                    if e.seal.as_ref() == Some(&seal) {
                        notice = "Seal already collected; unchanged.".into();
                        return Ok(());
                    }
                    let word = if e.seal.is_some() {
                        "REPLACE"
                    } else {
                        "COLLECT"
                    };
                    let mut lines = seal.lines();
                    lines.extend([
                        "".into(),
                        "Generic souvenir; no visit proof.".into(),
                        "No identity/admission authority.".into(),
                        "No notice sent to any board.".into(),
                    ]);
                    if review(t, &lines, word)? {
                        e.seal = Some(seal);
                        persist(&store, &mut data, e)?;
                        notice = "Souvenir collected locally.".into();
                    }
                    at = 0;
                }
                "b" => {
                    let mut e = data
                        .entries
                        .iter()
                        .find(|e| e.id == id)
                        .cloned()
                        .ok_or_else(invalid)?;
                    let snapshot = fresh(catalog, &id)?;
                    if review(
                        t,
                        &[
                            "Reassociate this local notebook?".into(),
                            "Verify profile/keys out of band.".into(),
                            "Notes/seal remain unverified.".into(),
                            "This grants no remote authority.".into(),
                        ],
                        "REBIND",
                    )? {
                        e.binding = snapshot.binding;
                        e.label = snapshot.label;
                        persist(&store, &mut data, e)?;
                        notice = "Local association updated.".into();
                    }
                    at = 0;
                }
                "s" => {
                    let e = data
                        .entries
                        .iter()
                        .find(|e| e.id == id)
                        .ok_or_else(invalid)?;
                    let (mut art, mut notes, mut favorites) = (true, false, false);
                    let mut p = 0;
                    loop {
                        let mut lines = vec![
                            format!("Share: seal={art} note={notes} rooms={favorites}"),
                            "Preview only; copy if you choose.".into(),
                        ];
                        lines.extend(e.share(art, notes, favorites));
                        page(t, &lines, &mut p, "A:seal E:note F:rooms N/P:page Q:back")?;
                        match t.line("Share> ", 1)?.to_ascii_lowercase().as_str() {
                            "a" => {
                                art = !art;
                                p = 0
                            }
                            "e" => {
                                notes = !notes;
                                p = 0
                            }
                            "f" => {
                                favorites = !favorites;
                                p = 0
                            }
                            "n" => p = p.saturating_add(1),
                            "p" => p = p.saturating_sub(1),
                            "q" => break,
                            _ => (),
                        }
                    }
                    at = 0;
                }
                "l" => {
                    let lines = if data.entries.is_empty() {
                        vec!["Notebook is empty.".into()]
                    } else {
                        data.entries
                            .iter()
                            .map(|e| format!("{} // {}", e.id, e.label))
                            .collect()
                    };
                    let mut p = 0;
                    loop {
                        page(t, &lines, &mut p, "N/P:page C:choose Q:back")?;
                        match t.line("List> ", 1)?.to_ascii_lowercase().as_str() {
                            "n" => p = p.saturating_add(1),
                            "p" => p = p.saturating_sub(1),
                            "q" => break,
                            "c" => {
                                let chosen = t.line("Saved local ID: ", 32)?;
                                if data.entries.iter().any(|e| e.id == chosen) {
                                    id = chosen;
                                }
                                break;
                            }
                            _ => (),
                        }
                    }
                    at = 0;
                }
                "d" => {
                    if data.entries.iter().any(|e| e.id == id)
                        && review(
                            t,
                            &[
                                "Delete this notebook entry?".into(),
                                "Removes local note, rooms and seal.".into(),
                                "Old backups may retain copies.".into(),
                            ],
                            "DELETE",
                        )?
                    {
                        let mut next = data.clone();
                        next.entries.retain(|e| e.id != id);
                        store.write(&next, false)?;
                        data = next;
                        notice = "Entry deleted locally.".into();
                    }
                    at = 0;
                }
                _ => (),
            }
            Ok(())
        })();
        if input == "q" {
            return Ok(());
        }
        if let Err(e) = result {
            if e.kind() == io::ErrorKind::Interrupted {
                return Err(e);
            }
            notice = "Operation stopped; check saved state.".into();
        }
    }
}
pub fn cli(args: &[String]) -> io::Result<()> {
    if args.len() < 3 {
        return Err(invalid());
    }
    let path = profile::path(args[1].clone())?;
    match args[2].as_str() {
        "init" if args.len() == 3 => {
            let store = Store::at(&path, true)?;
            store.write(&Data::default(), true)?;
            println!(
                "Local passport initialized. Notes are plaintext on your device; no travel log."
            );
        }
        "list" if args.len() == 3 => {
            let data = Store::at(&path, false)?.read()?;
            println!("Local passport / no visit or membership proof");
            for e in data.entries {
                println!("{} // {}", e.id, e.label);
            }
        }
        "export" if args.len() == 4 => {
            let data = Store::at(&path, false)?.read()?;
            let e = data
                .entries
                .iter()
                .find(|e| e.id == args[3])
                .ok_or_else(invalid)?;
            println!("{}", e.share(true, false, false).join("\n"));
        }
        _ => return Err(invalid()),
    }
    Ok(())
}

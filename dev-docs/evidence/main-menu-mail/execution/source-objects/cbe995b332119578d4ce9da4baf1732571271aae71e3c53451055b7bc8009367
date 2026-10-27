//! Filesystem-owner commands only: never exported by the shared terminal library.
use faction_cli::{Result, invalid};
use faction_core::{
    Board, RoamingAdmission, RoamingBinding, RoamingIssuer, RoamingOwnerState, RoamingRegistry,
};
use rustix::fs::{self, AtFlags, Mode, OFlags};
use std::{
    ffi::OsString,
    fmt::Write as _,
    fs::File,
    io::Write,
    os::unix::fs::MetadataExt,
    path::{Component, Path},
};

pub const HELP: &str = "Experimental filesystem-owner roaming (existing schema-20 state only):\n  faction --state-dir PATH roaming state\n  faction --state-dir PATH roaming member-id LOCAL_HANDLE\n  faction --state-dir PATH roaming invite NAME CA_FP\n  faction --state-dir PATH roaming issuer-enable|issuer-disable ID GEN\n  faction --state-dir PATH roaming link ISSUER_ID ISSUER_GEN SUBJECT PRINCIPAL MEMBER_ID UID CONSENT\n  faction --state-dir PATH roaming binding-enable|binding-disable ID GEN\n  faction --state-dir PATH roaming manifest-preview\n  faction --state-dir PATH roaming manifest-export NEW_FILE\nExport is inert; root separately reviews transport accounts, CA/principal policy and reserved UIDs. No session or host configuration is installed.";

// Walk directory descriptors rather than following links or racing pathname checks.
// Root-owned sticky temporary ancestors are allowed; the final directory is private.
fn directory(path: &Path) -> Result<File> {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()?.join(path)
    };
    let mut dir = File::from(fs::open(
        "/",
        OFlags::RDONLY | OFlags::DIRECTORY | OFlags::CLOEXEC,
        Mode::empty(),
    )?);
    for component in absolute.components() {
        match component {
            Component::RootDir | Component::CurDir => continue,
            Component::Normal(name) => {
                dir = File::from(fs::openat(
                    &dir,
                    name,
                    OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
                    Mode::empty(),
                )?);
                let m = dir.metadata()?;
                let sticky_root = m.uid() == 0 && m.mode() & 0o1000 != 0;
                if (m.uid() != 0 && m.uid() != rustix::process::geteuid().as_raw())
                    || (m.mode() & 0o022 != 0 && !sticky_root)
                {
                    return Err(invalid("unsafe roaming path ancestor"));
                }
            }
            _ => return Err(invalid("roaming paths cannot contain parent traversal")),
        }
    }
    let m = dir.metadata()?;
    if m.uid() != rustix::process::geteuid().as_raw() || m.mode() & 0o077 != 0 {
        return Err(invalid("roaming requires a private owned directory"));
    }
    Ok(dir)
}
fn open_existing(path: &Path) -> Result<Board> {
    let dir = directory(path)?;
    let file = File::from(fs::openat(
        &dir,
        "faction.sqlite3",
        OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::NONBLOCK | OFlags::CLOEXEC,
        Mode::empty(),
    )?);
    let m = file.metadata()?;
    if !m.is_file()
        || m.uid() != rustix::process::geteuid().as_raw()
        || m.mode() & 0o077 != 0
        || m.nlink() != 1
    {
        return Err(invalid("private initialized roaming state required"));
    }
    Ok(Board::open_existing(path)?)
}
fn number(value: &OsString) -> Result<i64> {
    let text = value
        .to_str()
        .ok_or_else(|| invalid("positive numeric ID/generation required"))?;
    if !text.bytes().all(|b| b.is_ascii_digit()) {
        return Err(invalid("positive numeric ID/generation required"));
    }
    text.parse::<i64>()
        .ok()
        .filter(|n| *n > 0)
        .ok_or_else(|| invalid("positive numeric ID/generation required"))
}
fn text(value: &OsString) -> Result<&str> {
    value
        .to_str()
        .ok_or_else(|| invalid("roaming metadata needs ASCII"))
}
// Values exclude whitespace, so one ASCII space separates fields and the first
// '=' separates each field name from its literal value. Quotes and backslashes
// remain inert printable bytes rather than doubling the bounded output.
fn metadata(value: &str, limit: usize) -> Result<&str> {
    if value.is_empty() || value.len() > limit || !value.bytes().all(|b| b.is_ascii_graphic()) {
        return Err(invalid("invalid roaming owner display metadata"));
    }
    Ok(value)
}
fn issuer(out: &mut String, i: &RoamingIssuer) -> Result<()> {
    writeln!(
        out,
        "issuer id={} home={} ca_fingerprint={} generation={} enabled={}",
        i.id,
        metadata(&i.name, 96)?,
        metadata(&i.ca_fingerprint, 50)?,
        i.generation,
        i.enabled
    )
    .unwrap();
    Ok(())
}
fn binding(out: &mut String, b: &RoamingBinding) -> Result<()> {
    writeln!(out, "binding id={} issuer_id={} subject={} principal={} member_id={} uid={} consent={} generation={} enabled={}", b.id, b.issuer_id, metadata(&b.subject, 128)?, metadata(&b.principal, 192)?, b.member_id, b.uid, metadata(&b.consent, 160)?, b.generation, b.enabled).unwrap();
    Ok(())
}
fn identity(bytes: &[u8; 32]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
fn manifest(board: &mut Board, state: &RoamingOwnerState) -> Result<String> {
    let entries = state
        .bindings
        .iter()
        .filter(|b| b.enabled)
        .map(|b| {
            let i = state
                .issuers
                .iter()
                .find(|i| i.id == b.issuer_id && i.enabled)
                .ok_or_else(|| invalid("enabled binding has no enabled issuer"))?;
            Ok(RoamingAdmission {
                uid: b.uid,
                binding_id: b.id,
                binding_generation: b.generation,
                issuer_generation: i.generation,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    let registry = RoamingRegistry {
        board_identity: state.board_identity,
        entries,
    };
    // This fresh Board has never authenticated. Validate current approval and exact
    // generations through core without invoking its session factory.
    board.configure_roaming_owner(Some(registry.clone()))?;
    let mut out = format!(
        "version = 1\nboard_identity = \"{}\"\n",
        identity(&state.board_identity)
    );
    if registry.entries.is_empty() {
        out.push_str("entries = []\n");
    }
    for a in registry.entries {
        writeln!(out, "\n[[entries]]\nuid = {}\nbinding_id = {}\nbinding_generation = {}\nissuer_generation = {}", a.uid, a.binding_id, a.binding_generation, a.issuer_generation).unwrap();
    }
    if out.len() > 8192 {
        return Err(invalid("roaming manifest exceeds 8192 bytes"));
    }
    Ok(out)
}
fn export(path: &Path, bytes: &[u8]) -> Result<()> {
    let name = path
        .file_name()
        .ok_or_else(|| invalid("new manifest file required"))?;
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let dir = directory(parent)?;
    let mut file = File::from(fs::openat(
        &dir,
        name,
        OFlags::WRONLY | OFlags::CREATE | OFlags::EXCL | OFlags::NOFOLLOW | OFlags::CLOEXEC,
        Mode::from_raw_mode(0o600),
    )?);
    if let Err(e) = file.write_all(bytes).and_then(|_| file.sync_all()) {
        let _ = fs::unlinkat(&dir, name, AtFlags::empty());
        return Err(e.into());
    }
    Ok(())
}
pub fn run(path: &Path, args: &[OsString]) -> Result<()> {
    let Some(command) = args.first().and_then(|a| a.to_str()) else {
        return Err(invalid(HELP));
    };
    if command == "--help" && args.len() == 1 {
        println!("{HELP}");
        return Ok(());
    }
    let expected = match command {
        "state" | "manifest-preview" => 1,
        "manifest-export" | "member-id" => 2,
        "invite" | "issuer-enable" | "issuer-disable" | "binding-enable" | "binding-disable" => 3,
        "link" => 8,
        _ => return Err(invalid(HELP)),
    };
    if args.len() != expected {
        return Err(invalid(HELP));
    }
    let mut board = open_existing(path)?;
    let mut out = String::new();
    match command {
        "state" => {
            let state = board.roaming_owner_state()?;
            writeln!(
                out,
                "Experimental roaming owner state\nboard_identity={}",
                identity(&state.board_identity)
            )
            .unwrap();
            for i in &state.issuers {
                issuer(&mut out, i)?;
            }
            for b in &state.bindings {
                binding(&mut out, b)?;
            }
        }
        "member-id" => {
            writeln!(
                out,
                "member_id={}",
                board.roaming_owner_member_id(text(&args[1])?)?
            )
            .unwrap();
        }
        "invite" => issuer(
            &mut out,
            &board.roaming_add_issuer(text(&args[1])?, text(&args[2])?)?,
        )?,
        "issuer-enable" | "issuer-disable" => issuer(
            &mut out,
            &board.roaming_set_issuer_enabled(
                number(&args[1])?,
                number(&args[2])?,
                command == "issuer-enable",
            )?,
        )?,
        "binding-enable" | "binding-disable" => binding(
            &mut out,
            &board.roaming_set_binding_enabled(
                number(&args[1])?,
                number(&args[2])?,
                command == "binding-enable",
            )?,
        )?,
        "link" => {
            let uid = u32::try_from(number(&args[6])?)
                .map_err(|_| invalid("positive u32 UID required"))?;
            binding(
                &mut out,
                &board.roaming_link_member(
                    number(&args[1])?,
                    number(&args[2])?,
                    text(&args[3])?,
                    text(&args[4])?,
                    number(&args[5])?,
                    uid,
                    text(&args[7])?,
                )?,
            )?;
        }
        "manifest-preview" | "manifest-export" => {
            let state = board.roaming_owner_state()?;
            let bytes = manifest(&mut board, &state)?;
            if command == "manifest-preview" {
                out = bytes;
            } else {
                export(Path::new(&args[1]), bytes.as_bytes())?;
                out.push_str("Wrote inert private manifest for separate root review/install; no transport policy validated or installed.\n");
            }
        }
        _ => unreachable!(),
    }
    if out.len() > 32768 {
        return Err(invalid("roaming owner output exceeds bound"));
    }
    print!("{out}");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn literal_metadata_preserves_field_values_and_rejects_non_graphical_stored_text() {
        let literal = "\\\"=subject=other";
        assert_eq!(metadata(literal, 128).unwrap(), literal);
        for invalid in [
            "",
            "space value",
            "line\nvalue",
            "\x1b[2J",
            "\t",
            "\x7f",
            "é",
        ] {
            assert!(metadata(invalid, 128).is_err());
            let row = RoamingIssuer {
                id: 1,
                name: invalid.into(),
                ca_fingerprint: "SHA256:AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA".into(),
                generation: 1,
                enabled: false,
            };
            let mut out = String::new();
            assert!(issuer(&mut out, &row).is_err());
            assert!(out.is_empty());
        }
        assert!(metadata(&"a".repeat(129), 128).is_err());
    }
}

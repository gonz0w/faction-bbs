#![forbid(unsafe_code)]
mod transfer;
use faction_files::invalid;
use std::{io, path::Path};
fn run() -> io::Result<()> {
    // This non-setuid helper never needs root or a temporary privileged identity.
    // In particular, SSH key lookup must have completed its permanent UID drop.
    let uid = rustix::process::getuid();
    let gid = rustix::process::getgid();
    if uid.as_raw() == 0
        || gid.as_raw() == 0
        || uid != rustix::process::geteuid()
        || gid != rustix::process::getegid()
    {
        return Err(invalid());
    }
    let a = std::env::args().skip(1).collect::<Vec<_>>();
    match a.iter().map(String::as_str).collect::<Vec<_>>().as_slice() {
        ["key", "ssh-ed25519", key] => {
            let c = transfer::Config::load(Path::new("/etc/faction/files.toml"))?;
            if key.len() > 128 {
                return Err(invalid());
            }
            let v = c.call(&["lookup", key])?;
            if v.len() != 1 || !faction_files::id(&v[0]) {
                return Err(invalid());
            }
            println!(
                "restrict,command=\"/usr/local/libexec/faction-files sftp {}\" ssh-ed25519 {}",
                v[0], key
            );
            Ok(())
        }
        ["sftp", id] => {
            if std::env::var("SSH_ORIGINAL_COMMAND").ok().as_deref()
                != Some("/usr/local/libexec/faction-files subsystem")
            {
                return Err(invalid());
            }
            transfer::sftp(
                transfer::Config::load(Path::new("/etc/faction/files.toml"))?,
                (*id).into(),
            )
        }
        ["stdio", config, id] => {
            transfer::sftp(transfer::Config::load(Path::new(config))?, (*id).into())
        }
        ["gc"] => {
            let c = transfer::Config::load(Path::new("/etc/faction/files.toml"))?;
            println!("Reclaimed {} removed file objects.", transfer::gc(&c)?);
            Ok(())
        }
        _ => Err(invalid()),
    }
}
fn main() {
    if let Err(error) = run() {
        eprintln!("Faction file transfer denied or failed: {error}");
        std::process::exit(1)
    }
}

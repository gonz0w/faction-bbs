//! OpenSSH ForceCommand frontend. Runs as a separate database-inaccessible UID.
#![forbid(unsafe_code)]
use faction_access::{Result, check_directory, denied, send_terminal};
use std::{
    fs::{self, File},
    io::{self, IsTerminal, Read},
    os::unix::{
        fs::{FileTypeExt, MetadataExt},
        net::UnixStream,
    },
    path::PathBuf,
    time::Duration,
};
struct Restore(File, rustix::termios::Termios);
impl Drop for Restore {
    fn drop(&mut self) {
        let _ =
            rustix::termios::tcsetattr(&self.0, rustix::termios::OptionalActions::Flush, &self.1);
    }
}
fn run() -> Result<()> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() != 4 || args[0] != "--socket" || args[2] != "--owner-uid" {
        return Err(denied());
    }
    // Presence rejects even an empty exec request. Shell-channel requests alone
    // enter the BBS; nothing from the client is interpreted as a host command.
    if std::env::var_os("SSH_ORIGINAL_COMMAND").is_some()
        || std::env::var_os("SSH_CONNECTION").is_none()
        || !io::stdin().is_terminal()
        || !io::stdout().is_terminal()
    {
        return Err(denied());
    }
    let owner: u32 = args[3].to_str().ok_or_else(denied)?.parse()?;
    if owner == 0 || owner == rustix::process::geteuid().as_raw() {
        return Err(denied());
    }
    let path = PathBuf::from(&args[1]);
    let parent = path.parent().ok_or_else(denied)?;
    check_directory(parent, owner)?;
    let m = fs::symlink_metadata(&path)?;
    if !m.file_type().is_socket() || m.uid() != owner || m.mode() & 0o007 != 0 {
        return Err(denied());
    }
    let tty = File::from(rustix::io::fcntl_dupfd_cloexec(io::stdin(), 3)?);
    let saved = rustix::termios::tcgetattr(&tty)?;
    let _restore = Restore(tty.try_clone()?, saved);
    let mut stream = UnixStream::connect(path)?;
    stream.set_read_timeout(Some(Duration::from_secs(1900)))?;
    stream.set_write_timeout(Some(Duration::from_secs(2)))?;
    send_terminal(
        &stream,
        &tty,
        &std::env::var("TERM").unwrap_or_else(|_| "dumb".into()),
    )?;
    let mut status = [0];
    stream.read_exact(&mut status)?;
    if status[0] != 0 {
        return Err(denied());
    }
    Ok(())
}
fn main() {
    if run().is_err() {
        eprintln!("Faction SSH: terminal admission denied or session ended.");
        std::process::exit(1)
    }
}

//! Explicit opt-in, synthetic-policy attachment provider embedding.
//! TLS credentials authorize approved routing; they do not attest OMEMO key possession.
//! Runtime ownership is authoritative: controls and sessions cannot retain it after drop.
pub mod authority;
mod service;
pub mod tls;
use authority::{Authority, Registry};
use faction_files::private_blobs;
pub use service::BinaryService;
use std::{
    net::{Shutdown, TcpStream},
    sync::{
        Arc, Mutex, Weak,
        atomic::{AtomicBool, Ordering},
    },
};
pub use tls::Session;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    Transport(tls::Stage),
    Store(private_blobs::Error),
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("private attachment provider unavailable")
    }
}
impl std::error::Error for Error {}
pub(crate) struct State {
    authority: Mutex<Authority>,
    stopped: Arc<AtomicBool>,
    active: AtomicBool,
    socket: Mutex<Option<TcpStream>>,
}
impl State {
    fn stop(&self) {
        self.stopped.store(true, Ordering::SeqCst);
        if let Ok(mut a) = self.authority.lock() {
            a.unavailable();
        }
        if let Ok(mut socket) = self.socket.lock()
            && let Some(s) = socket.take()
        {
            let _ = s.shutdown(Shutdown::Both);
        }
    }
}
/// Sole non-cloneable owner. Drop/stop terminally invalidate all its sessions.
pub struct Runtime {
    pub(crate) state: Arc<State>,
}
impl Runtime {
    pub fn new(registry: Option<Registry>) -> Self {
        Self {
            state: Arc::new(State {
                authority: Mutex::new(Authority::new(registry)),
                stopped: Arc::new(AtomicBool::new(false)),
                active: AtomicBool::new(false),
                socket: Mutex::new(None),
            }),
        }
    }
    pub fn control(&self) -> Control {
        Control {
            state: Arc::downgrade(&self.state),
        }
    }
    pub fn stop(&self) {
        self.state.stop();
    }
}
impl Drop for Runtime {
    fn drop(&mut self) {
        self.stop();
    }
}
/// Weak policy control; it cannot keep a dropped runtime alive or revive a stopped one.
#[derive(Clone)]
pub struct Control {
    state: Weak<State>,
}
impl Control {
    fn state(&self) -> Result<Arc<State>, Error> {
        let s = self
            .state
            .upgrade()
            .ok_or(Error::Transport(tls::Stage::Denied))?;
        if s.stopped.load(Ordering::SeqCst) {
            return Err(Error::Transport(tls::Stage::Denied));
        }
        Ok(s)
    }
    pub fn replace_registry(&self, registry: Registry) -> Result<(), Error> {
        let state = self.state()?;
        let mut a = state
            .authority
            .lock()
            .map_err(|_| Error::Transport(tls::Stage::Denied))?;
        if state.stopped.load(Ordering::SeqCst) {
            return Err(Error::Transport(tls::Stage::Denied));
        }
        a.replace_registry(registry).map_err(Error::Store)
    }
    pub fn unavailable(&self) -> Result<(), Error> {
        self.state()?
            .authority
            .lock()
            .map_err(|_| Error::Transport(tls::Stage::Denied))?
            .unavailable();
        Ok(())
    }
    pub fn reset(&self, reset_epoch: u64) -> Result<(), Error> {
        self.state()?
            .authority
            .lock()
            .map_err(|_| Error::Transport(tls::Stage::Denied))?
            .reset(reset_epoch)
            .map_err(Error::Store)
    }
    pub fn stop(&self) -> Result<(), Error> {
        self.state()?.stop();
        Ok(())
    }
}
#[cfg(test)]
mod authority_tests;

#[cfg(feature = "blob-iq")]
mod iq;
#[cfg(feature = "blob-iq")]
pub use iq::IqService;

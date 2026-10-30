//! Explicit opt-in, synthetic-policy attachment provider embedding.
//! TLS credentials authorize approved routing; they do not attest OMEMO key possession.
//! Runtime ownership is authoritative: controls and sessions cannot retain it after drop.
pub mod authority;
#[cfg(feature = "delegation-records")]
pub use authority::retrieval;
#[cfg(feature = "delegation-services")]
pub use authority::services;
#[cfg(feature = "delegation-issuer")]
pub use retrieval::issuer;
#[cfg(feature = "delegation-services")]
pub use tls::ServiceSession;
#[cfg(feature = "durable-policy")]
pub mod durable;
#[cfg(feature = "preparation")]
pub mod preparation;
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
    #[cfg(feature = "durable-policy")]
    disk: Option<durable::Disk>,
}
impl State {
    /// Validation, publication and installation share the dispatch authority lock.
    fn update(
        &self,
        change: impl FnOnce(&mut Authority) -> Result<(), private_blobs::Error>,
    ) -> Result<(), Error> {
        let mut authority = self
            .authority
            .lock()
            .map_err(|_| Error::Transport(tls::Stage::Denied))?;
        if self.stopped.load(Ordering::SeqCst) {
            return Err(Error::Transport(tls::Stage::Denied));
        }
        #[cfg(feature = "durable-policy")]
        if let Some(disk) = &self.disk {
            let mut candidate = Authority::restore(authority.snapshot()).map_err(Error::Store)?;
            change(&mut candidate).map_err(Error::Store)?;
            if let Err(error) = disk.write(&candidate.snapshot()) {
                authority.unavailable();
                self.stopped.store(true, Ordering::SeqCst);
                drop(authority);
                self.close_socket();
                return Err(Error::Store(error));
            }
            authority.install(candidate.snapshot());
            return Ok(());
        }
        change(&mut authority).map_err(Error::Store)
    }
    fn close_socket(&self) {
        if let Ok(mut socket) = self.socket.lock()
            && let Some(s) = socket.take()
        {
            let _ = s.shutdown(Shutdown::Both);
        }
    }
    fn stop(&self) {
        self.stopped.store(true, Ordering::SeqCst);
        if let Ok(mut a) = self.authority.lock() {
            a.unavailable();
        }
        self.close_socket();
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
                #[cfg(feature = "durable-policy")]
                disk: None,
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
    /// Durable runtimes acknowledge only after publication. Invalid proposals
    /// leave live policy unchanged; any storage uncertainty requires reopen.
    pub fn replace_registry(&self, registry: Registry) -> Result<(), Error> {
        self.state()?
            .update(|authority| authority.replace_registry(registry))
    }
    /// Explicitly persist unavailable policy when durability was selected.
    /// Ordinary Runtime stop/drop never persist this policy action.
    pub fn unavailable(&self) -> Result<(), Error> {
        self.state()?.update(|authority| {
            authority.unavailable();
            Ok(())
        })
    }
    pub fn reset(&self, reset_epoch: u64) -> Result<(), Error> {
        self.state()?
            .update(|authority| authority.reset(reset_epoch))
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

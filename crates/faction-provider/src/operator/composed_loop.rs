//! The maintained composed worker/pending/admission loop; no alternate router.
use super::ProviderError;
use super::{
    Delivery, DeliverySnapshot, Event, PublicationWireProfile, RelayRoutes, Runtime, Server, Stage,
    Store, mapped,
};
use std::{
    io::{self, Write},
    net::TcpListener,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread,
};

pub(super) struct Inputs<'env> {
    pub(super) server: &'env Server,
    pub(super) listener: TcpListener,
    pub(super) runtime: &'env Runtime,
    pub(super) cancel: &'env Arc<AtomicBool>,
    pub(super) routes: &'env RelayRoutes,
    pub(super) store: &'env Mutex<Store>,
    pub(super) home: &'env str,
    pub(super) profile: &'env PublicationWireProfile,
    pub(super) port: u16,
}
#[derive(Clone, Copy)]
pub(super) enum RejectionPhase {
    Serve = 1,
    Cleanup = 2,
}
pub(super) fn rejection_line(
    phase: RejectionPhase,
    error: Option<ProviderError>,
    pending: Option<DeliverySnapshot>,
) -> String {
    let category = match error {
        Some(ProviderError::Transport(Stage::Limit)) => 1,
        Some(ProviderError::Transport(Stage::Deadline)) => 2,
        Some(ProviderError::Transport(Stage::Cancelled)) => 3,
        Some(ProviderError::Transport(Stage::Denied)) => 4,
        Some(ProviderError::Transport(Stage::Frame)) => 5,
        Some(ProviderError::Store(faction_files::private_blobs::Error::Busy)) => 6,
        Some(ProviderError::Store(_)) => 7,
        Some(ProviderError::Transport(_)) => 8,
        None => 9,
    };
    let present = u8::from(pending.is_some());
    let state = pending.unwrap_or_default();
    format!(
        "carrier_rejection schema=1 phase={} category={category} pending={present} shared_attempts={} finished={} consumed={}",
        phase as u8,
        state.attempted,
        u8::from(state.finished),
        u8::from(state.consumed)
    )
}
pub(super) fn diagnostic_budget(value: Option<&std::ffi::OsStr>) -> u8 {
    if value == Some(std::ffi::OsStr::new("1")) {
        8
    } else {
        0
    }
}
pub(super) fn emit_rejection(
    output: &mut impl Write,
    remaining: &mut u8,
    phase: RejectionPhase,
    error: Option<ProviderError>,
    pending: Option<DeliverySnapshot>,
) {
    if *remaining == 0 {
        return;
    }
    *remaining -= 1;
    let _ = writeln!(output, "{}", rejection_line(phase, error, pending));
}
fn reap_finished<'scope>(
    worker: &mut Option<thread::ScopedJoinHandle<'scope, Result<(), ProviderError>>>,
    pending: &mut Option<Delivery>,
    diagnostics_remaining: &mut u8,
) -> Result<(), ()> {
    if worker.as_ref().is_some_and(|w| w.is_finished()) {
        let _ = mapped(worker.take().ok_or(())?.join())?;
    }
    if worker.is_none()
        && pending
            .as_ref()
            .is_some_and(|d| d.is_consumed() || d.is_expired())
    {
        if let Some(delivery) = pending.as_ref().filter(|d| !d.is_consumed()) {
            emit_rejection(
                &mut io::stderr(),
                diagnostics_remaining,
                RejectionPhase::Cleanup,
                None,
                Some(delivery.diagnostic_snapshot()),
            );
        }
        pending.take();
    }
    Ok(())
}

pub(super) fn run<'scope, 'env>(
    scope: &'scope thread::Scope<'scope, 'env>,
    inputs: Inputs<'env>,
    #[cfg(test)] hooks: Option<&'env Hooks>,
) -> Result<Result<(), ()>, ()> {
    let Inputs {
        server,
        listener,
        runtime,
        cancel,
        routes,
        store,
        home,
        profile,
        port,
    } = inputs;
    let mut worker: Option<thread::ScopedJoinHandle<'_, std::result::Result<(), ProviderError>>> =
        None;
    let mut pending: Option<Delivery> = None;
    let mut diagnostics_remaining =
        diagnostic_budget(std::env::var_os("FACTION_PROVIDER_DIAGNOSTICS").as_deref());
    let result = (|| {
        mapped(writeln!(io::stdout(), "ready {port}"))?;
        mapped(io::stdout().flush())?;
        while !cancel.load(Ordering::SeqCst) {
            reap_finished(&mut worker, &mut pending, &mut diagnostics_remaining)?;
            #[cfg(test)]
            if let Some(hooks) = hooks {
                hooks.after_cleanup(&worker, pending.as_ref());
            }
            if let Ok(incoming) = server.accept_composed(&listener, runtime, cancel.clone()) {
                // Admission may outlast a real worker. Join it before releasing
                // its consumed/expired reservation and dispatching this caller.
                reap_finished(&mut worker, &mut pending, &mut diagnostics_remaining)?;
                match incoming.serve(pending.as_mut(), store, home, profile) {
                    Ok(Event::Submitted(delivery, work)) => {
                        // Incoming already dropped the actual submitted TLS
                        // carrier. Exactly one continuation may now run.
                        if pending.is_some() || worker.is_some() {
                            return Err(());
                        }
                        #[cfg(test)]
                        let delivery = {
                            let mut delivery = delivery;
                            if let Some(hooks) = hooks {
                                hooks.submitted(&mut delivery);
                            }
                            delivery
                        };
                        pending = Some(delivery);
                        worker = Some(scope.spawn(move || {
                            let result = work.execute(routes, store);
                            #[cfg(test)]
                            if let Some(hooks) = hooks {
                                hooks.before_worker_exit();
                            }
                            result
                        }));
                    }
                    Ok(Event::Collected(_)) | Ok(Event::Served) => (),
                    Err(error) => emit_rejection(
                        &mut io::stderr(),
                        &mut diagnostics_remaining,
                        RejectionPhase::Serve,
                        Some(error),
                        pending.as_ref().map(Delivery::diagnostic_snapshot),
                    ),
                }
            }
        }
        Ok(())
    })();
    cancel.store(true, Ordering::SeqCst);
    runtime.stop();
    drop(listener);
    if let Some(worker) = worker {
        // Cancellation is an ordinary worker outcome during bounded drain.
        let _ = mapped(worker.join())?;
    }
    drop(pending);
    Ok(result)
}

// These one-shot lifecycle gates exist only in a test compilation. They neither
// replace a real handle nor change pending state, authority, framing or charges.
#[cfg(test)]
pub(super) type AttachmentHook = Box<dyn Fn(&mut Delivery) + Send + Sync>;
#[cfg(test)]
pub(super) struct Hooks {
    pub(super) lifecycle: Option<LifecycleHooks>,
    pub(super) attach: Option<AttachmentHook>,
}
#[cfg(test)]
impl Hooks {
    fn submitted(&self, delivery: &mut Delivery) {
        if let Some(attach) = &self.attach {
            attach(delivery);
        }
    }
    fn before_worker_exit(&self) {
        if let Some(lifecycle) = &self.lifecycle {
            lifecycle.before_worker_exit();
        }
    }
    fn after_cleanup(
        &self,
        worker: &Option<thread::ScopedJoinHandle<'_, Result<(), ProviderError>>>,
        pending: Option<&Delivery>,
    ) {
        if let Some(lifecycle) = &self.lifecycle {
            lifecycle.after_cleanup(worker, pending);
        }
    }
}
#[cfg(test)]
pub(super) struct LifecycleHooks {
    pub(super) worker_once: AtomicBool,
    pub(super) cleanup_once: AtomicBool,
    pub(super) work_returned: std::sync::mpsc::SyncSender<()>,
    pub(super) worker_exit: Mutex<std::sync::mpsc::Receiver<()>>,
    pub(super) cleanup_seen: std::sync::mpsc::SyncSender<()>,
    pub(super) physical_finished: std::sync::mpsc::SyncSender<()>,
    pub(super) resume_admission: Mutex<std::sync::mpsc::Receiver<()>>,
    pub(super) check_pending: fn(&Delivery),
}
#[cfg(test)]
impl LifecycleHooks {
    fn before_worker_exit(&self) {
        // At most two notifications in the controlled two-operation test;
        // only the first real worker exit is gated.
        self.work_returned.send(()).unwrap();
        if self.worker_once.swap(false, Ordering::SeqCst) {
            self.worker_exit
                .lock()
                .unwrap()
                .recv_timeout(std::time::Duration::from_secs(2))
                .unwrap();
        }
    }
    fn after_cleanup(
        &self,
        worker: &Option<thread::ScopedJoinHandle<'_, Result<(), ProviderError>>>,
        pending: Option<&Delivery>,
    ) {
        let Some(pending) = pending.filter(|d| d.is_consumed()) else {
            return;
        };
        if !self.cleanup_once.swap(false, Ordering::SeqCst) {
            return;
        }
        (self.check_pending)(pending);
        let worker = worker
            .as_ref()
            .expect("real worker handle disappeared before cleanup witness");
        assert!(
            !worker.is_finished(),
            "worker did not cross the cleanup boundary"
        );
        self.cleanup_seen.send(()).unwrap();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
        while !worker.is_finished() {
            assert!(
                std::time::Instant::now() < deadline,
                "real worker did not terminate"
            );
            thread::yield_now();
        }
        (self.check_pending)(pending);
        self.physical_finished.send(()).unwrap();
        self.resume_admission
            .lock()
            .unwrap()
            .recv_timeout(std::time::Duration::from_secs(2))
            .unwrap();
    }
}

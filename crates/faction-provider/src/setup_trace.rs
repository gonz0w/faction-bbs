//! Test-only setup chronology. Fixed atomics retain no transport or authority.
//! Snapshot/formatting is permitted only after all scoped producers have joined.
use std::{
    cell::RefCell,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering},
    },
    time::Instant,
};

const CAPACITY: usize = 128;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub(crate) enum Role {
    Outer,
    OwnHome,
    Resource,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub(crate) enum Kind {
    Binary,
    Http,
    Paced,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub(crate) enum Phase {
    Builder,
    Immutable,
    Tcp,
    Ssl,
    Postcheck,
    PacedWait,
    Lookup,
    Publication,
}
/// A hit reports no constructor work. Cold outcomes describe only publication;
/// actual builder, configuration and build intervals remain independently visible.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub(crate) enum SetupOutcome {
    None,
    Hit,
    ColdPublished,
    ColdUnpublished,
}
struct Slot {
    ready: AtomicBool,
    tag: AtomicU64,
    start: AtomicU64,
    end: AtomicU64,
}
impl Slot {
    fn new() -> Self {
        Self {
            ready: AtomicBool::new(false),
            tag: AtomicU64::new(0),
            start: AtomicU64::new(0),
            end: AtomicU64::new(0),
        }
    }
}
pub(crate) struct Trace {
    origin: Instant,
    enabled: AtomicBool,
    bad: AtomicBool,
    next: AtomicUsize,
    ordinals: [AtomicUsize; 9],
    slots: [Slot; CAPACITY],
}
#[derive(Clone)]
struct Context {
    trace: Arc<Trace>,
    role: Role,
}
thread_local! { static CONTEXT: RefCell<Option<Context>> = const { RefCell::new(None) }; }
pub(crate) fn scope_is_clear() -> bool {
    CONTEXT.with(|context| context.borrow().is_none())
}
pub(crate) struct Scope {
    previous: Option<Context>,
    _thread: std::marker::PhantomData<std::rc::Rc<()>>,
}
impl Drop for Scope {
    fn drop(&mut self) {
        CONTEXT.with(|context| {
            context.replace(self.previous.take());
        });
    }
}
impl Trace {
    pub(crate) fn new(origin: Instant) -> Arc<Self> {
        Arc::new(Self {
            origin,
            enabled: AtomicBool::new(false),
            bad: AtomicBool::new(false),
            next: AtomicUsize::new(0),
            ordinals: std::array::from_fn(|_| AtomicUsize::new(0)),
            slots: std::array::from_fn(|_| Slot::new()),
        })
    }
    pub(crate) fn arm(&self) {
        self.enabled.store(true, Ordering::SeqCst);
    }
    pub(crate) fn scope(self: &Arc<Self>, role: Role) -> Scope {
        let previous = CONTEXT.with(|context| {
            context.replace(Some(Context {
                trace: self.clone(),
                role,
            }))
        });
        if let Some(old) = &previous {
            old.trace.invalid();
            self.invalid();
        }
        Scope {
            previous,
            _thread: std::marker::PhantomData,
        }
    }
    fn invalid(&self) {
        self.bad.store(true, Ordering::SeqCst);
    }
    fn micros(&self, at: Instant) -> u64 {
        at.saturating_duration_since(self.origin)
            .as_micros()
            .try_into()
            .unwrap_or(u64::MAX)
    }
    fn emit(
        &self,
        connection: &Connection,
        phase: Phase,
        part: u8,
        start: Instant,
        complete: bool,
        outcome: SetupOutcome,
    ) {
        let end = Instant::now();
        let Ok(index) = self
            .next
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |n| {
                (n < CAPACITY).then_some(n + 1)
            })
        else {
            self.invalid();
            return;
        };
        let tag = connection.context.role as u64
            | ((connection.kind as u64) << 8)
            | ((connection.ordinal as u64) << 16)
            | ((phase as u64) << 24)
            | ((part as u64) << 32)
            | ((u64::from(complete)) << 40)
            | ((outcome as u64) << 48);
        let slot = &self.slots[index];
        slot.tag.store(tag, Ordering::Relaxed);
        slot.start.store(self.micros(start), Ordering::Relaxed);
        slot.end.store(self.micros(end), Ordering::Relaxed);
        slot.ready.store(true, Ordering::Release);
        if !complete {
            self.invalid();
        }
    }
    pub(crate) fn snapshot(&self) -> Snapshot {
        let mut bad = self.bad.load(Ordering::SeqCst);
        let mut points = Vec::with_capacity(CAPACITY);
        for slot in &self.slots[..self.next.load(Ordering::SeqCst)] {
            if !slot.ready.load(Ordering::Acquire) {
                bad = true;
                continue;
            }
            let tag = slot.tag.load(Ordering::Relaxed);
            points.push(Point {
                sequence: points.len() + 1,
                role: [Role::Outer, Role::OwnHome, Role::Resource][(tag & 255) as usize],
                kind: [Kind::Binary, Kind::Http, Kind::Paced][((tag >> 8) & 255) as usize],
                ordinal: ((tag >> 16) & 255) as usize,
                phase: [
                    Phase::Builder,
                    Phase::Immutable,
                    Phase::Tcp,
                    Phase::Ssl,
                    Phase::Postcheck,
                    Phase::PacedWait,
                    Phase::Lookup,
                    Phase::Publication,
                ][((tag >> 24) & 255) as usize],
                part: ((tag >> 32) & 255) as u8,
                complete: ((tag >> 40) & 255) == 1,
                outcome: [
                    SetupOutcome::None,
                    SetupOutcome::Hit,
                    SetupOutcome::ColdPublished,
                    SetupOutcome::ColdUnpublished,
                ][((tag >> 48) & 255) as usize],
                start_us: slot.start.load(Ordering::Relaxed),
                end_us: slot.end.load(Ordering::Relaxed),
            });
        }
        for role in [Role::Outer, Role::OwnHome, Role::Resource] {
            for kind in [Kind::Binary, Kind::Http, Kind::Paced] {
                let count = self.ordinals[role as usize * 3 + kind as usize].load(Ordering::SeqCst);
                for ordinal in 0..count {
                    let run: Vec<_> = points
                        .iter()
                        .filter(|p| p.role == role && p.kind == kind && p.ordinal == ordinal)
                        .collect();
                    let hit = run
                        .iter()
                        .any(|p| p.phase == Phase::Lookup && p.outcome == SetupOutcome::Hit);
                    let mut expected = Vec::new();
                    if kind == Kind::Paced {
                        expected.push((Phase::PacedWait, 0));
                    } else {
                        if kind == Kind::Binary {
                            expected.push((Phase::Tcp, 0));
                        }
                        expected.push((Phase::Lookup, 0));
                        if !hit {
                            expected.extend([(Phase::Builder, 0), (Phase::Immutable, 0)]);
                        }
                        // Both actual builds follow TCP; HTTP configuration
                        // precedes it. Hits contain no configuration/build phases.
                        if kind == Kind::Http {
                            expected.push((Phase::Tcp, 0));
                        }
                        if !hit {
                            expected.extend([(Phase::Immutable, 1), (Phase::Publication, 0)]);
                        }
                        expected.extend([(Phase::Ssl, 0), (Phase::Postcheck, 0)]);
                    }
                    if run.len() != expected.len()
                        || run.iter().zip(&expected).any(|(p, (phase, part))| {
                            p.phase != *phase
                                || p.part != *part
                                || !p.complete
                                || p.start_us > p.end_us
                                || match p.phase {
                                    Phase::Lookup => {
                                        p.outcome
                                            != if hit {
                                                SetupOutcome::Hit
                                            } else {
                                                SetupOutcome::None
                                            }
                                    }
                                    Phase::Publication => !matches!(
                                        p.outcome,
                                        SetupOutcome::ColdPublished | SetupOutcome::ColdUnpublished
                                    ),
                                    _ => p.outcome != SetupOutcome::None,
                                }
                        })
                        || run.windows(2).any(|w| w[0].end_us > w[1].start_us)
                    {
                        bad = true;
                    }
                }
            }
        }
        Snapshot {
            points,
            inconclusive: bad,
        }
    }
}
pub(crate) struct Connection {
    context: Context,
    kind: Kind,
    ordinal: usize,
}
impl Connection {
    pub(crate) fn begin(kind: Kind) -> Option<Self> {
        let context = CONTEXT.with(|context| context.borrow().clone())?;
        if !context.trace.enabled.load(Ordering::SeqCst) {
            return None;
        }
        if !matches!(
            (context.role, kind),
            (Role::Outer, Kind::Binary | Kind::Paced)
                | (Role::OwnHome, Kind::Binary | Kind::Http)
                | (Role::Resource, Kind::Http)
        ) {
            context.trace.invalid();
            return None;
        }
        let Ok(ordinal) = context.trace.ordinals[context.role as usize * 3 + kind as usize]
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |n| {
                (n < 8).then_some(n + 1)
            })
        else {
            context.trace.invalid();
            return None;
        };
        Some(Self {
            context,
            kind,
            ordinal,
        })
    }
    pub(crate) fn interval(&self, phase: Phase, part: u8) -> Interval<'_> {
        Interval {
            connection: self,
            phase,
            part,
            start: Instant::now(),
            complete: false,
            outcome: SetupOutcome::None,
        }
    }
}
pub(crate) struct Interval<'a> {
    connection: &'a Connection,
    phase: Phase,
    part: u8,
    start: Instant,
    complete: bool,
    outcome: SetupOutcome,
}
impl Interval<'_> {
    pub(crate) fn finish(mut self) {
        self.complete = true;
    }
    pub(crate) fn finish_as(mut self, outcome: SetupOutcome) {
        self.outcome = outcome;
        self.complete = true;
    }
    pub(crate) fn finish_optional(interval: Option<Self>) {
        if let Some(interval) = interval {
            interval.finish();
        }
    }
}
impl Drop for Interval<'_> {
    fn drop(&mut self) {
        self.connection.context.trace.emit(
            self.connection,
            self.phase,
            self.part,
            self.start,
            self.complete,
            self.outcome,
        );
    }
}
#[derive(Debug)]
pub(crate) struct Point {
    pub(crate) sequence: usize,
    pub(crate) role: Role,
    pub(crate) kind: Kind,
    pub(crate) ordinal: usize,
    pub(crate) phase: Phase,
    pub(crate) part: u8,
    pub(crate) start_us: u64,
    pub(crate) end_us: u64,
    pub(crate) complete: bool,
    pub(crate) outcome: SetupOutcome,
}
impl Point {
    pub(crate) fn line(&self) -> String {
        format!(
            "small_get_setup schema=2 sequence={} role={} kind={} ordinal={} phase={} part={} start_us={} end_us={} complete={} outcome={}",
            self.sequence,
            self.role as u8,
            self.kind as u8,
            self.ordinal,
            self.phase as u8,
            self.part,
            self.start_us,
            self.end_us,
            u8::from(self.complete),
            self.outcome as u8
        )
    }
}
#[derive(Debug)]
pub(crate) struct Snapshot {
    pub(crate) points: Vec<Point>,
    pub(crate) inconclusive: bool,
}
#[path = "setup_trace_tests.rs"]
mod tests;

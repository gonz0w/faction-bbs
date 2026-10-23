//! Synthetic codec requested-allocation/time probe, not RSS or capacity evidence.
use faction_articles::{Article, Headers};
use std::{
    alloc::{GlobalAlloc, Layout, System},
    sync::atomic::{AtomicUsize, Ordering},
    time::Instant,
};
struct Counting;
static LIVE: AtomicUsize = AtomicUsize::new(0);
static PEAK: AtomicUsize = AtomicUsize::new(0);
fn add(n: usize) {
    let live = LIVE.fetch_add(n, Ordering::SeqCst) + n;
    PEAK.fetch_max(live, Ordering::SeqCst);
}
// SAFETY: every request/deallocation delegates to System with the caller's
// unchanged layout/pointer. Counters never influence allocation or ownership.
unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, l: Layout) -> *mut u8 {
        let p = unsafe { System.alloc(l) };
        if !p.is_null() {
            add(l.size())
        }
        p
    }
    unsafe fn dealloc(&self, p: *mut u8, l: Layout) {
        unsafe { System.dealloc(p, l) };
        LIVE.fetch_sub(l.size(), Ordering::SeqCst);
    }
    unsafe fn alloc_zeroed(&self, l: Layout) -> *mut u8 {
        let p = unsafe { System.alloc_zeroed(l) };
        if !p.is_null() {
            add(l.size())
        }
        p
    }
    unsafe fn realloc(&self, p: *mut u8, l: Layout, n: usize) -> *mut u8 {
        let q = unsafe { System.realloc(p, l, n) };
        if !q.is_null() {
            LIVE.fetch_sub(l.size(), Ordering::SeqCst);
            add(n)
        }
        q
    }
}
#[global_allocator]
static ALLOCATOR: Counting = Counting;
fn main() {
    let mut body = ("x".repeat(998) + "\n").repeat(16);
    body.push_str(&"y".repeat(400));
    let a = Article::new(
        Headers {
            message_id: "<synthetic@fixture.invalid>".into(),
            from: "author@fixture.invalid".into(),
            date: "21 Oct 2026 18:00:00 +0000".into(),
            subject: "Synthetic allocation fixture".into(),
            newsgroups: vec!["faction.fixture".into()],
            path: "not-for-mail".into(),
            references: (0..16)
                .map(|i| format!("<ref-{i}@fixture.invalid>"))
                .collect(),
        },
        body,
    )
    .unwrap();
    let wire = a.to_wire().unwrap();
    let base = LIVE.load(Ordering::SeqCst);
    PEAK.store(base, Ordering::SeqCst);
    let start = Instant::now();
    for _ in 0..1000 {
        let parsed = Article::parse(std::hint::black_box(wire.as_bytes())).unwrap();
        std::hint::black_box(parsed);
    }
    let parse_ns = start.elapsed().as_nanos() / 1000;
    let parse_peak = PEAK.load(Ordering::SeqCst) - base;
    let base = LIVE.load(Ordering::SeqCst);
    PEAK.store(base, Ordering::SeqCst);
    let start = Instant::now();
    for _ in 0..1000 {
        std::hint::black_box(a.to_wire().unwrap());
    }
    let serialize_ns = start.elapsed().as_nanos() / 1000;
    let serialize_peak = PEAK.load(Ordering::SeqCst) - base;
    println!(
        "{{\"iterations\":1000,\"body_bytes\":{},\"wire_bytes\":{},\"parse_peak_requested_bytes\":{},\"serialize_peak_requested_bytes\":{},\"parse_mean_ns\":{},\"serialize_mean_ns\":{}}}",
        a.body().len(),
        wire.len(),
        parse_peak,
        serialize_peak,
        parse_ns,
        serialize_ns
    );
}

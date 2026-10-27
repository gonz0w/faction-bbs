//! Read-only host sampling; delta baselines never turn resets into false spikes.
use faction_unix::operator::{HostCounters, Interface, host_counters};
use std::{
    path::Path,
    time::{Duration, Instant},
};
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) enum Reading<T> {
    #[default]
    Unavailable,
    Warming,
    Value(T),
}
#[derive(Default)]
pub(super) struct Host {
    pub cpus: Option<u32>,
    pub ram: Option<u64>,
    pub free: Option<u64>,
    pub used: Option<u64>,
    pub load: Option<String>,
    pub disk: Option<(u64, u64)>,
    pub cpu: Reading<u8>,
    pub network: Reading<(u64, u64)>,
    pub interfaces: Option<usize>,
}
#[derive(Default)]
pub(super) struct Sampler {
    previous: Option<(Instant, HostCounters)>,
}
impl Sampler {
    pub fn reset(&mut self) {
        self.previous = None;
    }
    pub fn sample(&mut self, directory: &Path) -> Host {
        let current = host_counters();
        let now = Instant::now();
        let mut result = derive(
            &current,
            self.previous
                .as_ref()
                .map(|(at, old)| (now.saturating_duration_since(*at), old)),
        );
        if let Ok(v) = rustix::fs::statvfs(directory) {
            result.disk = u64::try_from(u128::from(v.f_bavail) * u128::from(v.f_frsize))
                .ok()
                .zip(u64::try_from(u128::from(v.f_blocks) * u128::from(v.f_frsize)).ok());
        }
        self.previous = Some((now, current));
        result
    }
}
fn derive(current: &HostCounters, previous: Option<(Duration, &HostCounters)>) -> Host {
    let mut result = Host {
        cpus: current.cpus,
        ram: current.physical,
        free: current.free,
        used: current
            .physical
            .zip(current.free)
            .and_then(|(a, b)| a.checked_sub(b)),
        load: current.milli_load.map(|a| {
            a.map(|v| format!("{}.{:02}", v / 1000, (v % 1000) / 10))
                .join(" ")
        }),
        interfaces: current.interfaces.as_ref().map(Vec::len),
        ..Host::default()
    };
    result.cpu = match (current.cpu, previous) {
        (None, _) => Reading::Unavailable,
        (Some(now), Some((_, old))) if current.cpus == old.cpus => old
            .cpu
            .map_or(Reading::Warming, |before| cpu_delta(before, now)),
        _ => Reading::Warming,
    };
    result.network = match (&current.interfaces, previous) {
        (None, _) => Reading::Unavailable,
        (Some(now), Some((dt, old))) => old
            .interfaces
            .as_ref()
            .map_or(Reading::Warming, |before| network_delta(before, now, dt)),
        _ => Reading::Warming,
    };
    result
}
fn cpu_delta(before: [u64; 5], now: [u64; 5]) -> Reading<u8> {
    let mut delta = [0u128; 5];
    for i in 0..5 {
        let Some(d) = now[i].checked_sub(before[i]) else {
            return Reading::Warming;
        };
        delta[i] = u128::from(d);
    }
    let total: u128 = delta.iter().sum();
    ((total - delta[4]) * 100)
        .checked_div(total)
        .map_or(Reading::Warming, |v| Reading::Value(v as u8))
}
fn network_delta(before: &[Interface], now: &[Interface], dt: Duration) -> Reading<(u64, u64)> {
    if before.len() != now.len() || dt.is_zero() {
        return Reading::Warming;
    }
    let (mut rx, mut tx) = (0u128, 0u128);
    for (a, b) in before.iter().zip(now) {
        if a.index != b.index {
            return Reading::Warming;
        }
        let Some(r) = b.received.checked_sub(a.received) else {
            return Reading::Warming;
        };
        let Some(t) = b.sent.checked_sub(a.sent) else {
            return Reading::Warming;
        };
        rx += u128::from(r);
        tx += u128::from(t);
    }
    let scaled = |v: u128| u64::try_from(v * 1_000_000_000 / dt.as_nanos()).ok();
    scaled(rx)
        .zip(scaled(tx))
        .map_or(Reading::Unavailable, Reading::Value)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cpu_and_network_deltas_measure_intervals_not_lifetime() {
        assert_eq!(cpu_delta([0; 5], [10, 0, 10, 0, 80]), Reading::Value(20));
        assert_eq!(cpu_delta([10; 5], [0; 5]), Reading::Warming);
        assert_eq!(cpu_delta([10; 5], [10; 5]), Reading::Warming);
        let a = [Interface {
            index: 1,
            received: 100,
            sent: 50,
        }];
        let b = [Interface {
            index: 1,
            received: 1124,
            sent: 562,
        }];
        assert_eq!(
            network_delta(&a, &b, Duration::from_secs(2)),
            Reading::Value((512, 256))
        );
        assert_eq!(
            network_delta(&b, &a, Duration::from_secs(2)),
            Reading::Warming
        );
        assert_eq!(network_delta(&a, &b, Duration::ZERO), Reading::Warming);
        assert_eq!(
            network_delta(&a, &[], Duration::from_secs(1)),
            Reading::Warming
        );
    }
    #[test]
    fn first_failed_or_changed_scope_never_fabricates_activity() {
        let current = HostCounters {
            cpus: Some(2),
            physical: Some(1024),
            free: Some(256),
            cpu: Some([1; 5]),
            interfaces: Some(vec![]),
            ..HostCounters::default()
        };
        let v = derive(&current, None);
        assert_eq!(v.cpu, Reading::Warming);
        assert_eq!(v.used, Some(768));
        assert_eq!(v.network, Reading::Warming);
        let failed = HostCounters::default();
        let v = derive(&failed, Some((Duration::from_secs(5), &current)));
        assert_eq!(v.cpu, Reading::Unavailable);
        assert_eq!(v.network, Reading::Unavailable);
        let changed = HostCounters {
            cpus: Some(4),
            ..current.clone()
        };
        assert_eq!(
            derive(&changed, Some((Duration::from_secs(5), &current))).cpu,
            Reading::Warming
        );
    }
}

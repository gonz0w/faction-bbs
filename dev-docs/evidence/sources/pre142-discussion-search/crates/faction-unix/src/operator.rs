//! Fixed native counter boundary; never configuration/host authority.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Interface {
    pub index: u64,
    pub received: u64,
    pub sent: u64,
}
#[derive(Clone, Debug, Default)]
pub struct HostCounters {
    pub cpus: Option<u32>,
    pub physical: Option<u64>,
    pub free: Option<u64>,
    pub cpu: Option<[u64; 5]>,
    pub milli_load: Option<[u64; 3]>,
    pub interfaces: Option<Vec<Interface>>,
}
unsafe extern "C" {
    fn faction_operator_stats(out: *mut u64, capacity: usize) -> i32;
}
pub fn host_counters() -> HostCounters {
    let mut raw = [0u64; 64];
    // SAFETY: C receives exactly 64 initialized writable scalar slots, borrows
    // them only until return and never stores pointers. Native buffers are fixed;
    // output contains counters only, no kernel pointers/addresses/names/content.
    if unsafe { faction_operator_stats(raw.as_mut_ptr(), raw.len()) } != 0 {
        return HostCounters::default();
    }
    decode(&raw)
}
fn decode(raw: &[u64; 64]) -> HostCounters {
    let flags = raw[0];
    let basic = flags & 1 != 0 && raw[1] > 0 && raw[2] > 0;
    let mut value = HostCounters {
        cpus: basic.then(|| u32::try_from(raw[1]).ok()).flatten(),
        physical: basic.then_some(raw[2]),
        free: (basic && flags & 2 != 0 && raw[3] <= raw[2]).then_some(raw[3]),
        cpu: (flags & 4 != 0).then(|| raw[4..9].try_into().expect("fixed CPU slots")),
        milli_load: (flags & 8 != 0).then(|| raw[9..12].try_into().expect("fixed load slots")),
        interfaces: None,
    };
    if flags & 16 != 0 && raw[15] <= 16 {
        let mut items = Vec::with_capacity(raw[15] as usize);
        for i in 0..raw[15] as usize {
            let at = 16 + i * 3;
            if raw[at] == 0 || items.iter().any(|v: &Interface| v.index == raw[at]) {
                return value;
            }
            items.push(Interface {
                index: raw[at],
                received: raw[at + 1],
                sent: raw[at + 2],
            });
        }
        items.sort_unstable_by_key(|v| v.index);
        value.interfaces = Some(items);
    }
    value
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn invalid_native_metrics_are_unavailable_independently() {
        let mut r = [0; 64];
        r[0] = 31;
        r[1] = 2;
        r[2] = 1024;
        r[3] = 1025;
        r[15] = 17;
        let v = decode(&r);
        assert_eq!(v.cpus, Some(2));
        assert_eq!(v.free, None);
        assert!(v.interfaces.is_none());
        r[15] = 2;
        r[16] = 1;
        r[19] = 1;
        assert!(decode(&r).interfaces.is_none());
        r[19] = 2;
        assert_eq!(decode(&r).interfaces.unwrap().len(), 2);
    }
}

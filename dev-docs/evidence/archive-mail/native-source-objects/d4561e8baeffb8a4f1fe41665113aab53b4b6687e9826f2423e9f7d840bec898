use faction_terminal::graphics::*;
fn main() {
    let mut wire = 0;
    let mut max = 0;
    let mut capacity = 0;
    for i in 0..CHUNKS {
        let s = chunk(i).unwrap();
        wire += s.len();
        max = max.max(s.len());
        capacity = capacity.max(s.capacity());
    }
    println!(
        "{{\"rgb_static_bytes\":{},\"encoded_bytes\":51200,\"wire_bytes\":{},\"chunks\":{},\"max_command_bytes\":{},\"chunk_capacity_bytes\":{},\"probe_struct_bytes\":{},\"gate_struct_bytes\":{},\"query_bytes\":{},\"cleanup_bytes\":{}}}",
        RGB.len(),
        wire,
        CHUNKS,
        max,
        capacity,
        std::mem::size_of::<Probe>(),
        std::mem::size_of::<Gate>(),
        QUERY.len(),
        CLEANUP.len()
    );
}

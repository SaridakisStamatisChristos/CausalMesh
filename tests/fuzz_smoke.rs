use causalmesh::codec::{decode_counters, decode_log};
use causalmesh::protocol::decode_wire;

fn next(seed: &mut u64) -> u64 {
    *seed = seed
        .wrapping_mul(2862933555777941757)
        .wrapping_add(3037000493);
    *seed
}

#[test]
fn bounded_parser_fuzz_smoke_never_panics() {
    let mut seed = 0x1234_5678_9abc_def0u64;
    for len in 0usize..512 {
        for _ in 0..8 {
            let mut bytes = vec![0u8; len];
            for byte in &mut bytes {
                *byte = (next(&mut seed) >> 32) as u8;
            }
            let _ = decode_wire(&bytes);
            let _ = decode_log(&bytes);
            let _ = decode_counters(&bytes);
        }
    }
}

use causalmesh::WireMessage;

#[test]
fn bounded_parser_smoke_over_deterministic_noise() {
    let mut x = 0x9e3779b97f4a7c15u64;
    for len in 0..2048usize {
        let mut bytes = vec![0u8; len];
        for byte in &mut bytes {
            x ^= x << 7;
            x ^= x >> 9;
            x ^= x << 8;
            *byte = x as u8;
        }
        let _ = WireMessage::decode(&bytes);
    }
}

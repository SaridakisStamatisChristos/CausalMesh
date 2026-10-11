use causalmesh::{storage::read_snapshot, CausalContext, Replica, WireMessage};
use tempfile::TempDir;

fn noise(seed: &mut u64, len: usize) -> Vec<u8> {
    let mut bytes = vec![0u8; len];
    for byte in &mut bytes {
        *seed ^= *seed << 7;
        *seed ^= *seed >> 9;
        *seed ^= *seed << 8;
        *byte = *seed as u8;
    }
    bytes
}

#[test]
fn wire_parser_smoke_over_deterministic_noise() {
    let mut seed = 0x9e37_79b9_7f4a_7c15u64;
    for len in 0..512usize {
        let bytes = noise(&mut seed, len);
        let _ = WireMessage::decode(&bytes);
    }
}

#[test]
fn causal_context_decoder_smoke_over_deterministic_noise() {
    let mut seed = 0xa076_1d64_78bd_642fu64;
    for len in 0..512usize {
        let bytes = noise(&mut seed, len);
        if let Ok(context) = serde_json::from_slice::<CausalContext>(&bytes) {
            let _ = context.validate_canonical();
        }
    }
}

#[test]
fn snapshot_decoder_smoke_over_deterministic_noise() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("snapshot-noise.json");
    let mut seed = 0xe703_7ed1_a0b4_28dbu64;
    for len in 0..256usize {
        std::fs::write(&path, noise(&mut seed, len)).unwrap();
        let _ = read_snapshot(&path);
    }
}

#[test]
fn operation_log_parser_smoke_over_deterministic_noise() {
    let mut seed = 0x8ebc_6af0_9c88_c6e3u64;
    for len in 0..64usize {
        let dir = TempDir::new().unwrap();
        let replica = Replica::init(dir.path()).unwrap();
        drop(replica);
        std::fs::write(dir.path().join("ops.log"), noise(&mut seed, len)).unwrap();
        let _ = Replica::open(dir.path());
    }
}

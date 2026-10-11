# Development

## Required toolchain

Stable Rust and Python 3 (manifest verification only).

## One-command critical verification

```bash
cargo fmt --check && \
cargo clippy --all-targets --all-features -- -D warnings && \
cargo test --all-targets && \
cargo build --release && \
python3 scripts/verify_manifest.py
```

## Test topology

- `crdt_properties.rs` — algebra, causality, deletion, canonical encoding;
- `reference_model.rs` — independent pairwise frontier oracle;
- `exhaustive_small_state.rs` — all 24 permutations of a bounded four-operation history;
- `integration_sync.rs` — partitions, stale peers, duplicate/retry sync;
- `property_campaign.rs` — 64 deterministic randomized multi-replica histories with sync, delete, restart, snapshot;
- `recovery.rs` — torn record, counter repair, missing identity, snapshot corruption;
- `adversarial.rs` — malformed/impossible inputs, clone identity, transaction-before-commit;
- `fuzz_smoke.rs` — bounded deterministic parser byte fuzzing;
- `cli_smoke.rs` — documented command path.

Failure seeds from the deterministic property campaign are printed in assertion messages and can be reproduced directly by changing the seed loop to the reported seed.

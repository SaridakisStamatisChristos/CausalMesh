# Development

Required local gates:

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets
cargo build --release
python scripts/verify_manifest.py
```

The CI matrix runs the Rust verification on Ubuntu, macOS, and Windows. Linux additionally verifies the evidence manifest and runs CLI smoke commands.

Do not weaken causal contexts to max-only vectors: sparse out-of-order dots are required for anti-entropy correctness. Do not add destructive history GC without a proven replica-retirement/stability protocol.

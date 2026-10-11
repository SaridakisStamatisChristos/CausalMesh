# Development

Required local gates:

```bash
cargo fmt --check
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked --all-targets
cargo build --locked --release
python scripts/verify_manifest.py
```

The CI matrix executes Rust verification on Ubuntu, macOS, and Windows; Ubuntu additionally runs the benchmark smoke, RustSec dependency audit, and evidence-manifest verification.

## Release artifact procedure

1. Start from a commit whose required CI jobs are green.
2. Run the required local/CI gates above with the committed `Cargo.lock`.
3. Build `cargo build --locked --release` on each target platform.
4. Run the installed/release CLI smoke against two fresh replica directories.
5. Record the source commit SHA and Rust toolchain with the produced binary.
6. Do not call a package production-ready if `evidence/FINAL_AUDIT.md` or `docs/LIMITATIONS.md` still contains unresolved production blockers.

Do not weaken causal contexts to max-only vectors: sparse out-of-order dots are required for anti-entropy correctness. Do not add destructive history GC without a proven replica-retirement/causal-stability protocol.

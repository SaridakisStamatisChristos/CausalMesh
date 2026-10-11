# Benchmarks

Correctness is the v0.1 release priority. No performance-superiority claim is made.

The repository includes `examples/bench_smoke.rs`, a dependency-free measurement harness for:

- local put throughput;
- local get throughput;
- zero-difference sync;
- delta sync;
- reopen/recovery.

Run in release mode:

```bash
cargo run --release --example bench_smoke
```

Results are environment-dependent. Record OS, CPU, filesystem, Rust version, and storage medium alongside any published numbers. CI executes build/correctness gates but does not treat shared-runner timing as a performance gate.

# Benchmarks

Correctness is the release gate. CausalMesh makes **no performance-superiority claim**.

`scripts/benchmark_smoke.py` is an observational process-level smoke benchmark using the release CLI, not a statistically controlled microbenchmark. CI stores the JSON result as an artifact and records environment metadata.

## Observed smoke run

GitHub Actions run `38162454086`, job `benchmark-smoke`, commit `4714976e7dd8b3b71660bf12f5d2dc08ea8798c0`:

- Ubuntu 24.04.5, x86_64 Azure hosted runner;
- Rust `1.99.0 (b940084d7 2026-09-28)`;
- CausalMesh `0.1.0`;
- 50 durable CLI puts: 0.083207333 s total, about 600.91 ops/s, mean 1.664 ms/op;
- 50 CLI gets: 0.051703751 s total, about 967.05 ops/s;
- reopen/verify: 1.115 ms;
- zero-delta sync: 1.122 ms;
- 10-operation delta sync: 1.707 ms;
- 100-operation delta sync: 3.595 ms;
- two-replica concurrent-conflict convergence: 1.362 ms;
- ten-replica gossip scenario: 65.91 ms;
- 100-replica construction + sync: 0.855 s;
- 100-replica summary JSON: 4,733 bytes;
- 100 operations in hub log: 16,790 bytes;
- snapshot creation for that hub: 1.595 ms.

Artifact ID: `11710357713`.

These numbers include process startup and hosted-runner noise. They are useful only as reproducible smoke evidence. They are not suitable for comparative performance claims.

## Future controlled campaign

A publication/production benchmark should separately measure in-process put/get, append/fsync latency, cold/warm recovery, delta-size scaling, metadata growth, snapshot creation, and multi-replica gossip while recording CPU, filesystem, durability mode, operation sizes, run counts, distributions, and direct competitor baselines.

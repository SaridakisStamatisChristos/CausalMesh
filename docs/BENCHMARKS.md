# Benchmarks

Correctness is the release gate; no comparative performance claim is made.

The v0.1 benchmark plan covers local put/get, append latency, reopen/recovery, zero/small/large delta sync, 2/10 replica convergence, 100-replica causal metadata overhead, snapshot creation, and persistent bytes per operation.

Automated trustworthy benchmark numbers are not yet committed because this campaign does not have a controlled performance runner. Any future results must record CPU, OS, Rust version, filesystem, durability mode, dataset, replica topology, operation sizes, run count, and statistical summary. See `evidence/CLAIMS.md`: performance remains UNVERIFIED/NOT CLAIMED.

# CausalMesh

A deterministic, offline-first replicated state engine with causal consistency, conflict-free convergence, and auditable anti-entropy.

CausalMesh is an embedded Rust library plus CLI for replicas that must mutate while disconnected and later reconcile without a coordinator. Version 0.1 implements `Map<Key, MV-Register<Value>>` with causally tracked deletes, exact dotted causal contexts, append-only durable logs, bounded parsers, deterministic JSON wire messages, in-process bidirectional anti-entropy, immutable snapshots, and fail-closed corruption/version checks.

## Status

This repository is an experimental systems implementation. It is designed for evidence-driven verification and does **not** claim Byzantine security, authenticated peers, safe destructive tombstone GC, destructive log compaction, or automatic clone prevention. See `docs/LIMITATIONS.md` and `evidence/CLAIMS.md`.

## Quick start

```bash
cargo build --release
cargo run -- init /tmp/a
cargo run -- init /tmp/b
cargo run -- put /tmp/a user:42 alice
cargo run -- put /tmp/b user:42 bob
cargo run -- sync /tmp/a /tmp/b
cargo run -- get /tmp/a user:42 --json
cargo run -- verify /tmp/a
```

After the sync, both concurrent values are visible. A later write performed by a replica that has observed both values causally dominates both.

## CLI

```text
causalmesh init <DIR>
causalmesh put <DIR> <KEY> <VALUE>
causalmesh get <DIR> <KEY> [--json]
causalmesh delete <DIR> <KEY>
causalmesh status <DIR>
causalmesh verify <DIR>
causalmesh export-summary <DIR>
causalmesh sync <DIR-A> <DIR-B>
causalmesh dump-log <DIR>
causalmesh snapshot <DIR>
```

## Core correctness shape

- Operation IDs are `(replica UUID, monotonically reserved counter)`.
- The causal context is a contiguous per-replica base plus a sparse cloud of dots, so out-of-order receipt never invents missing knowledge.
- Replicated state is the immutable set of validated operations. Register values are the causally maximal operations for a key.
- Deletes are first-class causal operations, not absence.
- Anti-entropy sends operations whose dots are absent from the peer's exact causal summary.
- Duplicate delivery is idempotent. Same ID with different payload fails closed.
- Local counter reservation is durably synced before an operation is appended; committed operations are fsynced before success is returned.
- Unsupported persistent/wire versions and checksum failures fail closed.

## Verification

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets
cargo build --release
python scripts/verify_manifest.py
```

CI runs the Rust gates on Linux, macOS, and Windows. The test suite includes independent-reference checks, exhaustive delivery permutations for a bounded case, property testing, partition/rejoin scenarios, unknown-outcome retry behavior, recovery/corruption tests, adversarial batch validation, and bounded parser noise smoke.

## Architecture

Read the design in `docs/ARCHITECTURE.md`, causal model in `docs/CAUSALITY.md`, CRDT semantics in `docs/CRDT_SEMANTICS.md`, synchronization in `docs/ANTI_ENTROPY.md`, and evidence closure in `evidence/`.

## License

No open-source license is granted by this repository at this time. The repository owner may add or change licensing later.

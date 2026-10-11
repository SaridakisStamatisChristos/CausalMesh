# CausalMesh

A deterministic, offline-first replicated state engine with causal consistency, conflict-free convergence, and auditable anti-entropy.

CausalMesh is an embedded Rust library plus CLI for replicas that must mutate while disconnected and later reconcile without a coordinator. Version 0.1 implements `Map<Key, MV-Register<Value>>` with causally tracked deletes, exact dotted causal contexts, append-only durable logs, bounded parsers, deterministic JSON wire messages, in-process bidirectional anti-entropy, immutable snapshots, and fail-closed corruption/version checks.

## Status

This repository is an evidence-backed experimental systems implementation, **not a production-ready distributed database**. It does not claim Byzantine security, authenticated peers, safe destructive tombstone GC, destructive log compaction, or transparent live-clone prevention. A peer presenting the local replica UUID is rejected before causal summary exchange, so cloned identities fail closed at synchronization time. See `docs/LIMITATIONS.md` and `evidence/CLAIMS.md`.

## Quick start

```bash
cargo build --locked --release
cargo run --locked -- init /tmp/a
cargo run --locked -- init /tmp/b
cargo run --locked -- put /tmp/a user:42 alice
cargo run --locked -- put /tmp/b user:42 bob
cargo run --locked -- sync /tmp/a /tmp/b
cargo run --locked -- get /tmp/a user:42 --json
cargo run --locked -- verify /tmp/a
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
- Duplicate delivery is idempotent; identical duplicates inside a batch are coalesced. Same ID with different payload fails closed.
- Synchronization rejects a peer presenting the local durable replica UUID before summary exchange.
- Local counter reservation is durably synced before the operation-log append; committed operations are synced before success is returned.
- Unsupported persistent/wire versions and claimed corruption classes fail closed.
- Destructive tombstone/history GC is intentionally absent because safe causal stability cannot be proven without membership/retirement semantics.

## Verification

```bash
cargo fmt --check
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked --all-targets
cargo build --locked --release
python scripts/verify_manifest.py
```

CI runs the Rust gates on Linux, macOS, and Windows. The suite includes independent-reference checks, exhaustive bounded delivery permutations, property/model-based generated histories, partition/rejoin scenarios, deterministic crash injection, unknown-outcome retry, recovery/corruption tests, adversarial batch validation, sync-time clone detection, and bounded deterministic parser fuzz-smoke across disk records, wire messages, causal contexts, and snapshots.

## Architecture and evidence

Read `docs/ARCHITECTURE.md`, `docs/CAUSALITY.md`, `docs/CRDT_SEMANTICS.md`, `docs/ANTI_ENTROPY.md`, `docs/STORAGE_FORMAT.md`, and the evidence ledgers under `evidence/`.

## License

No open-source license is granted by this repository at this time. The repository owner may add or change licensing later.

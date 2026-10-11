# CausalMesh

**A deterministic, offline-first replicated state engine with causal consistency, conflict-free convergence, and auditable anti-entropy.**

CausalMesh is an embedded Rust engine for applications that must keep accepting local writes while disconnected and converge after replicas reconnect. v0.1 deliberately focuses on one rigorously specified replicated type: `Map<Key, MVRegister<Value>>`.

## What v0.1 guarantees

- Stable UUID replica identity stored separately from history.
- 1-based operation dots `(replica_id, counter)` with origin-counter validation.
- Version-vector causal contexts; wall clocks are never used for causality.
- MV-register semantics that preserve causally concurrent writes and delete/write conflicts.
- Idempotent duplicate delivery and fail-closed conflicting duplicate IDs.
- Exact received counter ranges for anti-entropy, so out-of-order delivery cannot hide holes.
- Batch validation before durable peer mutation.
- Checksummed/versioned operation, counter, snapshot, and wire formats.
- Bounded parsers and explicit key/value/batch/context/message limits.
- Append-and-sync local durability: success is returned only after operation and counter evidence are synced.
- Recovery from a recognizable torn final operation/counter record.
- Deterministic logical-state serialization based on ordered maps and stable frontier ordering.
- No unsafe tombstone/history garbage collection.

See [`docs/CRDT_SEMANTICS.md`](docs/CRDT_SEMANTICS.md), [`docs/CAUSALITY.md`](docs/CAUSALITY.md), and [`docs/STORAGE_FORMAT.md`](docs/STORAGE_FORMAT.md) for exact semantics.

## Quick start

```bash
cargo build --release

./target/release/causalmesh init /tmp/a
./target/release/causalmesh init /tmp/b
./target/release/causalmesh put /tmp/a user:42 alice
./target/release/causalmesh sync /tmp/a /tmp/b
./target/release/causalmesh get /tmp/b user:42
./target/release/causalmesh verify /tmp/a
```

Machine-readable output is available with `--json` for scalar command results; status, summaries, logs, snapshots, and sync reports are JSON by default.

## Library API

```rust
use causalmesh::{sync_pair, Replica};

let mut a = Replica::open("./replica-a")?;
let mut b = Replica::open("./replica-b")?;

a.put("user:42", b"alice".to_vec())?;
sync_pair(&mut a, &mut b)?;
assert_eq!(b.get("user:42")?, vec![b"alice".to_vec()]);
# Ok::<(), causalmesh::Error>(())
```

## Verification

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets
cargo build --release
python3 scripts/verify_manifest.py
```

The CI matrix runs on Linux, macOS, and Windows. Correctness campaigns include CRDT algebra, an independent frontier oracle, exhaustive small-state delivery permutations, deterministic randomized partition/sync/restart histories, recovery tests, adversarial input tests, and bounded parser fuzz smoke.

## Deliberate limitations

v0.1 does **not** destructively compact the operation log or garbage-collect tombstones. Without explicit membership/retirement knowledge, deleting history can make a valid stale replica impossible to synchronize safely or can resurrect deleted data. `causalmesh compact` therefore writes a validated snapshot but retains authoritative history.

The protocol is transport-independent and the CLI currently performs in-process directory-to-directory synchronization. Authentication, encryption, remote TCP transport, consensus/linearizability, and Byzantine peer tolerance are not claimed. Checksums detect accidental corruption; they do not authenticate peers.

See [`docs/LIMITATIONS.md`](docs/LIMITATIONS.md) and [`evidence/CLAIMS.md`](evidence/CLAIMS.md).

## Repository evidence

The `evidence/` directory is part of the product:

- `REQUIREMENTS.md` — requirement/invariant to implementation/test mapping.
- `ASSUMPTIONS.md` — autonomous decisions and invalidation triggers.
- `CLAIMS.md` — calibrated claim states.
- `FAILURE_REPAIR_LEDGER.md` — retained defect/repair history.
- `LOCAL_VERIFICATION.md` — commands and observed results.
- `FINAL_AUDIT.md` — adversarial completion audit.
- `MANIFEST.sha256` — repository integrity manifest, excluding itself to avoid a self-hash cycle.

## License

No open-source license is granted by this repository. All rights are reserved unless and until the repository owner publishes a license.

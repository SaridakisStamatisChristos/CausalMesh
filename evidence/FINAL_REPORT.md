# CausalMesh R7 Final Report

## Identity

- Repository: `SaridakisStamatisChristos/CausalMesh`
- Working/finalization branch: `r7/zero-to-hero`
- Pull request: `#1`
- Language: Rust 2021
- Package/crate: `causalmesh` 0.1.0

## Replication design

- CRDT: `Map<Key, MV-Register<Value>>`.
- Causal metadata: exact dotted causal context using per-replica contiguous base + sparse cloud.
- Operation ID: `(128-bit replica UUID, positive monotonic u64 counter)`.
- Delete semantics: causal delete operation; no absence-based deletion.
- Anti-entropy: peer identity validation, exact summary exchange, missing-dot operation transfer, full-batch validation, durable append, acknowledgement/update.
- Bootstrap: transfer full causal operation history, not only visible values.
- Clone handling: same replica UUID peer is rejected before summary exchange.

## Persistence

- `identity.json`: immutable, versioned, SHA-256 checked.
- `counter.log`: append-only durable counter reservations.
- `ops.log`: append-only versioned framed records with bounded length and CRC32.
- snapshots: immutable complete operation-set envelope with SHA-256; not yet a recovery accelerator.
- `STATE_VERSION = LOG_VERSION = WIRE_VERSION = SNAPSHOT_VERSION = 1`.
- compaction: no destructive history truncation in v0.1.
- tombstone GC: not implemented because safe causal stability is not proven.
- replica retirement: not implemented.

## Verification campaign

- 52 named Rust tests after the four-surface fuzz-smoke expansion.
- 32 generated multi-replica property/model histories per proptest invocation.
- independent reference-model comparisons.
- exhaustive delivery checking: 6 permutations for three concurrent operations; 24 permutations for a four-operation/two-replica/two-key bounded universe.
- deterministic crash injection after counter reservation, after local operation persistence, and after received-batch persistence.
- sender/receiver unknown-outcome restart/retry scenarios.
- chain, star, seeded gossip, stale reconnect, reverse order, overlap and bootstrap scenarios.
- adversarial corruption/version/bounds/clone/duplicate tests.
- bounded deterministic fuzz-smoke: disk record parser, wire parser, causal-context decoder, snapshot decoder.

## R7 falsification findings repaired

1. cloned replica IDs could be masked by equal dot summaries — repaired by identity rejection before summary exchange;
2. identical duplicates inside one incoming batch could be staged more than once — repaired by intra-batch coalescing;
3. oversized identity/snapshot files were size-checked after read/allocation — repaired by metadata prechecks;
4. initial parser fuzz-smoke coverage was incomplete — expanded to all requested parser surfaces.

Failure and environmental history is preserved in `FAILURE_REPAIR_LEDGER.md`.

## Cross-platform/package evidence

GitHub Actions run `38162454086` on final code before evidence-only edits passed format, Clippy, tests, release build and CLI smoke on Linux, macOS and Windows. Benchmark smoke passed. The final evidence-only tree must pass the same gates plus RustSec and manifest verification before merge.

## Benchmark smoke

Run `38162454086` (Ubuntu x86_64, Rust 1.99.0) observed roughly 600.91 durable CLI puts/s and 967.05 CLI gets/s, 1.122 ms zero-delta sync, 1.707 ms 10-op delta, 3.595 ms 100-op delta, 65.91 ms for the scripted ten-replica gossip scenario, and 0.855 s for scripted 100-replica construction/sync. These are hosted-runner process-level smoke observations, not superiority evidence.

## Production readiness

**NO.** The engine is a rigorous experimental/research implementation. Production blockers include authenticated/encrypted transport, membership/retirement semantics, causal-stability-based safe GC/compaction, snapshot recovery acceleration, migration tooling, deeper power-cut testing, and long-running fuzzing.

## Recommended next steps

1. Add an authenticated transport while preserving transport-independent replication semantics.
2. Define membership epochs and explicit replica retirement; only then design causal-stability GC.
3. Add snapshot-based recovery and peer-safe log compaction.
4. Run filesystem/power-cut fault campaigns and coverage-guided fuzzing.
5. Add durable-format migration tooling and compatibility fixtures.
6. Run controlled in-process performance comparisons before any performance claim.

`FINAL_MANIFEST_STATUS: PENDING`

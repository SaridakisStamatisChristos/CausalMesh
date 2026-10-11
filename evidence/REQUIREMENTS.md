# Requirement Coverage Ledger

Status legend: `OPEN`, `VERIFIED`, `PARTIAL`, `UNVERIFIED`, `FAILED`, `N/A`.

| R-ID | Requirement | Source | Criticality | Implementation surface | Acceptance evidence | Initial status |
|---|---|---|---|---|---|---|
| H1 | Same valid logical operation set converges regardless of delivery order. | USER | HARD | `crdt.rs` | algebra + exhaustive permutations + randomized convergence | OPEN |
| H2 | Duplicate operation replay is idempotent. | USER | HARD | `CrdtState::apply`, `Replica::receive_operations` | duplicate/retry tests | OPEN |
| H3 | Concurrent merge is commutative. | USER | HARD | `CrdtState::merge` | algebra test | OPEN |
| H4 | Merge is associative. | USER | HARD | `CrdtState::merge` | algebra test | OPEN |
| H5 | Causal newer operations dominate observed older ones; concurrency remains distinct. | USER | HARD | version vectors + frontier | causality/delete/write tests + reference model | OPEN |
| H6 | Concurrent same-key writes are not silently lost. | USER | HARD | MV-register frontier | concurrent-write test | OPEN |
| H7 | Replica identity survives restart; missing identity with history fails closed. | USER | HARD | immutable `identity.json` | restart + missing-identity test | OPEN |
| H8 | Local operation IDs are unique `(replica_id,counter)` and counters never move backward. | USER | HARD | counter log + recovery | counter repair/rollback tests | OPEN |
| H9 | Reported-committed local mutation survives declared crash boundary. | USER | HARD | operation fsync then counter fsync before success | recovery tests + code path inspection | OPEN |
| H10 | Recovery cannot decrement/reuse IDs, invent ops, or accept partial record as complete. | USER | HARD | framed log + counter repair | torn-tail/counter tests | OPEN |
| H11 | Claimed canonical logical serialization is deterministic. | USER | HARD | BTreeMap + stable dot ordering | canonical-bytes test | OPEN |
| H12 | Disk and wire versions fail closed. | USER | HARD | version fields/validators | unsupported wire/disk tests | OPEN |
| H13 | Parsers enforce explicit bounds before large decode/application. | USER | HARD | codec/protocol/operation bounds | oversize/range/fuzz tests | OPEN |
| H14 | Repeated sync with eventual delivery reaches equal logical state. | USER | HARD | exact-range anti-entropy | partition/randomized convergence tests | OPEN |
| H15 | Sync of converged replicas creates no new logical mutations. | USER | HARD | missing-range discovery | repeated-sync test | OPEN |
| H16 | Tombstones/history are not garbage-collected without proof of safety. | USER | HARD | non-destructive compact | inspection + stale-peer delete test | OPEN |
| H17 | Claimed corruption classes fail closed. | USER | HARD | CRC/version/frame checks | snapshot/log corruption tests | OPEN |
| H18 | Invalid peer payload cannot mutate durable state before validation passes. | USER | HARD | preview-then-append receive | invalid-batch reopen test | OPEN |
| R19 | Stable causal metadata without wall clocks. | USER | HARD | `VersionVector` | unit/integration tests + docs | OPEN |
| R20 | Efficient difference discovery must not require full state transfer each sync. | USER | HARD | exact contiguous received ranges | summary/delta tests | OPEN |
| R21 | Duplicate, reorder, delayed, overlapping delivery safe; FIFO not assumed. | USER | HARD | operation-set semantics | exhaustive/randomized/retry tests | OPEN |
| R22 | Unknown outcome receive-persist-before-ack retry is safe. | USER | HARD | idempotent receive | explicit lost-ack test | OPEN |
| R23 | New replica can bootstrap without losing causality. | USER | HARD | full operation delta from empty summary | sync/bootstrap test path | OPEN |
| R24 | Snapshot is versioned/checksummed and contains causal frontier/inventory. | USER | HARD | `SnapshotData` | snapshot/reopen/corruption tests | OPEN |
| R25 | Unsafe destructive compaction is not performed. | USER | HARD | `compact_safely` | code/CLI audit | OPEN |
| R26 | Required CLI commands exist and use production paths. | USER | HARD | `main.rs` | CLI smoke | OPEN |
| R27 | Reusable library API exists beneath CLI. | USER | HARD | `Replica`, `sync_pair` | compile + integration tests | OPEN |
| R28 | Independent reference model/oracle exists. | USER | HARD | `tests/reference_model.rs` | test result | OPEN |
| R29 | Bounded exhaustive small-state campaign exists. | USER | HARD | `tests/exhaustive_small_state.rs` | 24 permutations pass | OPEN |
| R30 | Deterministic randomized multi-replica property campaign exists. | USER | HARD | `tests/property_campaign.rs` | 64 seeds pass | OPEN |
| R31 | Deterministic crash injection covers material append boundaries. | USER | HARD | `storage::FailPoint` | recovery tests | OPEN |
| R32 | Malformed/adversarial peers fail closed. | USER | HARD | validators + preview receive | adversarial suite | OPEN |
| R33 | Bounded parser fuzz smoke executes in CI. | USER | HARD | `tests/fuzz_smoke.rs` | CI test result | OPEN |
| R34 | Linux, macOS, Windows build/test lanes are required. | USER | HARD | GitHub Actions matrix | all three lanes green | OPEN |
| R35 | Release build, package metadata, installed CLI smoke succeed. | USER | HARD | Cargo + CI | build/install/smoke evidence | OPEN |
| R36 | Evidence manifest hashes authoritative files and fails closed. | USER | HARD | `scripts/verify_manifest.py` | CI manifest job | OPEN |
| R37 | Required documentation exists and does not overclaim. | USER | HARD | `README.md`, `docs/*` | final audit | OPEN |
| R38 | Dependency vulnerability audit executes. | USER | SHOULD | RustSec CI action | audit job | OPEN |
| R39 | Performance harness records environment but makes no superiority claim. | USER | SHOULD | `examples/bench_smoke.rs` | release build/run when available | OPEN |
| R40 | Replica retirement is explicit if implemented; otherwise documented as absent. | USER | SHOULD | docs | limitations/final audit | OPEN |

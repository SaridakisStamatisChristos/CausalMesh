# Requirement Ledger

Status vocabulary: `VERIFIED`, `PARTIAL`, `UNVERIFIED`, `FAILED`, `N/A`.

| ID | Requirement / invariant | Criticality | Implementation surface | Acceptance evidence | Status |
|---|---|---|---|---|---|
| R-H1 | Same valid logical operation set converges independent of delivery order. | HARD | `model`, replication tests | exhaustive/property/reference tests | UNVERIFIED |
| R-H2 | Duplicate operation delivery is idempotent. | HARD | `OperationSet::insert`, `receive_batch` | duplicate-delivery test | UNVERIFIED |
| R-H3 | Concurrent merge is commutative. | HARD | operation-set union semantics | permutation/property tests | UNVERIFIED |
| R-H4 | Merge is associative. | HARD | operation-set union semantics | multi-replica/permutation tests | UNVERIFIED |
| R-H5 | Causal newer operations dominate observed older same-key operations. | HARD | `Operation.context`, frontier projection | resolving-write test | UNVERIFIED |
| R-H6 | Concurrent same-key values are not silently lost. | HARD | MV-register frontier | concurrent-write test | UNVERIFIED |
| R-H7 | Replica identity is stable and missing/corrupt identity fails closed. | HARD | `identity.json` | recovery/adversarial inspection | UNVERIFIED |
| R-H8 | Local operation identity is unique `(replica_id,counter)`; counter never decreases. | HARD | `counter.log` reservation | restart/counter tests | UNVERIFIED |
| R-H9 | Successful default local commit crosses a durable sync boundary. | HARD | `reserve_counter`, `append_operation` | code path + recovery test; crash-boundary limits documented | PARTIAL |
| R-H10 | Recovery does not decrement counter, reuse IDs, invent ops, or accept partial record as complete. | HARD | frame scanner + counter transition checks | torn-tail/counter/corruption tests | UNVERIFIED |
| R-H11 | Canonical claimed serialization is deterministic. | HARD | ordered maps/sets + serde JSON | deterministic encoding tests | UNVERIFIED |
| R-H12 | Disk and wire versions are explicit and unsupported versions fail closed. | HARD | constants/storage/protocol | unsupported-version tests | UNVERIFIED |
| R-H13 | Declared lengths/counts are bounded before large allocation. | HARD | wire + framed log bounds | oversize parser tests | UNVERIFIED |
| R-H14 | Repeated synchronization with eventual delivery is complete. | HARD | exact dotted summaries + missing-dot transfer | partition/gossip/bootstrap tests | UNVERIFIED |
| R-H15 | Sync of converged replicas creates no new mutations. | HARD | `sync_bidirectional` | repeated-sync test | UNVERIFIED |
| R-H16 | No unsafe tombstone/history GC. | HARD | no destructive GC implemented | code/docs audit | VERIFIED |
| R-H17 | Claimed corruption classes fail closed. | HARD | SHA-256 identity/snapshot; CRC frames | corruption tests | UNVERIFIED |
| R-H18 | Invalid peer batch cannot mutate durable state before full validation. | HARD | `receive_batch` pre-validation | malformed-batch test + reopen check | UNVERIFIED |
| R-SYNC | Hello/summary/missing operations/validation/ack protocol exists independent of transport. | HARD | `WireMessage`, replica primitives | unit/integration tests | UNVERIFIED |
| R-BOOT | New replica bootstrap preserves causal history, not only visible values. | HARD | operation transfer | bootstrap test | UNVERIFIED |
| R-SNAP | Versioned checksummed snapshot reproduces complete operation set. | HARD | `write_snapshot/read_snapshot` | snapshot round-trip test | UNVERIFIED |
| R-GC | Destructive compaction/GC must be safe or explicitly not implemented. | HARD | retained operation history | docs + absence audit | VERIFIED |
| R-REF | Independent in-memory/reference oracle exists. | HARD | `tests/reference_model.rs` | test execution | UNVERIFIED |
| R-EXH | Bounded exhaustive schedule enumeration exists where feasible. | HARD | `tests/exhaustive.rs` | test execution | UNVERIFIED |
| R-FUZZ | Bounded parser fuzz/smoke exists; extended fuzzing documented if not continuous. | SHOULD | `tests/fuzz_smoke.rs` | test execution | UNVERIFIED |
| R-CLI | Required CLI commands exist and support machine-readable status/summary. | HARD | `src/main.rs` | CLI smoke | UNVERIFIED |
| R-XPLAT | Linux/macOS/Windows CI lanes execute required gates. | HARD | `.github/workflows/ci.yml` | GitHub Actions | UNVERIFIED |
| R-PKG | Release build and usable crate/library/CLI. | HARD | Cargo package | CI release build + smoke | UNVERIFIED |
| R-EVID | Requirements, assumptions, claims, failure ledger, local verification, final audit, manifest. | HARD | `evidence/` | manifest + inspection | PARTIAL |

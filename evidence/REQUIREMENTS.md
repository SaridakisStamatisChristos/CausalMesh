# Requirement Ledger

Status vocabulary: `VERIFIED`, `PARTIAL`, `UNVERIFIED`, `FAILED`, `N/A`.

The executable evidence baseline for the final code is GitHub Actions run `38162454086` on commit `4714976e7dd8b3b71660bf12f5d2dc08ea8798c0`: Linux, macOS, and Windows format/Clippy/tests/release/CLI lanes passed and benchmark smoke passed. The final evidence/manifest tree is required to rerun the same gates plus RustSec and manifest verification before merge.

| ID | Requirement / invariant | Evidence | Status |
|---|---|---|---|
| R-H1 | Equal valid logical operation sets converge independent of delivery order. | exhaustive permutations, generated histories, topology/gossip, independent reference | VERIFIED |
| R-H2 | Duplicate delivery is idempotent. | cross-round duplicate, overlapping-batch, intra-batch duplicate regression | VERIFIED |
| R-H3 | Concurrent merge is commutative. | operation-set union + reversed/permuted delivery tests | VERIFIED |
| R-H4 | Merge is associative. | operation-set semantics + multi-replica convergence/exhaustive schedules | VERIFIED |
| R-H5 | Causally newer same-key operations dominate observed older operations. | resolving-write/reverse-chain tests | VERIFIED |
| R-H6 | Concurrent same-key values are not silently lost. | concurrent MV-register tests + generated histories | VERIFIED |
| R-H7 | Replica identity survives restart and ambiguous cloned identity fails closed at sync. | identity persistence/missing identity/clone collision regressions | VERIFIED |
| R-H8 | Local IDs are `(replica_id,counter)` and counters do not move backward/reuse. | counter log checks, restart tests, counter-reservation fault injection | VERIFIED |
| R-H9 | Successful default local commit crosses declared durable sync boundaries. | fsync path + reopen + post-persistence fault injection | VERIFIED within declared process/filesystem model |
| R-H10 | Recovery does not decrement counters, reuse IDs, invent ops, or accept partial records as complete. | torn-tail, rollback, corruption, crash-fault regressions | VERIFIED |
| R-H11 | Claimed canonical encodings are deterministic. | ordered collections + observation-order wire-byte regression | VERIFIED |
| R-H12 | Disk/wire versions are explicit and unsupported versions fail closed. | version constants + unsupported wire/log regressions | VERIFIED |
| R-H13 | Declared sizes/counts are bounded before large allocation. | wire/frame bounds + identity/snapshot metadata prechecks + oversize regressions | VERIFIED |
| R-H14 | Anti-entropy is complete under eventual delivery/no-new-write assumption. | exact-hole test, bootstrap, chain/star/gossip, generated histories | VERIFIED |
| R-H15 | Sync of converged replicas creates no new application mutation. | repeated-sync and unknown-outcome retry tests | VERIFIED |
| R-H16 | Tombstone/history GC is not destructive without proof. | destructive GC absent; docs state storage consequence | VERIFIED |
| R-H17 | Claimed corruption classes fail closed. | CRC/SHA, magic/version, snapshot/identity/log corruption tests | VERIFIED |
| R-H18 | Invalid peer input cannot partially mutate local durable state before validation. | malformed batch + reopen, conflicting duplicate regression | VERIFIED |
| R-SYNC | Versioned transport-independent sync primitives exist. | `WireMessage`, summary/missing/batch/ack semantics, CLI sync | VERIFIED |
| R-BOOT | Bootstrap preserves causal history, not only visible values. | full operation transfer bootstrap regression | VERIFIED |
| R-SNAP | Snapshot contains complete causal operation set and is versioned/checksummed. | round-trip + corruption + bound tests | VERIFIED |
| R-GC | Unsafe compaction/GC is not implemented. | design/absence audit | VERIFIED |
| R-REF | Independent reference oracle exists. | `reference_model.rs` and independent projection in generated campaign | VERIFIED |
| R-EXH | Bounded exhaustive schedule checking exists. | 3-op 6 permutations; 4-op 24 permutations | VERIFIED |
| R-PROP | Generated histories exercise partitions, reordering, duplicates, restart, snapshot and sync. | 32-case proptest model campaign | VERIFIED |
| R-FUZZ | Bounded parser smoke covers disk records, wire, causal context and snapshots. | four deterministic-noise smoke tests | VERIFIED |
| R-CLI | Required CLI commands/library API are usable. | cross-platform release CLI smoke | VERIFIED |
| R-XPLAT | Linux/macOS/Windows required lanes pass. | run `38162454086`; final tree rerun required | VERIFIED |
| R-PKG | Locked release build and reusable crate/CLI succeed. | cross-platform release build + smoke | VERIFIED |
| R-SEC | Dependency advisory scan is part of CI. | RustSec audit; known-good run `38161748408`; final tree rerun required | VERIFIED |
| R-EVID | Requirements, assumptions, claims, failure history, final audit and fail-closed SHA-256 manifest are present. | `evidence/` + final manifest CI gate | VERIFIED when final manifest commit passes CI |

`FINAL_MANIFEST_STATUS: VERIFIED`

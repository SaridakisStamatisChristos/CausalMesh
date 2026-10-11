# Claims Ledger

| Claim | State | Evidence / qualification |
|---|---|---|
| CausalMesh implements `Map<Key, MV-Register<Value>>` with causal deletes. | VERIFIED | compiled/tested on Linux, macOS, Windows |
| Convergence is independent of duplicate/out-of-order delivery for the declared valid-operation/eventual-delivery model. | VERIFIED | exhaustive, property/model, reference and topology campaigns |
| Dotted causal summaries preserve out-of-order holes rather than inventing missing knowledge. | VERIFIED | exact-hole regression and generated histories |
| Same-ID live clones fail closed during synchronization before summary exchange. | VERIFIED | F-005 regression; startup-time clone prevention is NOT CLAIMED |
| Duplicate operations, including identical duplicates within one incoming batch, are idempotent. | VERIFIED | F-006 regression and repeated-sync tests |
| Successful default local writes cross explicit file-sync boundaries before returning. | VERIFIED within declared model | deterministic crash boundaries + reopen tests; physical-media guarantees beyond OS/filesystem contracts are NOT CLAIMED |
| Malformed peer batches are validated before durable mutation. | VERIFIED | malformed-batch failure followed by reopen shows no partial commit |
| Declared oversized identity/snapshot/log/wire inputs are rejected before their large allocations. | VERIFIED | F-008 regressions and parser bounds |
| Linux/macOS/Windows portability for build/tests/CLI. | VERIFIED | final merged `main` run `38162996142` |
| Parser fuzz-smoke covers all four requested surfaces. | VERIFIED | disk log, wire message, causal context, snapshot deterministic-noise tests |
| Dependency advisory scanning is integrated. | VERIFIED | RustSec passed in final merged `main` run `38162996142` |
| Production-ready distributed database. | NOT CLAIMED | see `docs/LIMITATIONS.md` |
| Byzantine or cryptographically authenticated replication. | NOT CLAIMED | outside v0.1 trust model |
| Safe destructive tombstone/history garbage collection. | NOT CLAIMED | intentionally absent |
| Performance superiority. | NOT CLAIMED | smoke timings only; no controlled comparative benchmark |

# Claims Ledger

| Claim | State | Evidence / qualification |
|---|---|---|
| CausalMesh implements `Map<Key, MV-Register<Value>>` with causal deletes. | UNVERIFIED | implementation exists; compile/test evidence pending |
| Convergence is independent of duplicate/out-of-order delivery for valid operations. | UNVERIFIED | property/exhaustive/reference tests authored; execution pending |
| Successful default local writes cross explicit file sync boundaries before returning. | OBSERVED | direct code inspection; true power-loss semantics remain filesystem/platform dependent |
| Malformed peer batches are validated before durable mutation. | UNVERIFIED | implementation and adversarial test authored; execution pending |
| Linux/macOS/Windows portability. | UNVERIFIED | CI matrix authored; runs pending |
| Production-ready distributed database. | NOT CLAIMED | embedded experimental engine; explicit limitations remain |
| Byzantine/cryptographically authenticated replication. | NOT CLAIMED | outside v0.1 trust model |
| Safe tombstone/history garbage collection. | NOT CLAIMED | intentionally not implemented |
| Performance superiority. | NOT CLAIMED | no comparative benchmark evidence |

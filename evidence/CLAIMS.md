# Claims Ledger

R7 requires consequential claims to remain calibrated to observed evidence.

| Claim | State before CI | Required evidence |
|---|---|---|
| v0.1 CRDT algebra satisfies idempotence/commutativity/associativity. | UNVERIFIED | executed algebra tests |
| Concurrent MV-register values and delete/write conflicts are preserved. | UNVERIFIED | executed semantic + independent-oracle tests |
| Arbitrary tested duplicate/out-of-order deliveries converge. | UNVERIFIED | exhaustive + randomized campaigns |
| Default local success is durable across the declared append/counter crash boundary. | UNVERIFIED | executed recovery tests; implementation audit |
| Unsupported/corrupt framed disk formats fail closed. | UNVERIFIED | executed adversarial/recovery tests |
| Malformed incoming batches do not partially mutate durable state. | UNVERIFIED | executed invalid-batch/reopen test |
| Unknown-outcome receive retry is idempotent. | UNVERIFIED | executed lost-ack retry test |
| Linux/macOS/Windows are supported. | UNVERIFIED | green matrix on current commit |
| Package/release build is usable. | UNVERIFIED | release build + installed CLI smoke |
| Dependency set has no known RustSec advisory at verification time. | UNVERIFIED | audit job |
| Destructive tombstone/log GC is safe. | NOT CLAIMED | v0.1 deliberately does not perform it |
| Remote transport is secure/authenticated. | NOT CLAIMED | no remote transport/authentication in v0.1 |
| CausalMesh is Byzantine fault tolerant. | NOT CLAIMED | outside trust model |
| CausalMesh is linearizable. | NOT CLAIMED | product provides causal/eventual convergence semantics |
| Performance is superior to another engine. | NOT CLAIMED | no comparative benchmark campaign |
| Long-term disk usage is bounded. | NOT CLAIMED | history retained until safe retirement/GC exists |

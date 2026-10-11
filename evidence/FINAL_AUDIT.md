# Final Adversarial Audit

This file starts as an execution checklist. A question closes only with observed evidence on the final revision.

| # | Falsification question | Planned evidence | State |
|---:|---|---|---|
| 1 | Can identical valid operation sets end differently? | exhaustive permutations + randomized convergence | VERIFIED |
| 2 | Can duplicate delivery alter result? | repeated receive/sync | VERIFIED |
| 3 | Can operation order alter converged state? | exhaustive permutations | VERIFIED |
| 4 | Can causally old value resurrect? | stale-value-after-delete test | VERIFIED |
| 5 | Can concurrent values be silently collapsed? | concurrent write test + reference oracle | VERIFIED |
| 6 | Can delete erase concurrent write incorrectly? | concurrent delete/write test | VERIFIED |
| 7 | Can delete be forgotten and permit resurrection? | offline delete/stale reconnect | VERIFIED |
| 8 | Can local counter move backward after crash? | counter-repair test | VERIFIED |
| 9 | Can operation ID be reused? | recovered next-counter test | VERIFIED |
| 10 | Can claimed corruption become a different accepted operation? | CRC corruption tests | VERIFIED |
| 11 | Can unsupported format be silently accepted? | wire/disk version tests | VERIFIED |
| 12 | Can malformed peer payload partially mutate state? | invalid-batch + reopen | VERIFIED |
| 13 | Can unknown-outcome retry duplicate logical mutation? | lost-ack retry | VERIFIED |
| 14 | Can crash-after-persist-before-ack break convergence? | lost-ack retry + sync | VERIFIED |
| 15 | Can stale peer undo newer state? | stale reconnect | VERIFIED |
| 16 | Can cloned identity create silent ambiguous IDs? | same-ID sync rejection | VERIFIED |
| 17 | Can snapshot omit declared causal frontier? | snapshot structure/roundtrip | VERIFIED |
| 18 | Can compaction strand a lagging peer? | destructive GC absent; history retained | VERIFIED |
| 19 | Can tombstone GC resurrect data? | destructive tombstone GC absent | VERIFIED |
| 20 | Can map/hash iteration alter canonical encoding? | canonical bytes test | VERIFIED |
| 21 | Can large declared sizes cause unbounded decode allocation? | explicit bounds + adversarial/fuzz | VERIFIED |
| 22 | Can random three-replica partitions fail convergence? | 64-seed campaign | VERIFIED |
| 23 | Can repeated converged sync create operations? | repeated-sync test | VERIFIED |
| 24 | Can recovery itself be interrupted safely? | append-only counter repair/torn-tail paths | PARTIAL |
| 25 | Do docs overclaim consistency/durability/security? | final claims/docs review | VERIFIED |

Overall state after required CI gates: **VERIFIED FOR DECLARED v0.1 TARGET — named limitations remain**.

Question 24 remains **PARTIAL** rather than a universal crash-proof claim: deterministic injection covers the declared append boundaries and recovery repair path, but v0.1 does not claim exhaustive power-loss modeling across every filesystem/hardware failure mode.

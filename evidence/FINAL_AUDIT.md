# Final Adversarial Audit

This file starts as an execution checklist. A question closes only with observed evidence on the final revision.

| # | Falsification question | Planned evidence | State |
|---:|---|---|---|
| 1 | Can identical valid operation sets end differently? | exhaustive permutations + randomized convergence | UNVERIFIED |
| 2 | Can duplicate delivery alter result? | repeated receive/sync | UNVERIFIED |
| 3 | Can operation order alter converged state? | exhaustive permutations | UNVERIFIED |
| 4 | Can causally old value resurrect? | stale-value-after-delete test | UNVERIFIED |
| 5 | Can concurrent values be silently collapsed? | concurrent write test + reference oracle | UNVERIFIED |
| 6 | Can delete erase concurrent write incorrectly? | concurrent delete/write test | UNVERIFIED |
| 7 | Can delete be forgotten and permit resurrection? | offline delete/stale reconnect | UNVERIFIED |
| 8 | Can local counter move backward after crash? | counter-repair test | UNVERIFIED |
| 9 | Can operation ID be reused? | recovered next-counter test | UNVERIFIED |
| 10 | Can claimed corruption become a different accepted operation? | CRC corruption tests | UNVERIFIED |
| 11 | Can unsupported format be silently accepted? | wire/disk version tests | UNVERIFIED |
| 12 | Can malformed peer payload partially mutate state? | invalid-batch + reopen | UNVERIFIED |
| 13 | Can unknown-outcome retry duplicate logical mutation? | lost-ack retry | UNVERIFIED |
| 14 | Can crash-after-persist-before-ack break convergence? | lost-ack retry + sync | UNVERIFIED |
| 15 | Can stale peer undo newer state? | stale reconnect | UNVERIFIED |
| 16 | Can cloned identity create silent ambiguous IDs? | same-ID sync rejection | UNVERIFIED |
| 17 | Can snapshot omit declared causal frontier? | snapshot structure/roundtrip | UNVERIFIED |
| 18 | Can compaction strand a lagging peer? | destructive GC absent; history retained | UNVERIFIED |
| 19 | Can tombstone GC resurrect data? | destructive tombstone GC absent | UNVERIFIED |
| 20 | Can map/hash iteration alter canonical encoding? | canonical bytes test | UNVERIFIED |
| 21 | Can large declared sizes cause unbounded decode allocation? | explicit bounds + adversarial/fuzz | UNVERIFIED |
| 22 | Can random three-replica partitions fail convergence? | 64-seed campaign | UNVERIFIED |
| 23 | Can repeated converged sync create operations? | repeated-sync test | UNVERIFIED |
| 24 | Can recovery itself be interrupted safely? | append-only counter repair/torn-tail paths | PARTIAL |
| 25 | Do docs overclaim consistency/durability/security? | final claims/docs review | UNVERIFIED |

Overall state before execution evidence: **SYNTHESIS IN PROGRESS — verification incomplete**.

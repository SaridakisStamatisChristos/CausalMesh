# Adversarial Assurance Review

This review is calibrated to the v0.1 trust and durability model. It does not turn explicit non-goals into claims. It summarizes durable adversarial conclusions backed by the current regression suite and CI gates.

1. **Can two replicas receiving the same valid operations end differently?** No counterexample is known within the tested model; exhaustive, generated-history and independent-reference checks pass.
2. **Can duplicate delivery alter the result?** No within tested semantics; cross-round, overlapping and same-batch duplicates are idempotent.
3. **Can operation order alter converged state?** No counterexample is known across reverse order, exhaustive permutations and generated histories.
4. **Can a causally old value resurrect?** Regression coverage says no; causal delete/write frontier preserves dominance.
5. **Can concurrent values be silently collapsed?** Regression coverage says no for MV-register semantics.
6. **Can a delete erase a concurrent write incorrectly?** Regression coverage says no; a concurrent put remains visible.
7. **Can a delete be forgotten and allow resurrection?** Destructive history/tombstone GC is absent.
8. **Can a replica counter move backward after crash/reopen?** Counter-log transition and rollback checks fail closed.
9. **Can an operation ID be reused?** Deterministic post-reservation crash coverage consumes the counter; the next ID advances.
10. **Can corruption create a valid-looking different operation?** Declared CRC/SHA corruption classes fail closed. Cryptographic peer authenticity is not claimed.
11. **Can an unsupported format be silently accepted?** Unsupported disk/wire versions fail closed.
12. **Can malformed peer payload partially mutate durable state?** Batch prevalidation plus reopen regression says no.
13. **Can unknown-outcome retry duplicate logical mutation?** Stable operation IDs make retry idempotent.
14. **Can receiver crash-after-persist-before-ack break convergence?** Fault-injection/retry coverage recovers and converges.
15. **Can a stale peer reconnect and undo newer state?** Stale replay does not resurrect dominated data.
16. **Can cloned replica identity cause ambiguous IDs?** A same-ID peer is rejected before summary exchange. Startup-time clone prevention is not claimed.
17. **Can a snapshot omit causality needed for later merge?** Snapshots store the complete operation set; round-trip regression covers this contract.
18. **Can compaction make a lagging peer impossible to sync safely?** Destructive compaction is absent.
19. **Can tombstone GC resurrect data?** Destructive tombstone GC is absent.
20. **Can map/hash iteration order alter canonical encoding?** Ordered collections plus observation-order byte regression cover canonical output.
21. **Can huge declared sizes cause unbounded allocation?** Wire/log bounds and identity/snapshot metadata prechecks close the declared surfaces.
22. **Can generated multi-replica partitions fail to converge?** Seeded gossip and generated histories close with eventual all-pair delivery and independent reference comparison.
23. **Can repeated sync of converged replicas create new operations?** Repeated sync transfers zero new operations.
24. **Can recovery itself be interrupted safely?** Declared counter/local-persist/receive-persist boundaries and torn-tail recovery are tested. Exhaustive OS/kernel/power-cut faulting is **not claimed**.
25. **Does documentation claim stronger consistency/durability/security than demonstrated?** No known overclaim remains in the canonical product surface; trust, platform durability, GC, transport, fuzzing and performance limits are explicit.

## Residual adversarial risk

The main unresolved production risks are peer authentication/encryption, explicit membership/retirement and causal stability, safe destructive compaction/GC, snapshot-accelerated recovery, real production transport, migration tooling, deeper filesystem/power-cut fault injection, and long-running coverage-guided fuzzing.

## Verdict

**Experimental/research-ready: YES. Production-ready: NO.**

The implemented correctness contract is evidence-backed within its declared model; the unresolved items above are material production blockers rather than hidden claims.

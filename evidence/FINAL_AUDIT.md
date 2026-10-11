# Final Adversarial Audit

Initial pre-CI audit. Final answers are evidence-calibrated and must be updated after executable verification.

1. Identical valid operations ending differently: targeted by exhaustive/property/reference tests; UNVERIFIED until CI.
2. Duplicate delivery alters result: targeted; UNVERIFIED.
3. Operation order alters converged state: targeted; UNVERIFIED.
4. Causally old value resurrection: design prevents via frontier dominance; test authored; UNVERIFIED.
5. Concurrent values silently collapsed: test authored; UNVERIFIED.
6. Delete incorrectly erases concurrent write: test authored; UNVERIFIED.
7. Delete forgotten by GC: destructive GC is absent; VERIFIED by design inspection.
8. Counter rollback after normal reopen: counter log transition checks + test; UNVERIFIED execution.
9. Operation ID reuse: reservation-before-append design; crash injection beyond process-level tests remains PARTIAL.
10. Corruption becoming valid different operation: framed CRC/SHA checks target declared classes; cryptographic authenticity NOT CLAIMED.
11. Unsupported format silently accepted: explicit version checks; UNVERIFIED execution.
12. Malformed peer partially mutates local state: batch pre-validation; adversarial test authored; UNVERIFIED.
13. Unknown-outcome retry duplicates logical mutation: stable IDs/idempotence; test coverage via duplicate delivery; UNVERIFIED.
14. Receiver crash-after-persist-before-ack breaks convergence: architecture is idempotent; deterministic power-loss injection not yet fully executed; PARTIAL.
15. Stale peer undoes newer state: stale replay test authored; UNVERIFIED.
16. Cloned identity ambiguity: two clones can allocate conflicting dots; conflicting payload is detected and sync fails closed. Automatic clone prevention NOT CLAIMED.
17. Snapshot omits required causality: snapshot contains complete operation set; round-trip test authored; UNVERIFIED.
18. Compaction strands lagging peer: destructive compaction is not implemented; VERIFIED avoidance.
19. Tombstone GC resurrects data: destructive GC absent; VERIFIED avoidance.
20. Hash/map iteration changes canonical bytes: ordered collections used; explicit deterministic-byte regression still required before VERIFIED.
21. Large declared sizes allocate unboundedly: message/frame bounds exist; tests pending.
22. Random three-replica partitions fail convergence: deterministic three-way plus property campaign authored; broader randomized topology campaign remains PARTIAL.
23. Repeated converged sync creates operations: test authored; UNVERIFIED.
24. Recovery itself interrupted safely: torn-tail handling exists; nested interruption campaign is not exhaustive; PARTIAL.
25. Documentation overclaims: README and limitations explicitly calibrate non-claims; VERIFIED by review.

Pre-CI verdict: **SYNTHESIS COMPLETE — verification incomplete**.

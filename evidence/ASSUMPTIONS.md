# Assumptions Ledger

| A-ID | Assumption | Criticality | Confidence | Affected surfaces | Invalidation trigger |
|---|---|---:|---:|---|---|
| A-001 | Rust is the implementation language. | LOW | High | Entire crate | A platform constraint makes stable Rust unavailable. |
| A-002 | v0.1 implements only `Map<Key, MVRegister<Value>>`; optional CRDTs are excluded. | LOW | High | CRDT/API/docs | User requires an optional type in the v0.1 acceptance scope. |
| A-003 | UUIDv4 textual replica IDs are sufficient identity tokens; peer authentication is out of scope. | MEDIUM | High | Identity/protocol/security | Authenticated remote transport becomes required. |
| A-004 | Exact per-origin counter ranges are the anti-entropy inventory; causal version vectors are not treated as proof that operation bytes are present. | HIGH | High | Protocol/sync | A different completeness proof replaces range inventory. |
| A-005 | A recognizable truncated final log/counter frame is a crash-torn tail and may be ignored; complete bad-checksum frames are corruption and fail closed. | HIGH | High | Recovery/storage | Storage format contract changes. |
| A-006 | No valid-replica membership/retirement oracle exists in v0.1, so destructive history/tombstone GC is unsafe. | HIGH | High | Compaction/tombstones | Explicit retirement epochs or full-observation proof is implemented and verified. |
| A-007 | Directory-to-directory in-process sync is an acceptable v0.1 transport because the machine-readable protocol and semantics are transport-independent. | MEDIUM | High | CLI/protocol | A real remote transport becomes a hard requirement. |
| A-008 | CRC32 is used only for accidental corruption detection, never authenticity or cryptographic integrity. | HIGH | High | Storage/security | Cryptographic adversary resistance becomes required. |
| A-009 | Empty keys are legal; the hard bound is byte length, not non-emptiness. | LOW | Medium | Operation validation | Product semantics require non-empty keys. |
| A-010 | No open-source license is inferred; the public repository grants no license by default. | HIGH | High | Packaging/legal | Owner explicitly selects a license. |

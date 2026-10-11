# Assumptions

| ID | Assumption | Criticality | Confidence | Affects | Invalidation trigger |
|---|---|---:|---:|---|---|
| A-001 | Rust 1.82+ stable is acceptable for v0.1. | LOW | High | build/CI | user requires older MSRV |
| A-002 | Ordinary synchronization is between non-Byzantine peers; malformed input is rejected but authenticated identity is out of scope. | HIGH | High | trust model | authentication becomes mandatory |
| A-003 | Replica membership is open-ended and stale replicas may reappear indefinitely. | HIGH | High | tombstones/GC | explicit retirement/membership epoch is introduced |
| A-004 | Correctness is more important than bounded persistent history in v0.1. | MEDIUM | High | storage | storage ceiling becomes hard requirement |
| A-005 | Keys are non-empty UTF-8 strings <= 4096 bytes; values are arbitrary bytes <= 4 MiB. | MEDIUM | Medium | public API/parser | different bounds/empty keys required |
| A-006 | In-process/file-system sync is sufficient to prove protocol semantics; production network transport is optional. | LOW | High | transport | network server becomes mandatory |
| A-007 | Repository licensing is intentionally not inferred; no open-source license is granted in v0.1. | HIGH | High | packaging/legal | owner selects a license |
| A-008 | Public repository `SaridakisStamatisChristos/CausalMesh` is the intended target because it already exists empty. | MEDIUM | High | GitHub execution | owner specifies another repository |

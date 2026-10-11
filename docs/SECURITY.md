# Security and Trust Model

CausalMesh treats peer payloads and on-disk records as potentially malformed. It validates versions, bounded lengths/counts, keys/values, causal-context canonicality, operation counters, checksums, and duplicate-ID payload equality.

Incoming operation batches are fully validated against themselves and existing state before the first durable write, so a malformed peer batch cannot partially commit because validation failed mid-batch. Identical duplicate operations within one batch are coalesced; the same operation ID with a different payload fails closed.

Peer identity must be validated before summary exchange. A peer presenting the local replica UUID is rejected as a replica-identity collision. This prevents two live copies of a cloned data directory from masking conflicting same-dot operations behind identical causal summaries. It is clone detection at synchronization time, not startup-time clone prevention.

Not claimed in v0.1:

- peer authentication or authorization;
- transport confidentiality/integrity;
- Byzantine consensus or protection from a peer that lies consistently about causal history;
- defense against a malicious local process with filesystem access;
- defense against a compromised OS/kernel;
- cryptographic authenticity from CRC/SHA checksums.

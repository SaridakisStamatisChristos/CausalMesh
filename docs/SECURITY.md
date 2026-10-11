# Security and Trust Model

CausalMesh treats peer payloads and on-disk records as potentially malformed. It validates versions, bounded lengths/counts, keys/values, causal-context canonicality, operation counters, checksums, and duplicate-ID payload equality.

Incoming operation batches are fully validated against themselves and existing state before the first durable write, so a malformed peer batch cannot partially commit because validation failed mid-batch.

Not claimed in v0.1:

- peer authentication or authorization;
- transport confidentiality/integrity;
- Byzantine consensus;
- defense against a malicious local process with filesystem access;
- defense against a compromised OS/kernel;
- cryptographic authenticity from CRC/SHA checksums.

A cloned data directory can create two live processes with the same replica UUID and reservation history. If they independently reuse the same next dot with different payloads, synchronization detects the conflicting duplicate ID and fails closed; automatic clone prevention is not implemented.

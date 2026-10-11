# Security and Trust Model

## Defended boundary

CausalMesh treats peer/disk input as untrusted within declared parser/resource bounds. It validates:

- UUID replica IDs;
- format versions;
- record/message lengths;
- checksums on durable binary frames;
- operation origin-counter transitions;
- causal-context bounds;
- key/value/batch bounds;
- duplicate operation IDs with payload equality;
- same-local-identity unknown operations (clone/ambiguity defense).

Incoming operation batches are fully validated against a preview state before the durable log is changed.

## Not defended

v0.1 does not claim defense against:

- a malicious process with arbitrary write access to the local replica directory;
- a compromised OS/kernel/filesystem;
- forged peer identity over an unauthenticated transport;
- Byzantine peers that create individually valid but semantically dishonest histories;
- confidentiality or traffic analysis.

CRC32 detects accidental corruption, not malicious forgery or authenticity.

## Clone identity

Two copied replica directories initially have the same identity and counter state. Automatic pre-contact detection is impossible without an external uniqueness authority. CausalMesh therefore fails `sync_pair` whenever two peers present the same replica ID and rejects previously unknown same-local-ID operations. Divergent clones cannot silently merge ambiguous operation IDs.

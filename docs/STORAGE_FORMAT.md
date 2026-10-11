# Storage Format

Versions:

```text
STATE_VERSION = 1
LOG_VERSION = 1
WIRE_VERSION = 1
SNAPSHOT_VERSION = 1
```

## identity.json

Immutable JSON envelope containing `{version, replica_id}` plus SHA-256 of the canonical payload. Missing/corrupt identity with existing state fails closed; CausalMesh does not silently mint a replacement identity.

## counter.log

Append-only framed records, each containing one little-endian u64 reservation. Transitions must be exactly `n -> n+1`. A reservation is synced before its corresponding operation is appended. Gaps in committed operations are allowed; counter reuse is not.

## ops.log

Each record is:

```text
4-byte magic | u16 version | u32 payload_len | JSON Operation | u32 CRC32
```

CRC covers version, length, and payload. Declared lengths are bounded before allocation. A recognizable truncated final record is discarded by truncating to the last verified boundary. Bad magic, a checksum failure on a complete record, unsupported version, or impossible local-counter relation fails closed.

## snapshots

Snapshots are immutable JSON envelopes containing the complete operation set and SHA-256 checksum. File names use a checksum prefix. Snapshots do not authorize log truncation in v0.1.

No automatic migration exists yet. Unknown versions fail closed.

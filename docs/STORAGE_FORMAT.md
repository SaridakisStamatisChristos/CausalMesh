# Storage Format

Versions:

```text
STATE_VERSION    = 1
LOG_VERSION      = 1
COUNTER_VERSION  = 1
SNAPSHOT_VERSION = 1
```

Unsupported versions fail closed. No migration from a future/unknown version is attempted.

## `identity.json`

Immutable after creation:

```json
{"state_version":1,"replica_id":"<uuid>"}
```

If identity is missing while nonempty durable history exists, open fails rather than silently generating a new identity.

## `operations.log`

Append-only framed batches:

```text
4 bytes magic "CMOP"
2 bytes version (LE)
4 bytes payload length (LE)
N bytes JSON payload: Vec<Operation>
4 bytes CRC32(payload) (LE)
```

The declared payload length is bounded before allocation/parse. A recognizable truncated final frame is treated as a torn tail and ignored. Bad magic, checksum mismatch, unsupported version, malformed JSON, or structural corruption fail closed.

Each received sync batch is one frame, so invalid input is rejected before append and a torn crash record cannot expose a prefix of a batch as committed logical operations.

## `counter.log`

Append-only local counter evidence:

```text
"CMCT" | version:u16 | counter:u64 | CRC32(counter)
```

The operation log is synced before the counter evidence. Recovery derives the maximum local operation dot and repairs lagging counter evidence by appending the recovered value. Counter evidence ahead of durable local operation history fails closed.

## `snapshot.bin`

A `CMSN` framed/versioned/checksummed deterministic JSON snapshot containing generation, replica identity, local counter, full visible logical frontier/causal clock, and exact received summary.

Snapshots are validated on open but are not yet used to delete authoritative operation history.

# Architecture

## Topology lock

CausalMesh intentionally has five core ownership boundaries:

1. **Types / causality (`types.rs`)** — replica IDs, dots, version vectors, operation validation, hard bounds.
2. **CRDT semantics (`crdt.rs`)** — the MV-register frontier and operation-set merge. It knows nothing about files or transports.
3. **Durability (`codec.rs`, `storage.rs`)** — framed/checksummed formats, identity, operation log, counter evidence, snapshots, torn-tail recovery.
4. **Replication (`protocol.rs`, `sync.rs`)** — exact received-range summaries, machine-readable wire envelopes, missing-operation discovery, pair sync.
5. **Replica API (`replica.rs`)** — coordinates local mutation, receive-before-publish discipline, cloned-identity defense, status/verification.

The CLI is an adapter over the public library and does not own replicated semantics.

## Dependency graph

```text
CLI
 |
 v
Replica API ----> Sync helper
 |   |               |
 |   v               v
 |  CRDT <-------- Protocol
 |   |
 v   v
Storage/Codec <---- Types
```

Forbidden couplings:

- CRDT code may not read/write files.
- Storage code may not invent merge semantics.
- Transport/protocol code may not use wall-clock ordering.
- CLI behavior may not define semantics unavailable through the library API.

## Mutation path

Local mutation:

```text
read causal clock
 -> allocate (local_id, counter+1)
 -> validate/apply to preview state
 -> append checksummed operation batch
 -> fsync operation log
 -> append counter evidence
 -> fsync counter log
 -> publish preview state in memory
 -> report success
```

A failure after a durable write poisons the in-memory handle; reopening reconstructs the truth from disk before further writes.

Incoming batch:

```text
validate entire batch against cloned state
 -> reject without disk mutation on any invalid op
 -> append one checksummed batch frame
 -> fsync
 -> publish preview state
```

This prevents malformed peer data from partially committing. A process crash may leave a recognizable torn final frame, which recovery ignores; a complete frame is replayed atomically as a batch.

## Consistency model

CausalMesh provides causal conflict preservation and eventual convergence under eventual delivery of valid operations. It does not provide linearizability, transactions across keys, consensus, or a global total order.

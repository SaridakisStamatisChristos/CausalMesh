# Architecture

## Components

`causal` defines dots and exact dotted causal contexts. `model` is the deterministic operation-set CRDT and MV-register projection. `storage` owns identity, counter reservation, append-only operation records, recovery, corruption checks, and immutable snapshots. `protocol` defines the versioned machine-readable JSON messages. `replica` binds persistence to replication semantics. `main` is a thin CLI.

The transport-independent sync primitive is `summary -> missing_for -> receive_batch`. The convenience `sync_bidirectional` helper computes both deltas from pre-sync summaries and then applies them.

## Dependency direction

```text
causal <- model <- replica -> storage
   ^        ^         |
   |        |         v
protocol ---+       CLI
```

`storage` serializes validated operations but does not decide CRDT visibility. `protocol` validates message envelopes but does not mutate durable state. `receive_batch` validates the entire batch before the first persistent write.

## Durable layout

```text
<replica>/
  identity.json      immutable, checksummed replica UUID + state version
  counter.log        append-only durable counter reservations
  ops.log            append-only framed operations
  snapshot-*.json    optional immutable content-addressed snapshots
```

Snapshots are not used to discard history in v0.1. This is intentional: without explicit replica retirement or peer-liveness proof, destructive history/tombstone GC could resurrect stale data.

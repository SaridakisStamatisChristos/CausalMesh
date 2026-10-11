# Recovery

On open CausalMesh:

1. creates the directory if absent;
2. creates a new identity only when no durable history exists;
3. validates identity/version;
4. scans operation frames in order;
5. ignores only a recognizable torn final frame;
6. validates every recovered operation and conflicting duplicate ID;
7. scans counter evidence;
8. fails if counter evidence is ahead of durable local operations;
9. appends repaired counter evidence if the operation log is ahead;
10. validates any snapshot checksum/version/identity;
11. exposes the reconstructed replica.

## Crash boundaries

The deterministic storage fault injector supports:

- before append;
- mid-record;
- after append before sync;
- after sync before publication.

Tests exercise torn tails and the critical operation-durable/counter-metadata-missing state. The public `Replica` handle is marked poisoned if a durable append returns an error, forcing reopen before further writes so a process cannot append behind an unresolved torn tail.

## Durability statement

In default mode a local mutation reports success only after both its operation record and monotonic counter evidence have been `sync_all()`'d. On Unix, identity/snapshot creation/replacement also attempts directory sync. Filesystem/hardware behavior below the OS durability contract is outside the claim.

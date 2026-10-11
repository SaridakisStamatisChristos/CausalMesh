# Tombstones

Deletion is a causally tracked MV-register event, not absence.

A tombstone stays in the visible frontier until a causally later operation supersedes it. Historical delete operations remain in the operation log indefinitely in v0.1.

There is no age-based garbage collection. Safe tombstone/history GC would require either:

1. proof that every replica that may ever return has observed the delete; or
2. an explicit membership and replica-retirement rule that invalidates sufficiently stale replicas.

Neither mechanism exists in v0.1, so destructive tombstone GC is deliberately NOT CLAIMED.

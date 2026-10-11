# CRDT Semantics

## Data model

Version 0.1 exposes `Map<Key, MV-Register<Value>>`.

Every mutation is an immutable `Operation { id, context, key, kind }`, where `kind` is `put(value)` or `delete`. The operation set is a grow-only set keyed by globally scoped dot `(replica_id, counter)`. A repeated identical operation is a no-op; a repeated dot with a different payload is corruption/hostile input and fails closed.

## Visibility

For one key, operation `x` is dominated when another operation `y` on the same key has `x.id` in `y.context`. The visible register frontier is the set of non-dominated operations. Visible values are the `put` values on that frontier; frontier deletes contribute no visible value.

This makes the result a pure function of the logical operation set, not delivery order.

## Write

A local write snapshots the replica's current causal context, durably reserves a new local dot, then appends the operation. Because the context contains all events observed by the replica, the new write dominates prior observed mutations for that key while remaining concurrent with unseen remote mutations.

## Delete

Deletion is a causal operation. An old put cannot resurrect after its delete is known because the delete context dominates that put. A delete concurrent with a put does not erase the concurrent put. A later mutation that has observed both can dominate both.

## Algebra

Operation-set merge is set union subject to same-dot payload equality. Union is associative, commutative, and idempotent. Register projection is deterministic from the merged operation set, so replicas with the same valid operations converge to the same logical state.

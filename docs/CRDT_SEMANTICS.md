# CRDT Semantics

## Data type

v0.1 implements:

```text
Map<Key, MVRegister<Value>>
```

Each mutation is an immutable operation with a unique dot `(replica_id, counter)` and the version-vector context visible when the mutation was created.

## Frontier rule

For one key, operation `x` is not part of the visible MV-register frontier when another known operation `y` for the same key has `y.context >= x.dot`. In other words, `y` causally observed and superseded `x`.

Two operations are concurrent when neither context dominates the other's dot. Concurrent frontier entries are preserved.

The production engine uses an incremental frontier. `tests/reference_model.rs` independently recomputes the frontier by pairwise dominance over the complete operation set.

## Put

A local put captures the full causal clock, allocates the next local dot, removes frontier entries observed by that context, preserves concurrent entries, and inserts the new value.

## Delete

Delete is a normal causally tracked operation whose frontier value is a tombstone (`None`), not absence.

A causally later delete dominates an observed value. If that older value is delivered after the delete, the delete's causal context prevents resurrection. A concurrent delete and put both remain in the frontier; `get` returns surviving value entries while `entries` exposes the full conflict including the tombstone.

An explicit later write that has observed both sides dominates both and resolves the conflict.

## Algebra

The replicated state is the deterministic result of the set of valid immutable operations. Set union plus frontier recomputation is associative, commutative, and idempotent. The implementation tests these laws directly and also tests all 24 delivery permutations of a four-operation bounded universe.

## Read ordering

Frontier entries are sorted by operation dot. Reads therefore return concurrent values in stable dot order. Equivalent operation sets produce byte-identical `canonical_state_bytes()` output.

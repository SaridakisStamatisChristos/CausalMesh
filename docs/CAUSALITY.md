# Causality

## Version vectors

A causal context is a `BTreeMap<ReplicaId, u64>`. Missing entries mean counter `0`.

- `observe(dot)` takes the maximum counter for the dot's replica.
- `join(a,b)` takes component-wise maxima.
- a context dominates a dot when its component is `>= dot.counter`.
- two contexts are concurrent when neither component-wise dominates the other.

Wall-clock time is never used for causality.

## Operation-origin invariant

For an operation `(R,n)`, its context entry for `R` must be exactly `n-1`. Counters are 1-based. This detects impossible origin transitions and makes local history monotonic.

A receiver can accept an operation whose earlier origin operations are not physically present: the operation's context still carries causal knowledge. This is why CausalMesh keeps **causal knowledge** separate from **received operation inventory**.

## Received ranges

Anti-entropy summaries encode exact contiguous counter ranges physically held per origin replica. Example:

```text
R1: [1..3, 5..7]
```

Counter `4` is still missing even though the receiver causally knows about or physically has later operations. This prevents the classic error of using a max-counter version vector as an inventory summary under out-of-order delivery.

## Replica removal

v0.1 has no membership/retirement protocol. Consequently, causal entries and tombstone-relevant history are not destructively garbage-collected merely because they are old.

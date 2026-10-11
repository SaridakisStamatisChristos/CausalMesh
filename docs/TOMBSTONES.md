# Tombstones and History GC

Deletes are ordinary causal operations and remain in the operation history.

Version 0.1 intentionally implements **no destructive tombstone/history garbage collection**. Age is not evidence that every replica has observed a delete. Without an explicit membership/retirement protocol, deleting old causal history could let a stale replica reintroduce data that should remain dominated.

Storage therefore grows with accepted operations. A future safe GC design needs a membership epoch or equivalent proof that every replica allowed to rejoin has crossed a causal stability frontier.

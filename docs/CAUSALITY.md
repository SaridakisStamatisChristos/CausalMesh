# Causality

## Dot

A dot is `(replica UUID, positive u64 counter)`.

## Dotted causal context

A context contains:

- `base[replica] = n`: every dot `1..=n` is known;
- `cloud`: sparse dots strictly above the contiguous base.

This preserves holes. If counter 7 arrives before 6, the context can represent `base=5, cloud={7}` and will still request 6 during anti-entropy. When 6 arrives, normalization advances the base to 7 and removes the cloud entry.

## Rules

`contains(dot)` is true if the dot is at/below the base or is present in the cloud. `observe(dot)` extends the base only across contiguous knowledge and otherwise inserts a cloud dot. `merge(context)` takes per-replica base maxima, unions clouds, removes dots subsumed by the base, then absorbs contiguous cloud entries.

Canonical contexts reject zero counters, cloud dots at/below `base+1`, and contexts above the declared entry bound.

Wall-clock time is never used for causality.

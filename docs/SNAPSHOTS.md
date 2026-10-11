# Snapshots and Compaction

`causalmesh snapshot <DIR>` writes a checksummed/versioned snapshot containing:

- snapshot generation;
- replica identity;
- local counter;
- logical causal clock and MV-register frontier;
- exact received-operation summary.

`causalmesh compact <DIR>` currently creates the same safe snapshot **without deleting operation history**.

This is intentional. v0.1 has no membership/retirement proof that every valid replica has observed the history a compactor might remove. Therefore destructive log truncation is not implemented and no bounded-storage claim is made.

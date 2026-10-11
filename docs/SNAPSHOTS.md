# Snapshots

`causalmesh snapshot <DIR>` writes a complete, versioned, checksummed operation-set snapshot. The payload is serialized deterministically from the dot-ordered operation map. The final file name is content addressed by SHA-256 prefix.

Creation writes a unique temporary file, syncs it, then renames it to a new immutable content-addressed name. Existing matching snapshots are verified and reused.

Version 0.1 does not recover from snapshots and does not delete operation-log history after snapshot creation. This sacrifices space efficiency to avoid unsafe compaction while arbitrary stale replicas may return.

# Limitations

v0.1 intentionally leaves the following outside its verified claim surface:

- destructive operation-log compaction;
- tombstone/history garbage collection;
- explicit replica retirement/membership epochs;
- remote network transport;
- peer authentication/encryption;
- Byzantine fault tolerance;
- cross-key transactions;
- linearizable/serializable consistency;
- bounded long-term disk growth;
- background synchronization;
- cryptographic content integrity;
- performance superiority.

Snapshots are integrity-checked and contain the causal frontier, but restart currently reconstructs authoritative state from the operation log. This preserves correctness while safe history deletion remains unresolved.

Filesystem durability is limited by the guarantees provided by `fsync`/`sync_all`, the OS, filesystem, and storage hardware. Unix directory sync is attempted for identity/snapshot metadata; non-Unix metadata durability is documented but not claimed beyond standard-library behavior.

# Limitations

- No authenticated/encrypted transport; wire semantics are defined independently of transport.
- No Byzantine fault tolerance or consensus.
- No automatic cloned-replica prevention; conflicting duplicate dots fail closed when detected.
- No destructive log compaction or tombstone/history GC.
- Snapshots are verified artifacts but are not yet a recovery accelerator.
- No explicit replica-retirement/membership epoch protocol.
- Filesystem durability differs by OS. Rust exposes directory fsync on Unix through normal file APIs; the standard library does not provide equivalent portable directory durability semantics on every target, so Windows rename-directory metadata durability is not claimed at the same strength.
- No migration path beyond fail-closed version rejection for v0.1 formats.
- No performance superiority claim.

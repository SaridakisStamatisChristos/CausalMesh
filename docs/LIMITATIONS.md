# Limitations

- No authenticated/encrypted transport; wire semantics are defined independently of transport.
- No Byzantine fault tolerance or consensus.
- A peer presenting the local replica UUID is rejected before summary exchange, which detects a live cloned identity at synchronization time. There is no startup-time lease, host binding, or automatic clone-prevention service.
- No destructive log compaction or tombstone/history GC.
- Snapshots are verified artifacts but are not yet used as a recovery accelerator.
- No explicit replica-retirement/membership epoch protocol.
- Filesystem durability differs by OS. Unix directory sync is used where available; equivalent directory-metadata durability is not claimed uniformly on every platform.
- Deterministic fault injection covers declared logical durability boundaries; exhaustive kernel/filesystem power-cut testing is not claimed.
- No migration path beyond fail-closed version rejection for v0.1 formats.
- No built-in production network server/transport.
- No performance superiority claim.
- Bounded deterministic fuzz-smoke is part of CI; long-running coverage-guided fuzzing is not yet a continuous gate.

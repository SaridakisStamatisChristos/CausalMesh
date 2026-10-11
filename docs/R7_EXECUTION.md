# UCS-Pro R7 Execution Record

## CONTRACT

The authoritative user brief defines H1-H18 plus required CLI, library, anti-entropy, crash/recovery, adversarial, CI, documentation, packaging, and evidence gates. `evidence/REQUIREMENTS.md` is the traceability ledger.

## PLAN / SPEC

- Rust core.
- `Map<Key, MVRegister<Value>>` only for v0.1.
- Version-vector causal contexts.
- Exact per-replica received ranges for anti-entropy inventory.
- Append-only framed/checksummed operation and counter logs.
- Immutable identity file.
- Snapshot-without-destructive-GC until replica retirement can prove safety.
- In-process sync transport plus standalone wire encoding.

## SCAFFOLD / TOPOLOGY LOCK

Batches:

- B1: package + types/errors/causality;
- B2: CRDT + independent model tests;
- B3: storage/codec/recovery;
- B4: protocol/sync/replica API/CLI;
- B5: integration/property/exhaustive/adversarial/fuzz tests;
- B6: docs/evidence/CI/packaging/manifest.

The dependency boundaries are documented in `ARCHITECTURE.md`.

## GENERATE / INTEGRATE

Every public path is connected through the Rust crate and CLI. The required commands use production library paths; test-only semantics do not replace runtime behavior.

## VERIFY / FALSIFY / REPAIR

Observed execution evidence belongs in `evidence/LOCAL_VERIFICATION.md`; defects and their earliest invalidated phase belong in `evidence/FAILURE_REPAIR_LEDGER.md`. A failing required CI lane blocks completion.

## PACKAGE / MANIFEST

The root Cargo package builds both library and `causalmesh` binary. `scripts/verify_manifest.py` checks SHA-256 entries for repository files while excluding the manifest itself, `.git`, and build output.

# Failure / Repair Ledger

Failures are retained. Each entry records the earliest invalidated phase and the repair/reverification path.

## F-001 — Local Rust toolchain unavailable in campaign runtime
- phase: VERIFY
- reproducer: `rustc --version`
- observed: command not found
- class: environmental verification gap
- repair: use GitHub Actions as executable Rust oracle; never claim local Rust execution
- status: CLOSED BY EXTERNAL CI EVIDENCE (environment itself unchanged)

## F-002 — Generated Rust failed immutable formatting gate
- phase: GENERATE → VERIFY
- reproducer: run `38160915150`, `cargo fmt --check`
- observed: rustfmt diff
- repair: capture/apply canonical runner-produced rustfmt and commit `Cargo.lock`
- regression/reverification: later strict format gates pass on all OSes
- status: CLOSED

## F-003 — Evidence manifest intentionally absent while tree was changing
- phase: MANIFEST / EVIDENCE
- reproducer: evidence job on intermediate runs
- observed: `evidence/MANIFEST.sha256` missing
- repair: defer manifest until code/docs/evidence freeze; generate from exact final repository bytes and verify fail-closed
- reverification: final merged `main` run `38162996142` passed the committed SHA-256 manifest gate
- status: CLOSED

## F-004 — GitHub dependency-review action unsupported because Dependency Graph disabled
- phase: VERIFY / SECURITY
- reproducer: run `38161547921`
- observed: dependency review reported repository support disabled; no vulnerability finding
- repair: replace repository-setting-dependent action with RustSec `Cargo.lock` audit
- reverification: RustSec gate succeeded on run `38161748408`
- status: CLOSED

## F-005 — Cloned replica identity could be masked by dot-only summaries
- phase: FALSIFY
- reproducer: two cloned directories independently allocate the same `(replica_id,counter)` with different payloads, then sync
- observed: equal dot summaries caused neither conflicting operation to transfer; sync could appear successful
- violated invariant: stable/non-ambiguous replica identity / hostile-peer fail-closed intent
- root cause: peer identity was not rejected before summary exchange
- earliest invalidated phase: SPEC/INTEGRATE handshake semantics
- repair: `Replica::validate_peer_identity`; reject peer UUID equal to local UUID before causal summaries; `ReplicaIdentityCollision`
- regression: `conflicting_cloned_replica_identity_fails_closed`
- reverification: cross-platform runs including `38161748408`, `38162454086`, and final merged `main` run `38162996142`
- status: CLOSED

## F-006 — Identical duplicate operations inside one batch were staged twice
- phase: FALSIFY
- reproducer: one incoming batch containing the same valid operation multiple times
- observed: identical IDs passed conflict check but each copy was added to the persistence staging list
- violated invariant: sync idempotence / duplicate delivery safety
- root cause: validation map detected equality but did not coalesce it
- earliest invalidated phase: INTEGRATE receive-batch logic
- repair: continue past already-seen identical batch ID; conflicting duplicate still fails closed
- regression: `identical_duplicates_inside_one_batch_are_coalesced`, including reopen/log reconstruction
- status: CLOSED

## F-007 — Generated model-based campaign failed rustfmt gate
- phase: GENERATE → VERIFY
- reproducer: run `38162056360`
- observed: new test file formatting only; semantic tests had not run
- repair: apply exact rustfmt diff only, commit `46b6597a...`
- reverification: subsequent model campaign passed
- status: CLOSED

## F-008 — Snapshot and identity size checks occurred after `fs::read`
- phase: FALSIFY / STATIC AUDIT
- reproducer: oversized sparse identity/snapshot files
- observed: size was checked only after full file allocation/read
- violated invariant: H13 bounded parsing/allocation
- root cause: post-read rather than metadata precheck
- earliest invalidated phase: GENERATE storage parser
- repair: metadata length check before `fs::read`; sparse-file regressions; commit `1514b080...`
- reverification: incorporated in final cross-platform test run `38162454086` and final merged `main` run `38162996142`
- status: CLOSED

## F-009 — Initial fuzz-smoke covered only wire parser
- phase: VERIFY / EVIDENCE COVERAGE
- reproducer: inspect `tests/fuzz_smoke.rs`
- observed: disk record, causal-context and snapshot decoder surfaces requested by contract were not smoke-fuzzed
- root cause: incomplete verification surface, not known product failure
- repair: deterministic bounded noise tests for wire, causal context, snapshot and operation log; commit `4714976e...`
- reverification: final cross-platform run `38162454086` and final merged `main` run `38162996142`
- status: CLOSED

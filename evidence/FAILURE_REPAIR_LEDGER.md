# Failure / Repair Ledger

Failures are never erased. New entries must include reproducer, violated requirement, root cause, earliest invalidated phase, repair, regression test, reverification, and status.

## F-001 — Local Rust toolchain unavailable in campaign runtime

- lifecycle phase: VERIFY
- reproducer: `rustc --version`
- observed behavior: command not found
- violated invariant: none in product; verification capability gap
- root cause: campaign container does not include Rust
- earliest invalidated phase: none
- repair: use GitHub Actions matrix as execution oracle; do not label compile/tests verified until observed
- regression test: N/A
- reverification: pending CI
- final status: OPEN / ENVIRONMENTAL

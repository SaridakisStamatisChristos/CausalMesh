# Failure / Repair Ledger

Failures are retained; entries are never deleted when repaired.

## F-001 — Local Rust execution unavailable

- **Lifecycle phase:** VERIFY
- **Reproducer:** `rustc --version`
- **Observed behavior:** orchestration container returned `rustc: command not found`.
- **Violated requirement/claim:** no product invariant; local execution evidence unavailable.
- **Root cause:** the current orchestration runtime does not include the Rust toolchain.
- **Earliest invalidated phase:** VERIFY only.
- **Repair:** use repository-native GitHub Actions as the execution oracle; never convert missing local tools into PASS.
- **Regression/evidence path:** CI runs format, clippy, tests, cross-platform matrix, release build, CLI smoke, manifest verification, and dependency audit.
- **Status:** OPEN until CI evidence is observed.

## F-002 — Bootstrap push denied on workflow permission boundary

- **Lifecycle phase:** INTEGRATE / CI BOUNDARY
- **Reproducer:** GitHub Actions run 38161988966, `git push origin HEAD:r7-zero-to-hero` after synthesis.
- **Observed behavior:** payload expansion, ZIP CRC validation, Cargo.lock generation, and manifest generation succeeded; GitHub rejected the push because the Actions token may not create or update `.github/workflows/ci.yml` without workflow permission.
- **Violated requirement/claim:** repository integration was incomplete; no product correctness invariant was falsified.
- **Root cause:** GitHub App token capability boundary for workflow-file mutation.
- **Earliest invalidated phase:** INTEGRATE.
- **Repair:** keep workflow files outside the bot commit; synthesize and push all non-workflow content, then install/remove workflows through the authorized GitHub connector.
- **Regression/evidence path:** rerun bootstrap and require successful synthesis commit; then execute CI from a connector-installed workflow.
- **Status:** REPAIRED when the synthesis commit lands and CI is connector-installed.

## F-003 — Repair workflow rejected by YAML parser

- **Lifecycle phase:** INTEGRATE / CI BOUNDARY
- **Reproducer:** GitHub Actions run 38162080982.
- **Observed behavior:** run failed before job creation because a shell here-document embedded in the YAML block scalar was not indented as YAML content.
- **Violated requirement/claim:** integration automation was syntactically invalid; no product correctness invariant was exercised.
- **Root cause:** workflow repair text escaped the YAML block-scalar indentation.
- **Earliest invalidated phase:** INTEGRATE.
- **Repair:** indent the entire here-document inside the `run: |` scalar so YAML strips indentation before shell execution.
- **Regression/evidence path:** next bootstrap run must create and execute the `synthesize` job.
- **Status:** REPAIRED when the corrected bootstrap job starts successfully.

## F-004 — First authoritative CI trigger races the manifest transition

- **Lifecycle phase:** MANIFEST / EVIDENCE
- **Reproducer:** first `.ci-trigger` push after connector installation of the final CI workflow.
- **Observed behavior:** that event necessarily starts from the pre-CI manifest while a dedicated reconciliation workflow creates the first manifest covering the final workflow set; its manifest gate is expected to fail closed.
- **Violated requirement/claim:** none silently accepted; the manifest gate intentionally rejects the transitional event SHA.
- **Root cause:** GitHub contents/workflow mutations are separate commits, while the manifest hashes authoritative repository content.
- **Earliest invalidated phase:** MANIFEST / EVIDENCE only.
- **Repair:** reconcile the manifest with the final CI workflow, remove this temporary reconciler, then trigger CI again from an event SHA whose manifest is already authoritative.
- **Regression/evidence path:** the next CI run must pass `scripts/verify_manifest.py` before evidence closure.
- **Status:** REPAIRED when the clean post-reconciliation CI run passes the manifest gate.

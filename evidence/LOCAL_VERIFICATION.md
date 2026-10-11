# Verification Record

## Local campaign environment

The orchestration container did not provide a Rust toolchain:

```text
rustc --version
=> command not found
```

No local Rust execution is claimed.

## GitHub Actions executable evidence

Final-code baseline: run `38162454086`, commit `4714976e7dd8b3b71660bf12f5d2dc08ea8798c0`.

Observed:

- Ubuntu: format PASS; Clippy PASS; tests PASS; release build PASS; CLI smoke PASS.
- macOS: format PASS; Clippy PASS; tests PASS; release build PASS; CLI smoke PASS.
- Windows: format PASS; Clippy PASS; tests PASS; release build PASS; CLI smoke PASS.
- benchmark-smoke: PASS; artifact `11710357713`.
- RustSec advisory audit is a required CI gate; known-good run `38161748408` passed.
- intermediate evidence-manifest jobs failed exactly as intended while the manifest was deferred until the repository tree stopped changing.

The final evidence/manifest commit must rerun all required jobs. Completion is claimed only after `python scripts/verify_manifest.py` succeeds against `evidence/MANIFEST.sha256` on that final tree.

`FINAL_MANIFEST_STATUS: VERIFIED`

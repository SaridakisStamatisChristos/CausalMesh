# Verification Record

## Orchestration environment

Observed before repository CI:

```text
rustc --version
=> rustc: command not found
```

Therefore no local Rust compile/test result is claimed from the orchestration container.

## Required repository commands

CI is configured to execute the critical user/release paths:

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets
cargo build --release
cargo install --path . --root <temp-root>
python3 scripts/verify_manifest.py
```

Cross-platform test/build lanes are configured for Linux, macOS, and Windows. Results will be recorded here only after they are observed for the final revision.

## Observed GitHub Actions closure

- Verified source SHA: `3a8af213c6d5852e228763d817108fb47accdd96`
- Workflow run: https://github.com/SaridakisStamatisChristos/CausalMesh/actions/runs/38162727329
- Required predecessor jobs all succeeded before evidence closure ran: Linux quality (fmt/clippy/tests/release/install smoke), Linux/macOS/Windows matrix, manifest check, RustSec audit.
- The evidence-closure commit changes evidence/manifest only; executable code is the verified source SHA above.

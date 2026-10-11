# CI Evidence

- Repository: `SaridakisStamatisChristos/CausalMesh`
- Verified source SHA: `3a8af213c6d5852e228763d817108fb47accdd96`
- Workflow run ID: `38162727329`
- Workflow run URL: https://github.com/SaridakisStamatisChristos/CausalMesh/actions/runs/38162727329

The `evidence-close` job is dependency-gated on successful completion of:

1. `quality-linux`: lockfile presence, fmt, clippy with warnings denied, all-target tests, release build, installed CLI smoke;
2. `test-ubuntu-latest`;
3. `test-macos-latest`;
4. `test-windows-latest`;
5. `evidence-manifest`;
6. `rustsec-audit`.

This file is generated only after all of those required jobs succeed.

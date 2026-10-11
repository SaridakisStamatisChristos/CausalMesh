# Local Verification

Campaign environment observation:

```text
rustc --version
=> command not found
```

Therefore no local Rust compilation/test result is claimed. Repository generation and static file inspection were performed, while executable Rust evidence is delegated to GitHub Actions. This document must be updated only with actually observed results.

Manifest verification is executable with:

```bash
python scripts/verify_manifest.py
```

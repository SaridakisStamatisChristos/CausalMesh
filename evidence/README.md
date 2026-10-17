# Evidence

The `evidence/` directory is part of CausalMesh's assurance model. It is not a build-log dump.

## Canonical evidence

The active product/research evidence is:

- `ASSUMPTIONS.md` — explicit model boundaries and environmental assumptions.
- `CLAIMS.md` — supported claims and explicit non-claims.
- `REQUIREMENTS.md` — current invariants and their evidence status.
- `ASSURANCE_REVIEW.md` — durable adversarial review of the implemented contract.
- `MANIFEST.sha256` — SHA-256 integrity manifest for repository files other than the manifest itself.

These files describe the current tree and should be maintained when behavior, claims, verification, or repository contents change.

## Historical implementation evidence

`history/` contains one-off implementation-campaign records preserved for provenance. Those files can explain how defects were found or how a particular development campaign was verified, but they are **not authoritative descriptions of the current product**.

Historical run IDs, branch names, repair chronology, and campaign labels belong there rather than in canonical claims or requirements.

## Integrity

Generate a proposed manifest with:

```bash
python scripts/generate_manifest.py
```

Verify the committed manifest with:

```bash
python scripts/verify_manifest.py
```

CI runs manifest verification together with the Rust, benchmark-smoke, and security gates.

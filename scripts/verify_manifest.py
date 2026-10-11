#!/usr/bin/env python3
from __future__ import annotations

import hashlib
from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[1]
MANIFEST = ROOT / "evidence" / "MANIFEST.sha256"
EXCLUDED_DIRS = {".git", "target", ".ci-install", "__pycache__"}
EXCLUDED_FILES = {Path("evidence/MANIFEST.sha256"), Path(".ci-trigger")}


def repository_files() -> list[Path]:
    files: list[Path] = []
    for path in ROOT.rglob("*"):
        if not path.is_file():
            continue
        rel = path.relative_to(ROOT)
        if any(part in EXCLUDED_DIRS for part in rel.parts):
            continue
        if rel in EXCLUDED_FILES:
            continue
        files.append(rel)
    return sorted(files, key=lambda p: p.as_posix())


def sha256(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as f:
        for chunk in iter(lambda: f.read(1024 * 1024), b""):
            h.update(chunk)
    return h.hexdigest()


def parse_manifest() -> dict[Path, str]:
    entries: dict[Path, str] = {}
    for number, raw in enumerate(MANIFEST.read_text(encoding="utf-8").splitlines(), 1):
        line = raw.strip()
        if not line or line.startswith("#"):
            continue
        try:
            digest, name = line.split("  ", 1)
        except ValueError:
            raise SystemExit(f"invalid manifest line {number}: {raw!r}")
        if len(digest) != 64 or any(c not in "0123456789abcdef" for c in digest):
            raise SystemExit(f"invalid SHA-256 at line {number}")
        rel = Path(name)
        if rel.is_absolute() or ".." in rel.parts:
            raise SystemExit(f"unsafe manifest path at line {number}: {name}")
        if rel in entries:
            raise SystemExit(f"duplicate manifest entry: {name}")
        entries[rel] = digest
    return entries


def main() -> int:
    if not MANIFEST.exists():
        print("manifest missing", file=sys.stderr)
        return 2
    expected = parse_manifest()
    actual_files = repository_files()
    actual_set = set(actual_files)
    expected_set = set(expected)
    missing = sorted(expected_set - actual_set, key=lambda p: p.as_posix())
    unmanifested = sorted(actual_set - expected_set, key=lambda p: p.as_posix())
    bad = []
    for rel in sorted(expected_set & actual_set, key=lambda p: p.as_posix()):
        observed = sha256(ROOT / rel)
        if observed != expected[rel]:
            bad.append((rel, expected[rel], observed))
    if missing or unmanifested or bad:
        for rel in missing:
            print(f"missing: {rel.as_posix()}", file=sys.stderr)
        for rel in unmanifested:
            print(f"unmanifested: {rel.as_posix()}", file=sys.stderr)
        for rel, exp, obs in bad:
            print(f"hash mismatch: {rel.as_posix()} expected={exp} observed={obs}", file=sys.stderr)
        return 1
    print(f"manifest verified: {len(expected)} files")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

#!/usr/bin/env python3
from __future__ import annotations
import hashlib
from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[1]
MANIFEST = ROOT / "evidence" / "MANIFEST.sha256"


def main() -> int:
    failures = []
    for raw in MANIFEST.read_text(encoding="utf-8").splitlines():
        line = raw.strip()
        if not line or line.startswith("#"):
            continue
        expected, rel = line.split("  ", 1)
        path = ROOT / rel
        if not path.is_file():
            failures.append(f"missing: {rel}")
            continue
        actual = hashlib.sha256(path.read_bytes()).hexdigest()
        if actual != expected:
            failures.append(f"hash mismatch: {rel}")
    if failures:
        print("manifest verification FAILED", file=sys.stderr)
        for failure in failures:
            print(f"- {failure}", file=sys.stderr)
        return 1
    print("manifest verification PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

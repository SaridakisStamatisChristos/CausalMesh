#!/usr/bin/env python3
from __future__ import annotations
import hashlib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
MANIFEST = ROOT / "evidence" / "MANIFEST.sha256"

for path in sorted(ROOT.rglob("*")):
    if not path.is_file() or ".git" in path.parts or path == MANIFEST:
        continue
    rel = path.relative_to(ROOT).as_posix()
    print(f"{hashlib.sha256(path.read_bytes()).hexdigest()}  {rel}")

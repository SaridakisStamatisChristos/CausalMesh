#!/usr/bin/env python3
from __future__ import annotations
import hashlib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
MANIFEST = ROOT / "evidence" / "MANIFEST.sha256"
EXCLUDED_DIRS = {".git", "target", ".ci-install", "__pycache__"}
EXCLUDED_FILES = {Path("evidence/MANIFEST.sha256"), Path(".ci-trigger")}

files=[]
for path in ROOT.rglob("*"):
    if not path.is_file():
        continue
    rel=path.relative_to(ROOT)
    if any(part in EXCLUDED_DIRS for part in rel.parts) or rel in EXCLUDED_FILES:
        continue
    files.append(rel)
files.sort(key=lambda p:p.as_posix())
lines=["# SHA-256 manifest. This file intentionally excludes itself and .ci-trigger."]
for rel in files:
    h=hashlib.sha256((ROOT/rel).read_bytes()).hexdigest()
    lines.append(f"{h}  {rel.as_posix()}")
MANIFEST.write_text("\n".join(lines)+"\n", encoding="utf-8")
print(f"wrote {len(files)} manifest entries")

#!/usr/bin/env python3
from __future__ import annotations

import json
import os
import platform
from pathlib import Path
import shutil
import subprocess
import tempfile
import time

ROOT = Path(__file__).resolve().parents[1]
BIN = ROOT / "target" / "release" / ("causalmesh.exe" if os.name == "nt" else "causalmesh")


def run(*args: object) -> str:
    result = subprocess.run(
        [str(BIN), *(str(arg) for arg in args)],
        cwd=ROOT,
        check=True,
        capture_output=True,
        text=True,
    )
    return result.stdout.strip()


def timed(fn):
    start = time.perf_counter()
    value = fn()
    return time.perf_counter() - start, value


def main() -> int:
    root = Path(tempfile.mkdtemp(prefix="causalmesh-bench-"))
    try:
        a = root / "a"
        b = root / "b"
        run("init", a)
        run("init", b)

        puts = 50
        put_seconds, _ = timed(lambda: [run("put", a, f"put:{i}", f"v{i}") for i in range(puts)])
        gets = 50
        get_seconds, _ = timed(lambda: [run("get", a, f"put:{i % puts}") for i in range(gets)])
        recovery_seconds, _ = timed(lambda: run("verify", a))

        small = 10
        for i in range(small):
            run("put", a, f"small:{i}", "x")
        small_delta_seconds, _ = timed(lambda: run("sync", a, b))
        zero_delta_seconds, _ = timed(lambda: run("sync", a, b))

        large_src = root / "large-src"
        large_dst = root / "large-dst"
        run("init", large_src)
        run("init", large_dst)
        large = 100
        for i in range(large):
            run("put", large_src, f"large:{i}", "payload")
        large_delta_seconds, _ = timed(lambda: run("sync", large_src, large_dst))

        two_a = root / "two-a"
        two_b = root / "two-b"
        run("init", two_a)
        run("init", two_b)
        run("put", two_a, "conflict", "a")
        run("put", two_b, "conflict", "b")
        two_replica_seconds, _ = timed(lambda: run("sync", two_a, two_b))

        gossip_dirs = [root / f"gossip-{i}" for i in range(10)]
        for i, directory in enumerate(gossip_dirs):
            run("init", directory)
            run("put", directory, f"g:{i}", str(i))
        gossip_start = time.perf_counter()
        for _ in range(3):
            for i in range(9):
                run("sync", gossip_dirs[i], gossip_dirs[i + 1])
            for i in range(8, -1, -1):
                run("sync", gossip_dirs[i], gossip_dirs[i + 1])
        gossip_seconds = time.perf_counter() - gossip_start

        hub = root / "hub"
        run("init", hub)
        metadata_start = time.perf_counter()
        for i in range(100):
            peer = root / f"meta-{i}"
            run("init", peer)
            run("put", peer, f"meta:{i}", "m")
            run("sync", peer, hub)
        metadata_seconds = time.perf_counter() - metadata_start
        summary = run("export-summary", hub)
        hub_status = json.loads(run("status", hub))
        snapshot_seconds, snapshot_path = timed(lambda: run("snapshot", hub))

        report = {
            "claim_policy": "observational smoke benchmark only; no superiority claim",
            "environment": {
                "platform": platform.platform(),
                "machine": platform.machine(),
                "python": platform.python_version(),
                "rustc": subprocess.run(["rustc", "--version"], capture_output=True, text=True, check=True).stdout.strip(),
                "causalmesh": run("--version"),
            },
            "durable_cli": {
                "put_operations": puts,
                "put_seconds": put_seconds,
                "put_ops_per_second": puts / put_seconds,
                "mean_put_seconds": put_seconds / puts,
                "get_operations": gets,
                "get_seconds": get_seconds,
                "get_ops_per_second": gets / get_seconds,
                "reopen_verify_seconds": recovery_seconds,
            },
            "sync": {
                "zero_delta_seconds": zero_delta_seconds,
                "small_delta_operations": small,
                "small_delta_seconds": small_delta_seconds,
                "large_delta_operations": large,
                "large_delta_seconds": large_delta_seconds,
                "two_replica_conflict_convergence_seconds": two_replica_seconds,
                "ten_replica_gossip_seconds": gossip_seconds,
            },
            "metadata_100_replicas": {
                "construction_and_sync_seconds": metadata_seconds,
                "summary_json_bytes": len(summary.encode("utf-8")),
                "hub_operation_count": hub_status["operation_count"],
                "hub_log_bytes": hub_status["log_bytes"],
            },
            "snapshot": {
                "seconds": snapshot_seconds,
                "path": snapshot_path,
            },
        }
        print(json.dumps(report, indent=2, sort_keys=True))
        return 0
    finally:
        shutil.rmtree(root, ignore_errors=True)


if __name__ == "__main__":
    raise SystemExit(main())

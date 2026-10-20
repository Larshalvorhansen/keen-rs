#!/usr/bin/env python3
"""Generate schema-3 reference files using pinned upstream Minsky's rewriteMky.py.

Requires the upstream pyminsky environment; never rewrites source fixtures.
No claim of semantic equivalence is made until outputs are reviewed.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import xml.etree.ElementTree as ET

UPSTREAM_SHA = "06923f7e4df5d2bcbc67c5cf726ecc1d609da775"
NAMES = ("exponentialGrowth.mky", "GoodwinLinear.mky", "1Free.mky")


def schema_version(path):
    root = ET.parse(path).getroot()
    for elem in root.iter():
        if elem.tag.split("}")[-1] == "schemaVersion":
            return (elem.text or "").strip()
    return None


def sha256(path):
    h = hashlib.sha256()
    with path.open("rb") as f:
        for chunk in iter(lambda: f.read(1024 * 1024), b""):
            h.update(chunk)
    return h.hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--upstream", required=True, type=Path, help="Original C++ Minsky repository")
    parser.add_argument("--fixtures", type=Path, default=Path(__file__).resolve().parents[1] / "tests/fixtures/legacy")
    parser.add_argument("--output", type=Path, default=Path(__file__).resolve().parents[1] / "tests/reference-generated")
    parser.add_argument("--python", default=sys.executable, help="Interpreter with upstream pyminsky installed")
    parser.add_argument("--check-only", action="store_true", help="Validate prerequisites without running conversions")
    args = parser.parse_args()

    upstream = args.upstream.resolve()
    fixtures = args.fixtures.resolve()
    output = args.output.resolve()
    script = upstream / "test/rewriteMky.py"
    if not script.is_file():
        parser.error(f"Upstream migration script missing: {script}")
    rev = subprocess.run(["git", "-C", str(upstream), "rev-parse", "HEAD"], capture_output=True, text=True, check=True).stdout.strip()
    if rev != UPSTREAM_SHA:
        parser.error(f"Upstream checkout is {rev}, expected {UPSTREAM_SHA}")
    jobs = []
    for version in ("schema1", "schema2"):
        for name in NAMES:
            src = fixtures / version / name
            if not src.is_file():
                parser.error(f"Missing fixture {src}")
            expected = "1" if version == "schema1" else "2"
            if schema_version(src) != expected:
                parser.error(f"Fixture does not declare schema {expected}: {src}")
            dest = output / version / name
            if dest.resolve() == src.resolve():
                parser.error("Refusing to overwrite original fixture")
            jobs.append((version, name, src, dest))
    print(f"Pinned upstream: {rev}; fixtures: {len(jobs)}")
    if args.check_only:
        print("Preflight passed (upstream pyminsky execution not checked)")
        return
    # Never overwrite prior golden candidates, even after a partial failure.
    existing = [str(dst) for _, _, _, dst in jobs if dst.exists()]
    if existing:
        parser.error("Reference files already exist; refusing to overwrite: " + ", ".join(existing))
    records = []
    for version, name, src, dst in jobs:
        dst.parent.mkdir(parents=True, exist_ok=True)
        print(f"Converting {version}/{name}", flush=True)
        try:
            subprocess.run([args.python, str(script), str(src), str(dst)], cwd=upstream, check=True)
            if schema_version(dst) != "3":
                raise RuntimeError(f"Converted file does not declare schema 3: {dst}")
        except Exception:
            if dst.exists():
                dst.unlink()  # incomplete output is never retained
            raise
        records.append(dict(version=version, name=name, source_sha256=sha256(src), converted_sha256=sha256(dst)))
    manifest = {"upstream_commit": UPSTREAM_SHA, "generator": "test/rewriteMky.py", "references": records}
    (output / "manifest.json").write_text(json.dumps(manifest, indent=2) + "\n")
    print("Generated reference models in", output)
    print("Note: inspect semantic equivalence; generation alone is not validation")


if __name__ == "__main__":
    main()

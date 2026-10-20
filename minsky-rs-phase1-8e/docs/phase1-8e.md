# Phase 1.8e — upstream reference generation

Pinned upstream: `06923f7e4df5d2bcbc67c5cf726ecc1d609da775`.

Upstream source `test/rewriteMky.py` calls `minsky.load(...)`, `minsky.garbageCollect()`, and `minsky.save(...)` through `pyminsky`. This tool invokes that original script on six genuine schema 1/2 fixtures and records original/output SHA-256 values. Schema 0 requires a separate loader audit and is intentionally excluded.

The script refuses to overwrite prior outputs or fixtures, checks the upstream commit, and verifies each output declares schema 3. It does **not** mean numerical or semantic equivalence has been verified. Requires a working `pyminsky` environment and original native Minsky libraries, which are **not** supplied by the Rust workspace. `--check-only` verifies the pinned checkout and fixture paths without invoking `pyminsky`.

```bash
python3 tools/generate_legacy_references.py --upstream ../minsky --check-only
python3 tools/generate_legacy_references.py --upstream ../minsky
```

Generated references are deliberately excluded from version control until reviewed for correctness. If `pyminsky` is unavailable, the preflight check can still pass; report the module error instead of changing the conversion script.

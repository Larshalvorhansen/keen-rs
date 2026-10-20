# Minsky-RS Phase 1.8d — Genuine upstream legacy fixtures

This **test-only** increment adds nine authentic historical models from `test/oldSchema/` in upstream Minsky at pinned commit `06923f7e4df5d2bcbc67c5cf726ecc1d609da775`.

The source archive contains **9** schema-0, **21** schema-1, and **24** schema-2 `.mky` models. This increment includes three representative examples from each generation, retaining the original bytes. SHA-256 checksums are recorded in `docs/legacy-fixture-manifest.md`.

* Schema 1 and 2 use the `<Minsky>` root and declare their version.
* Schema 0 uses `<root>` and is deliberately **not** accepted by the current `<Minsky>` probe.
* No schema migration, file writing, or semantic equivalence is claimed.
* The schema-2 structural preview remains deliberately restricted. Its output MUST NOT be saved as schema 3.

Next engineering step: run the upstream C++ conversion on these pinned historical fixtures to produce reference schema-3 outputs, then implement a tested semantic migration increment.

## Test

```bash
cargo test --workspace
cargo test -p minsky-reference-tests --test genuine_legacy_fixtures
cargo run -p minsky-cli -- --probe-schema tests/fixtures/upstream/exponentialGrowth.mky
cargo run -p minsky-cli -- tests/fixtures/upstream/exponentialGrowth.mky
```

## Phase 1.8e

Adds `tools/generate_legacy_references.py`, a guarded reference generator using upstream `test/rewriteMky.py`. See `docs/phase1-8e.md`. No Rust library changes.

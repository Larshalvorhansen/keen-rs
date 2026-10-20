# Minsky-RS Phase 1.8a

Preserves Phase 1.7 behavior and adds an opt-in, read-only schema generation probe; **not** a legacy model converter.

```
cargo test --workspace
cargo run -p minsky-cli -- --probe-schema tests/fixtures/upstream/exponentialGrowth.mky
cargo run -p minsky-cli -- tests/fixtures/upstream/exponentialGrowth.mky
```

The probe recognizes declared versions 1, 2, 3 and unknown versions; absent version is marked *unversioned* rather than assumed to be schema 0. See `docs/legacy-schema.md`.

Dependencies unchanged. No mutation of fixtures.

# Minsky-RS — Phase 1.2 read-only schema 3 inspection

Upstream reference: `06923f7e4df5d2bcbc67c5cf726ecc1d609da775`.

This increment implements **inspection only**: top-level metadata, item IDs/types/port IDs,
wire IDs/endpoints, and group IDs/child item IDs. It does not validate references,
decode tensor/Ravel/Godley details, migrate schemas, or save models. It does not
claim to be a complete XML-to-domain-model parser.

Inside your Nix development shell, from the workspace root:

```sh
cargo test --workspace
cargo run -p minsky-cli -- tests/fixtures/upstream/exponentialGrowth.mky
cargo run -p minsky-cli -- tests/fixtures/upstream/indexing.mky
```

Expected: tests pass; CLI prints schema 3, item and wire counts, and each item ID/type.

Dependency added: `quick-xml = 0.37.5` in `minsky-format`, for XML tokenization
and entity decoding. The exact version is pinned deliberately for reproducibility.

**No save functionality.** Retain the original model files unchanged.

## Phase 1.3 — Read-only structural validation

`minsky_format::validate_structure(&summary)` returns diagnostics with stable codes. The CLI prints them and exits unsuccessfully if any are found. It validates only the subset parsed in 1.2, plus group input/output variable IDs; it does **not** yet validate port direction, connections to unparsed schema entities, or accounting semantics.

Run `cargo test --workspace` and `cargo run -p minsky-cli -- tests/fixtures/upstream/exponentialGrowth.mky`.

Fixture files are never modified.

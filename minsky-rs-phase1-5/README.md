# Minsky-RS — Phase 1.5

This is a complete workspace extending Phase 1.4 with **read-only Godley/Ravel metadata inspection**.

## Changed files

- `crates/minsky-format/src/metadata.rs`: Godley table rows, asset classes, display flags, and selected Ravel metadata.
- `crates/minsky-format/src/lib.rs`: exports the metadata module.
- `crates/minsky-reference-tests/tests/specialized_metadata.rs`: tests against a genuine upstream Godley fixture and a small synthetic Ravel fixture.
- `tests/fixtures/upstream/EndogenousMoney.mky`: pinned upstream fixture.

## Limits

- No XML writing, Godley semantics validation, or simulation.
- Ravel handle states and lock-handle details are deliberately opaque; radius, output handle names, lock-group ID and group membership are the limited supported subset.
- The normal CLI remains unchanged.
- No new crates/dependencies.

## Test

```bash
cargo test --workspace
cargo run -p minsky-cli -- tests/fixtures/upstream/exponentialGrowth.mky
```

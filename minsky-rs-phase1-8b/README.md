# Minsky-RS Phase 1.8b — restricted schema-2 preview

Adds `minsky_format::legacy_migration::preview_schema2` for **read-only** structural inspection of a deliberately restricted subset of schema-2 models. Not a general migration or a schema-3 serializer. Unsupported XML fields and attributes are rejected. Original XML is kept verbatim. Important unimplemented conversions include schema-2 tensor data re-encoding, slider representation, zoomFactor, Godley tables, Ravel state, schema defaults, and legacy schema 0/1 migrations. We deliberately do not expose an output file writer.

Based on upstream `schema/schema3.h`, notably `Item(const schema2::Item&)`, `Minsky(const schema2::Minsky&)`, and `schema/schema3.cc` tensor conversion.

## Local tests

```bash
cargo test --workspace
cargo run -p minsky-cli -- tests/fixtures/upstream/exponentialGrowth.mky
```

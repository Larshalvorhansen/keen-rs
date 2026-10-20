# Minsky-RS Phase 1.6a — tensor binary envelope

Extends 1.5; decodes schema3 `tensorData` CDATA through upstream's ASCII85 (`~` for `]`) and zlib layers. **Does not unpack `classdesc::pack_t` into values, indexes, axes or dimensions**; doing so requires additional verified binary-format work. Unmodified XML remains byte-identical through the preserved-document layer.

Dependency added: `flate2 = 1.1.2` (Rust zlib decompression), plus its transitive dependencies. Must be resolved by Cargo in the Nix shell.

## Test

```bash
cargo test --workspace
```

New test uses actual `importedCSV.mky` from the pinned upstream examples. No model writing or execution added.

## Phase 1.6a.1 compile correction

Fixed Rust E0716 in `tensor_payload.rs` by owning the end-element name before comparison. No decoding semantics changed.

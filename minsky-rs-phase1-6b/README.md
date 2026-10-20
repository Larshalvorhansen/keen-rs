# Minsky-RS Phase 1.6b — read-only binary tensor decoding

Based on Minsky commit `06923f7e4df5d2bcbc67c5cf726ecc1d609da775`.

- Decodes schema 3 tensor values, sparse indexes, and axis names/labels/units from classdesc-style little-endian packed bytes.
- Real `importedCSV.mky` verifies dense (3x3) and sparse (3x3) tensors.
- Does not write tensors, execute computations, or implement all possible dimension formats; preserve original XML for round trips.
- `tensor_binary::decode_tensor_binary` consumes the existing `tensor_payload::decode_envelope` output.
- Existing broken `EndogenousMoney.mky` include path corrected.
- No new dependencies. Rust 1.85, edition 2024.

## Local verification

```bash
mv ~/Downloads/minsky-rs-phase1-6b.zip ~/Documents/keen-rs/
cd ~/Documents/keen-rs
unzip -q minsky-rs-phase1-6b.zip
cd minsky-rs-phase1-6b
nix develop
cargo test --workspace
cargo run -p minsky-cli -- tests/fixtures/upstream/exponentialGrowth.mky
```

## Caveat

The C++ classdesc primitive layout inferred from pinned upstream fixtures must be checked against its `pack_base.h` implementation before claiming all schema-3 tensors work. Unsupported layouts should return an error rather than produce fabricated values.

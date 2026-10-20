# Minsky-RS Phase 1.7 — conservative atomic saves

Based on Phase 1.6b and upstream Minsky commit `06923f7e4df5d2bcbc67c5cf726ecc1d609da775`.

- `PreservedDocument::write_new`: checks schema 3 and known structural references, creates and syncs a temporary file alongside destination, verifies bytes on disk, then atomically links into place without overwriting existing files.
- `PreservedDocument::replace_if_unchanged`: guards against detected external edits by comparing the destination against original bytes, then atomically renames a verified temporary file. **Not concurrency-safe**; external writers can race after the check. Only use when simultaneous editing is ruled out.
- Unmodified XML preserves original bytes. Title edits retain unrelated bytes.
- Unknown schema content remains opaque; structural validation applies **only** to the currently inspected subset.
- Five new integration tests for byte preservation, non-overwrite, external modification detection, successful replacement, and corrupt structural references.
- No new dependencies. No tensor, Godley, or Ravel editing.

## Limitations

- New-file creation uses hard linking and requires filesystem support for hard links.
- Directory fsync errors may be reported after destination creation; consult the destination before retrying.
- Replacement may change file metadata such as permissions. It must not be exposed as a production overwrite command until permission and concurrency handling are finalized.
- For existing models not understood by our partial validator, save operations may be conservatively rejected.
- Cargo.lock is not bundled, because this environment does not have Cargo available to generate a verified lock file. On Mac, after successful build, retain Cargo.lock in git and carry it between releases.

## Test locally (Mac + Nix)

```bash
mv ~/Downloads/minsky-rs-phase1-7.zip ~/Documents/keen-rs/
cd ~/Documents/keen-rs
unzip -q minsky-rs-phase1-7.zip
cd minsky-rs-phase1-7
nix develop
cargo test --workspace
cargo run -p minsky-cli -- tests/fixtures/upstream/exponentialGrowth.mky
```

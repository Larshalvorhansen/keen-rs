# Minsky-RS — Phase 1.1 workspace

Baseline: highperformancecoder/minsky @ `06923f7e4df5d2bcbc67c5cf726ecc1d609da775`.

This increment only establishes a Rust 2024 workspace and fixture presence checks.
It does **not** parse `.mky`, execute simulations, or launch a GUI.

From the workspace root, inside your existing Nix shell:

```sh
cargo test --workspace
cargo run -p minsky-cli
```

Expected test results: 2 fixture smoke tests passing. The CLI prints a scaffold message.

Next increment: read-only schema 3 parser design and implementation.

License notice: Workspace manifest expresses the intended GPL-3.0-or-later license.
A final release requires inclusion of the actual GPL license text and license audit.

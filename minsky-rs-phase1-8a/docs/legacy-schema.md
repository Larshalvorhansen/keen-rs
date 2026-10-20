# Phase 1.8a — Legacy schema reconnaissance

Pinned upstream revision: `06923f7e4df5d2bcbc67c5cf726ecc1d609da775`.

- `schema/schema0.h` defines a distinct legacy model organized around maps of ports, wires, operations, variables and groups. Its `Minsky` struct does not declare `schemaVersion`.
- `schema/schema1.h` declares `version=1`, with top-level `model` and `layout` structures.
- `schema/schema2.h` declares `version=2`, with top-level vectors for `items`, `wires`, `groups`.
- `schema/schema3.h` declares version 3; all present reference fixtures are schema 3.

**This release does not migrate models.** Missing `schemaVersion` means unversioned/unknown, not verified schema 0. Only schema 3 can pass through the current structural parser. Legacy files remain read-only and must not be rewritten with schema-3 serialization routines. Versions are detected solely from XML declarations, not verified by checking full legacy schemas.

Before implementing conversion, obtain representative upstream-generated legacy fixtures and examine `loadSchema` and schema conversion constructors in `schema1.cc`, `schema2.cc`, `schema3.cc`.

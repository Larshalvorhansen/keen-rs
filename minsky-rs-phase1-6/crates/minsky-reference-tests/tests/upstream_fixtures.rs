//! Phase 1.1 smoke tests only; no XML parsing or semantic validation.
use std::{fs, path::PathBuf};

fn fixture(name: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/upstream")
        .join(name);
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

#[test]
fn exponential_growth_fixture_is_present() {
    let xml = fixture("exponentialGrowth.mky");
    assert!(xml.contains("<Minsky "));
    assert!(xml.contains("<schemaVersion>3</schemaVersion>"));
}

#[test]
fn indexing_fixture_is_present() {
    let xml = fixture("indexing.mky");
    assert!(xml.contains("<Minsky "));
    assert!(xml.contains("<schemaVersion>3</schemaVersion>"));
}

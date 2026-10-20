//! Tests based on actual upstream test/oldSchema fixtures, not synthetic XML.
//! These tests intentionally do not claim successful semantic conversion.
use minsky_format::schema_probe::{probe_schema, SchemaGeneration};
use minsky_format::legacy_migration::preview_schema2;
use std::{fs, path::PathBuf};

fn fixture(generation: &str, name: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/legacy")
        .join(generation)
        .join(name);
    fs::read_to_string(path).expect("upstream fixture should be available as UTF-8")
}

#[test]
fn genuine_schema1_files_are_detected_but_not_treated_as_schema3() {
    for name in ["exponentialGrowth.mky", "GoodwinLinear.mky", "1Free.mky"] {
        let xml = fixture("schema1", name);
        assert_eq!(probe_schema(&xml).unwrap(), SchemaGeneration::V1, "{name}");
        assert!(!probe_schema(&xml).unwrap().can_inspect_with_current_parser());
    }
}

#[test]
fn genuine_schema2_files_are_detected_but_not_treated_as_schema3() {
    for name in ["exponentialGrowth.mky", "GoodwinLinear.mky", "1Free.mky"] {
        let xml = fixture("schema2", name);
        assert_eq!(probe_schema(&xml).unwrap(), SchemaGeneration::V2, "{name}");
        assert!(!probe_schema(&xml).unwrap().can_inspect_with_current_parser());
    }
}

#[test]
fn historical_root_format_is_explicitly_not_supported_by_minsky_root_probe() {
    for name in ["exponentialGrowth.mky", "GoodwinLinear.mky", "1Free.mky"] {
        let xml = fixture("schema0", name);
        // Historical schema0 fixtures are rooted at <root>, not <Minsky>.
        assert!(xml.trim_start().starts_with("<root>"), "{name}");
        assert!(probe_schema(&xml).is_err(), "{name} must not be silently migrated");
    }
}

#[test]
fn genuine_schema2_sources_are_never_modified_by_restricted_preview() {
    for name in ["exponentialGrowth.mky", "GoodwinLinear.mky", "1Free.mky"] {
        let xml = fixture("schema2", name);
        let before = xml.as_bytes().to_vec();
        // A success is only a structural preview. An error must be explicit.
        match preview_schema2(&xml) {
            Ok(preview) => assert_eq!(preview.source_xml.as_bytes(), before, "{name}"),
            Err(error) => assert!(!error.to_string().is_empty(), "{name}"),
        }
        assert_eq!(xml.as_bytes(), before, "{name}");
    }
}

use minsky_format::schema_probe::{probe_schema, SchemaGeneration as S};

#[test]
fn detects_declared_legacy_generations() {
    for (xml, expected) in [
        ("<Minsky><schemaVersion>1</schemaVersion><model/></Minsky>", S::V1),
        ("<Minsky><schemaVersion>2</schemaVersion><items/></Minsky>", S::V2),
        ("<Minsky><schemaVersion>3</schemaVersion><items/></Minsky>", S::V3),
        ("<Minsky><ports/><wires/></Minsky>", S::LegacyUnversioned),
        ("<Minsky><schemaVersion>19</schemaVersion></Minsky>", S::Unsupported(19)),
    ] { assert_eq!(probe_schema(xml).unwrap(), expected); }
}

#[test]
fn denies_legacy_inspection_until_migration_implemented() {
    assert!(!S::V1.can_inspect_with_current_parser());
    assert!(!S::V2.can_inspect_with_current_parser());
    assert!(!S::LegacyUnversioned.can_inspect_with_current_parser());
    assert!(S::V3.can_inspect_with_current_parser());
}

#[test]
fn rejects_invalid_versions_and_non_minsky_xml() {
    for xml in ["<Other/>", "<Minsky><schemaVersion>abc</schemaVersion></Minsky>", "<Minsky><schemaVersion/></Minsky>", "<Minsky>"] {
        assert!(probe_schema(xml).is_err(), "unexpected success: {xml}");
    }
}

#[test]
fn upstream_schema3_fixture_is_recognized() {
    let xml = include_str!("../../../tests/fixtures/upstream/exponentialGrowth.mky");
    assert_eq!(probe_schema(xml).unwrap(), S::V3);
}

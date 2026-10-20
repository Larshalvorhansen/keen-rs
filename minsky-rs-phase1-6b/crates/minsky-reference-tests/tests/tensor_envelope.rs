use minsky_format::tensor_payload::{decode_envelope, inspect_tensor_payloads};
#[test]
fn decodes_upstream_tensor_envelope() {
    let xml = include_str!("../../../tests/fixtures/upstream/importedCSV.mky");
    let p = inspect_tensor_payloads(xml).expect("valid upstream tensor envelope");
    assert!(!p.is_empty(), "fixture must contain tensorData");
    assert!(p.iter().all(|v| !v.packed_data.is_empty()));
}
#[test]
fn malformed_envelopes_are_errors() {
    assert!(decode_envelope("z").is_err());
    assert!(decode_envelope("!!!!").is_err());
    assert!(decode_envelope("!!!!!!!!!!").is_err());
}

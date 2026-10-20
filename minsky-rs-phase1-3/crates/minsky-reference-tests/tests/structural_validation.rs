use minsky_format::{inspect_xml, validate_structure};

#[test]
fn genuine_example_is_structurally_valid() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../tests/fixtures/upstream/exponentialGrowth.mky");
    let model = minsky_format::inspect_file(path).unwrap();
    let issues = validate_structure(&model);
    assert!(issues.is_empty(), "{issues:#?}");
}

#[test]
fn catches_missing_endpoints_and_duplicate_ids() {
    let xml = r#"<Minsky><schemaVersion>3</schemaVersion><items>
      <Item><id>1</id><ports><int>10</int><int>10</int></ports></Item>
      <Item><id>1</id><ports><int>11</int></ports></Item>
      </items><wires><Wire><id>5</id><from>10</from><to>99</to></Wire>
      <Wire><id>5</id><from>11</from></Wire></wires>
      <groups><Group><id>3</id><items><int>404</int></items></Group></groups></Minsky>"#;
    let codes: Vec<_> = validate_structure(&inspect_xml(xml).unwrap()).into_iter().map(|d| d.code).collect();
    for expected in ["duplicate_object_id", "duplicate_port_id", "duplicate_wire_id", "unknown_port", "missing_wire_endpoint", "unknown_group_object"] {
        assert!(codes.contains(&expected), "missing {expected} in {codes:?}");
    }
}

#[test]
fn group_ids_share_object_namespace() {
    let xml = r#"<Minsky><schemaVersion>3</schemaVersion><items><Item><id>7</id></Item></items><groups><Group><id>7</id></Group></groups></Minsky>"#;
    assert!(validate_structure(&inspect_xml(xml).unwrap()).iter().any(|d| d.code == "duplicate_object_id"));
}

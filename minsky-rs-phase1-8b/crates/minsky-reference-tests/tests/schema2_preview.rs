use minsky_format::legacy_migration::preview_schema2;

#[test]
fn converts_minimal_schema2_structure_read_only() {
    let xml = r#"<Minsky><schemaVersion>2</schemaVersion><items><Item><id>1</id><type>Variable:flow</type><ports><int>7</int></ports></Item></items><wires/><groups/></Minsky>"#;
    let preview = preview_schema2(xml).unwrap();
    assert_eq!(preview.model.items.len(), 1);
    assert_eq!(preview.model.items[0].ports, vec![7]);
    assert_eq!(preview.source_xml.as_bytes(), xml.as_bytes());
}

#[test]
fn refuses_fields_that_need_semantic_migration() {
    let xml = "<Minsky><schemaVersion>2</schemaVersion><items><Item><id>1</id><tensorData>data</tensorData></Item></items></Minsky>";
    assert!(preview_schema2(xml).unwrap_err().to_string().contains("tensorData"));
}

#[test]
fn rejects_schema3_and_unrecognized_fields() {
    assert!(preview_schema2("<Minsky><schemaVersion>3</schemaVersion></Minsky>").is_err());
    assert!(preview_schema2("<Minsky><schemaVersion>2</schemaVersion><surprise/></Minsky>").is_err());
}

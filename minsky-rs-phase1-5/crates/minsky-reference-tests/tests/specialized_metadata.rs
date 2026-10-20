use minsky_format::metadata::inspect_specialized_xml;

#[test]
fn reads_real_upstream_godley_tables_and_preserves_empty_cells() {
    let xml = include_str!("../../../tests/fixtures/upstream/EndogenousMoney.mky");
    let metadata = inspect_specialized_xml(xml).unwrap();
    assert_eq!(metadata.godley_tables.len(), 4);
    let first = &metadata.godley_tables[0];
    assert_eq!(first.item_id, Some(450));
    assert_eq!(first.rows[0][0], "Flows V / Stock Variables ->");
    assert_eq!(first.rows[0][1], "Reserves");
    assert_eq!(first.rows[1][1], "90");
    assert_eq!(first.rows[2][2], "Lend");
    assert_eq!(first.rows[2][1], "");
    assert_eq!(first.asset_classes[1], "asset");
}

#[test]
fn reads_selected_ravel_fields_without_claiming_full_state_support() {
    let xml = r#"<Minsky><schemaVersion>3</schemaVersion><items><Item><id>7</id><type>Ravel</type><ravelState><radius>5.25</radius><outputHandles><string>Year</string></outputHandles><handleStates><unknown>retain</unknown></handleStates></ravelState><lockGroup>2</lockGroup><lockGroupHandles><string>Year</string></lockGroupHandles></Item></items><lockGroups><LockGroup><ravels><int>7</int></ravels></LockGroup></lockGroups></Minsky>"#;
    let data = inspect_specialized_xml(xml).unwrap();
    assert_eq!(data.ravels.len(), 1);
    assert_eq!(data.ravels[0].item_id, Some(7));
    assert_eq!(data.ravels[0].radius, Some(5.25));
    assert_eq!(data.ravels[0].output_handles, vec!["Year"]);
    assert_eq!(data.ravels[0].lock_group, Some(2));
    assert_eq!(data.lock_groups[0].ravel_item_ids, vec![7]);
}

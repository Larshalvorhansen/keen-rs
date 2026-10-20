use std::path::PathBuf;

fn model(name: &str) -> minsky_format::ModelSummary {
    let file = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/upstream")
        .join(name);
    minsky_format::inspect_file(file).unwrap()
}

#[test]
fn reads_exponential_growth() {
    let model = model("exponentialGrowth.mky");
    assert_eq!(model.schema_version, 3);
    assert!(!model.items.is_empty());
    assert!(!model.wires.is_empty());
    assert!(model.items.iter().all(|item| item.id.is_some()));
    assert!(model.wires.iter().all(|wire| wire.id.is_some() && wire.from.is_some() && wire.to.is_some()));
}

#[test]
fn reads_indexing() {
    let model = model("indexing.mky");
    assert_eq!(model.schema_version, 3);
    assert!(!model.items.is_empty());
}

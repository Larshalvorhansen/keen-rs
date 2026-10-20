//! Read-only extraction of selected schema-3 Godley and Ravel fields.
//! This is intentionally not a complete deserializer. Unrecognized content is
//! left in the original XML and is never written by this module.
use quick_xml::{events::Event, Reader};
use crate::ParseError;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct GodleyMetadata {
    pub item_id: Option<i64>,
    pub rows: Vec<Vec<String>>,
    pub asset_classes: Vec<String>,
    pub currency: Option<String>,
    pub editor_mode: Option<bool>,
    pub button_display: Option<bool>,
    pub variable_display: Option<bool>,
    pub display_values: Option<bool>,
}
#[derive(Debug, Clone, PartialEq, Default)]
pub struct RavelMetadata {
    pub item_id: Option<i64>,
    pub radius: Option<f64>,
    pub output_handles: Vec<String>,
    pub handle_state_names: Vec<String>,
    pub lock_group: Option<i64>,
    pub lock_group_handles: Vec<String>,
}
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RavelLockGroupMetadata {
    pub ravel_item_ids: Vec<i64>,
    /// Number of opaque HandleLockInfo entries, not decoded in this phase.
    pub handle_lock_info_count: usize,
}
#[derive(Debug, Clone, PartialEq, Default)]
pub struct SpecializedMetadata {
    pub godley_tables: Vec<GodleyMetadata>,
    pub ravels: Vec<RavelMetadata>,
    pub lock_groups: Vec<RavelLockGroupMetadata>,
}

/// Extract metadata without taking ownership of or rewriting XML.
/// Only top-level `Minsky/items/Item` entries and `Minsky/lockGroups` are read.
pub fn inspect_specialized_xml(xml: &str) -> Result<SpecializedMetadata, ParseError> {
    // Validate root and schema version using the existing parser first.
    crate::inspect_xml(xml)?;
    let mut reader = Reader::from_str(xml);
    let mut path = Vec::<String>::new();
    let mut result = SpecializedMetadata::default();
    let mut id = None;
    let mut kind = String::new();
    let mut godley = GodleyMetadata::default();
    let mut ravel = RavelMetadata::default();
    let mut lock_group = RavelLockGroupMetadata::default();
    let mut text = String::new();
    loop {
        match reader.read_event().map_err(|e| ParseError(e.to_string()))? {
            Event::Start(e) => {
                let name = String::from_utf8_lossy(e.local_name().as_ref()).into_owned();
                path.push(name);
                text.clear();
                if path_eq(&path, &["Minsky", "items", "Item"]) {
                    id = None; kind.clear(); godley = GodleyMetadata::default(); ravel = RavelMetadata::default();
                }
                if path_eq(&path, &["Minsky", "lockGroups", "LockGroup"]) {
                    lock_group = RavelLockGroupMetadata::default();
                }
                if path_eq(&path, &["Minsky", "items", "Item", "data", "vector"]) {
                    godley.rows.push(Vec::new());
                }
            }
            Event::Empty(e) => {
                let name = String::from_utf8_lossy(e.local_name().as_ref()).into_owned();
                // Empty Godley cells are significant: do not lose column positions.
                if name == "string" && path_eq(&path, &["Minsky", "items", "Item", "data", "vector"]) {
                    if let Some(row) = godley.rows.last_mut() { row.push(String::new()); }
                }
                if name == "HandleLockInfo" && path_eq(&path, &["Minsky", "lockGroups", "LockGroup", "handleLockInfo"]) {
                    lock_group.handle_lock_info_count += 1;
                }
            }
            Event::Text(e) => { text.push_str(&e.unescape().map_err(|e| ParseError(e.to_string()))?); }
            Event::CData(e) => { text.push_str(&String::from_utf8_lossy(e.as_ref())); }
            Event::End(e) => {
                let name = String::from_utf8_lossy(e.local_name().as_ref()).into_owned();
                if path.last().map(String::as_str) != Some(name.as_str()) { return Err(ParseError(format!("unexpected end tag: {name}"))); }
                let value = text.trim();
                let p = &path;
                if in_item(p, "id") { id = Some(integer(value, "item id")?); }
                else if in_item(p, "type") { kind = value.to_owned(); }
                else if in_item(p, "currency") { godley.currency = Some(value.to_owned()); }
                else if in_item(p, "editorMode") { godley.editor_mode = Some(boolean(value)?); }
                else if in_item(p, "buttonDisplay") { godley.button_display = Some(boolean(value)?); }
                else if in_item(p, "variableDisplay") { godley.variable_display = Some(boolean(value)?); }
                else if in_item(p, "displayValues") { godley.display_values = Some(boolean(value)?); }
                else if path_eq(p, &["Minsky", "items", "Item", "assetClasses", "AssetClass"]) { godley.asset_classes.push(value.to_owned()); }
                else if path_eq(p, &["Minsky", "items", "Item", "data", "vector", "string"]) {
                    if let Some(row) = godley.rows.last_mut() { row.push(text.clone()); }
                }
                else if path_eq(p, &["Minsky", "items", "Item", "ravelState", "radius"]) { ravel.radius = Some(value.parse().map_err(|_| ParseError(format!("invalid Ravel radius: {value}")))?); }
                else if path_eq(p, &["Minsky", "items", "Item", "ravelState", "outputHandles", "string"]) { ravel.output_handles.push(text.clone()); }
                else if in_item(p, "lockGroup") { ravel.lock_group = Some(integer(value, "lock group ID")?); }
                else if path_eq(p, &["Minsky", "items", "Item", "lockGroupHandles", "string"]) { ravel.lock_group_handles.push(text.clone()); }
                else if path_eq(p, &["Minsky", "lockGroups", "LockGroup", "ravels", "int"]) { lock_group.ravel_item_ids.push(integer(value, "Ravel item ID")?); }
                if path_eq(p, &["Minsky", "lockGroups", "LockGroup", "handleLockInfo", "HandleLockInfo"]) { lock_group.handle_lock_info_count += 1; }
                if path_eq(p, &["Minsky", "lockGroups", "LockGroup"]) { result.lock_groups.push(std::mem::take(&mut lock_group)); }
                if path_eq(p, &["Minsky", "items", "Item"]) {
                    if kind == "GodleyIcon" { godley.item_id = id; result.godley_tables.push(std::mem::take(&mut godley)); }
                    if kind == "Ravel" || kind == "RavelIcon" { ravel.item_id = id; result.ravels.push(std::mem::take(&mut ravel)); }
                }
                path.pop(); text.clear();
            }
            Event::Eof => break,
            _ => {}
        }
    }
    Ok(result)
}
fn path_eq(actual: &[String], expected: &[&str]) -> bool {
    actual.len() == expected.len() && actual.iter().map(String::as_str).eq(expected.iter().copied())
}
fn in_item(path: &[String], field: &str) -> bool {
    path.len() == 4 && path[0] == "Minsky" && path[1] == "items" && path[2] == "Item" && path[3] == field
}
fn integer(value: &str, field: &str) -> Result<i64, ParseError> {
    value.parse().map_err(|_| ParseError(format!("invalid {field}: {value:?}")))
}
fn boolean(value: &str) -> Result<bool, ParseError> {
    match value { "true" | "1" => Ok(true), "false" | "0" => Ok(false), _ => Err(ParseError(format!("invalid boolean: {value:?}"))) }
}

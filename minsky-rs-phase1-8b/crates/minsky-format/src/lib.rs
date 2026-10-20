//! Partial inspection of Minsky schema-3 XML.
//! Safe-save operations are limited to byte-preserving documents and supported title edits;
//! this is not a complete model serializer.

pub mod safe_save;
pub mod schema_probe;
pub mod legacy_migration;

use quick_xml::{events::Event, Reader};
use std::{collections::{HashMap, HashSet}, error::Error, fmt, fs, path::Path};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelSummary {
    pub schema_version: u32,
    pub minsky_version: Option<String>,
    pub title: String,
    pub author: String,
    pub items: Vec<ItemSummary>,
    pub wires: Vec<WireSummary>,
    pub groups: Vec<GroupSummary>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ItemSummary {
    pub id: Option<i64>,
    pub kind: Option<String>,
    pub ports: Vec<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct WireSummary {
    pub id: Option<i64>,
    pub from: Option<i64>,
    pub to: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct GroupSummary {
    pub id: Option<i64>,
    pub kind: Option<String>,
    pub child_items: Vec<i64>,
    pub input_variables: Vec<i64>,
    pub output_variables: Vec<i64>,
}

#[derive(Debug)]
pub struct ParseError(pub String);
impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { f.write_str(&self.0) }
}
impl Error for ParseError {}

pub fn inspect_file(path: impl AsRef<Path>) -> Result<ModelSummary, ParseError> {
    let xml = fs::read_to_string(path).map_err(|e| ParseError(e.to_string()))?;
    inspect_xml(&xml)
}

/// Inspect only direct children of Minsky/items, Minsky/wires and Minsky/groups.
/// No graph validation, schema migration, tensor decoding or serialization occurs.
pub fn inspect_xml(xml: &str) -> Result<ModelSummary, ParseError> {
    let mut reader = Reader::from_str(xml);
    let mut path: Vec<String> = Vec::new();
    let mut summary = ModelSummary {
        schema_version: 0, minsky_version: None, title: String::new(),
        author: String::new(), items: vec![], wires: vec![], groups: vec![],
    };
    let mut item: Option<ItemSummary> = None;
    let mut wire: Option<WireSummary> = None;
    let mut group: Option<GroupSummary> = None;
    let mut saw_root = false;
    let mut saw_version = false;

    loop {
        match reader.read_event().map_err(|e| ParseError(e.to_string()))? {
            Event::Start(e) => {
                let name = String::from_utf8_lossy(e.local_name().as_ref()).into_owned();
                if path.is_empty() {
                    if name != "Minsky" || saw_root {
                        return Err(ParseError("expected a single <Minsky> root".into()));
                    }
                    saw_root = true;
                }
                if path == ["Minsky", "items"] && name == "Item" {
                    item = Some(ItemSummary::default());
                } else if path == ["Minsky", "wires"] && name == "Wire" {
                    wire = Some(WireSummary::default());
                } else if path == ["Minsky", "groups"] && name == "Group" {
                    group = Some(GroupSummary::default());
                }
                path.push(name);
            }
            Event::Empty(e) => {
                let name = String::from_utf8_lossy(e.local_name().as_ref()).into_owned();
                if path.is_empty() { return Err(ParseError("expected <Minsky> root".into())); }
                if path == ["Minsky", "items"] && name == "Item" { summary.items.push(ItemSummary::default()); }
                if path == ["Minsky", "wires"] && name == "Wire" { summary.wires.push(WireSummary::default()); }
                if path == ["Minsky", "groups"] && name == "Group" { summary.groups.push(GroupSummary::default()); }
            }
            Event::Text(e) => {
                let value = e.unescape().map_err(|e| ParseError(e.to_string()))?;
                let value = value.trim();
                match path.as_slice() {
                    [a,b] if a == "Minsky" && b == "schemaVersion" => {
                        summary.schema_version = parse_u32(value, "schemaVersion")?;
                        saw_version = true;
                    }
                    [a,b] if a == "Minsky" && b == "minskyVersion" => summary.minsky_version = Some(value.into()),
                    [a,b] if a == "Minsky" && b == "title" => summary.title.push_str(value),
                    [a,b] if a == "Minsky" && b == "author" => summary.author.push_str(value),
                    [a,b,c,d] if a == "Minsky" && b == "items" && c == "Item" => {
                        if let Some(item) = item.as_mut() {
                            match d.as_str() {
                                "id" => item.id = Some(parse_i64(value, "item ID")?),
                                "type" => item.kind = Some(value.into()),
                                _ => {}
                            }
                        }
                    }
                    [a,b,c,d,e] if a == "Minsky" && b == "items" && c == "Item" && d == "ports" && e == "int" => {
                        if let Some(item) = item.as_mut() { item.ports.push(parse_i64(value, "port ID")?); }
                    }
                    [a,b,c,d] if a == "Minsky" && b == "wires" && c == "Wire" => {
                        if let Some(wire) = wire.as_mut() {
                            match d.as_str() {
                                "id" => wire.id = Some(parse_i64(value, "wire ID")?),
                                "from" => wire.from = Some(parse_i64(value, "wire source")?),
                                "to" => wire.to = Some(parse_i64(value, "wire destination")?),
                                _ => {}
                            }
                        }
                    }
                    [a,b,c,d] if a == "Minsky" && b == "groups" && c == "Group" => {
                        if let Some(group) = group.as_mut() {
                            match d.as_str() {
                                "id" => group.id = Some(parse_i64(value, "group ID")?),
                                "type" => group.kind = Some(value.into()),
                                _ => {}
                            }
                        }
                    }
                    [a,b,c,d,e] if a == "Minsky" && b == "groups" && c == "Group" && d == "items" && e == "int" => {
                        if let Some(group) = group.as_mut() { group.child_items.push(parse_i64(value, "group item ID")?); }
                    }
                    [a,b,c,d,e] if a == "Minsky" && b == "groups" && c == "Group" && d == "inVariables" && e == "int" => {
                        if let Some(group) = group.as_mut() { group.input_variables.push(parse_i64(value, "group input variable ID")?); }
                    }
                    [a,b,c,d,e] if a == "Minsky" && b == "groups" && c == "Group" && d == "outVariables" && e == "int" => {
                        if let Some(group) = group.as_mut() { group.output_variables.push(parse_i64(value, "group output variable ID")?); }
                    }
                    _ => {}
                }
            }
            Event::End(e) => {
                let name = String::from_utf8_lossy(e.local_name().as_ref()).into_owned();
                if path.last().map(String::as_str) != Some(name.as_str()) {
                    return Err(ParseError(format!("mismatched closing element: {name}")));
                }
                match path.as_slice() {
                    [a,b,c] if a == "Minsky" && b == "items" && c == "Item" => {
                        summary.items.push(item.take().expect("item started"));
                    }
                    [a,b,c] if a == "Minsky" && b == "wires" && c == "Wire" => {
                        summary.wires.push(wire.take().expect("wire started"));
                    }
                    [a,b,c] if a == "Minsky" && b == "groups" && c == "Group" => {
                        summary.groups.push(group.take().expect("group started"));
                    }
                    _ => {}
                }
                path.pop();
            }
            Event::Eof => break,
            _ => {}
        }
    }
    if !saw_root || !path.is_empty() { return Err(ParseError("incomplete Minsky XML document".into())); }
    if !saw_version || summary.schema_version != 3 {
        return Err(ParseError(format!("expected schemaVersion 3, found {}", summary.schema_version)));
    }
    Ok(summary)
}

fn parse_i64(text: &str, field: &str) -> Result<i64, ParseError> {
    text.parse().map_err(|_| ParseError(format!("invalid {field}: {text:?}")))
}
fn parse_u32(text: &str, field: &str) -> Result<u32, ParseError> {
    text.parse().map_err(|_| ParseError(format!("invalid {field}: {text:?}")))
}

/// Diagnostic from the *inspected* structural subset. This is not full Minsky validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub code: &'static str,
    pub message: String,
}

/// Check IDs and references represented by `ModelSummary`.
/// Item and group IDs share an object namespace; wire IDs are unique.
/// Port IDs can be shared by different items in upstream models, so sharing
/// is not a duplicate-ID error. Repeating one port within one item is flagged.
/// Does not validate port direction, hidden schema fields, or cycles.
pub fn validate_structure(model: &ModelSummary) -> Vec<Diagnostic> {
    let mut problems = Vec::new();
    let mut objects = HashMap::<i64, String>::new();
    let mut ports = HashMap::<i64, String>::new();
    let mut wires = HashSet::<i64>::new();

    for (index, item) in model.items.iter().enumerate() {
        let owner = format!("item[{index}]");
        register_object(item.id, &owner, &mut objects, &mut problems);
        let mut item_ports = HashSet::new();
        for (port_index, port) in item.ports.iter().enumerate() {
            let location = format!("{owner}.ports[{port_index}]");
            if !item_ports.insert(*port) {
                problems.push(Diagnostic { code: "duplicate_item_port", message: format!("port ID {port} appears more than once on {owner}") });
            }
            // Different items may legitimately refer to one underlying port.
            ports.entry(*port).or_insert(location);
        }
    }
    for (index, group) in model.groups.iter().enumerate() {
        register_object(group.id, &format!("group[{index}]"), &mut objects, &mut problems);
    }
    for (index, wire) in model.wires.iter().enumerate() {
        let owner = format!("wire[{index}]");
        match wire.id {
            None => problems.push(Diagnostic { code: "missing_wire_id", message: format!("{owner} has no ID") }),
            Some(id) if !wires.insert(id) => problems.push(Diagnostic { code: "duplicate_wire_id", message: format!("wire ID {id} occurs more than once") }),
            _ => {}
        }
        for (field, endpoint) in [("from", wire.from), ("to", wire.to)] {
            match endpoint {
                None => problems.push(Diagnostic { code: "missing_wire_endpoint", message: format!("{owner}.{field} is missing") }),
                Some(id) if !ports.contains_key(&id) => problems.push(Diagnostic { code: "unknown_port", message: format!("{owner}.{field} references nonexistent port {id}") }),
                _ => {}
            }
        }
    }
    for (index, group) in model.groups.iter().enumerate() {
        for (field, refs) in [
            ("items", &group.child_items),
            ("inVariables", &group.input_variables),
            ("outVariables", &group.output_variables),
        ] {
            let mut seen = HashSet::new();
            for id in refs {
                if !seen.insert(*id) {
                    problems.push(Diagnostic { code: "duplicate_group_reference", message: format!("group[{index}].{field} repeats ID {id}") });
                }
                if !objects.contains_key(id) {
                    problems.push(Diagnostic { code: "unknown_group_object", message: format!("group[{index}].{field} references nonexistent object {id}") });
                }
            }
        }
    }
    problems
}

fn register_object(id: Option<i64>, owner: &str, ids: &mut HashMap<i64, String>, problems: &mut Vec<Diagnostic>) {
    match id {
        None => problems.push(Diagnostic { code: "missing_object_id", message: format!("{owner} has no ID") }),
        Some(id) => {
            if let Some(previous) = ids.insert(id, owner.to_owned()) {
                problems.push(Diagnostic { code: "duplicate_object_id", message: format!("object ID {id} used by both {previous} and {owner}") });
            }
        }
    }
}


/// Preservation-first schema 3 document. Unrecognized XML stays in the original
/// bytes, including whitespace, entity spelling, comments and binary payloads.
/// Only a narrowly scoped root title edit is supported in this increment.
#[derive(Debug, Clone)]
pub struct PreservedDocument {
    original: Vec<u8>,
    current: Vec<u8>,
    title_changed: bool,
}

impl PreservedDocument {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, ParseError> {
        let bytes = fs::read(path).map_err(|e| ParseError(e.to_string()))?;
        Self::from_bytes(bytes)
    }

    pub fn from_bytes(bytes: Vec<u8>) -> Result<Self, ParseError> {
        let xml = std::str::from_utf8(&bytes)
            .map_err(|e| ParseError(format!("file is not UTF-8: {e}")))?;
        inspect_xml(xml)?;
        Ok(Self { original: bytes.clone(), current: bytes, title_changed: false })
    }

    pub fn inspect(&self) -> Result<ModelSummary, ParseError> {
        let xml = std::str::from_utf8(&self.current)
            .map_err(|e| ParseError(e.to_string()))?;
        inspect_xml(xml)
    }

    pub fn is_modified(&self) -> bool { self.current != self.original }
    pub fn changed_fields(&self) -> Vec<&'static str> {
        if self.title_changed && self.is_modified() { vec!["title"] } else { vec![] }
    }
    pub fn as_bytes(&self) -> &[u8] { &self.current }

    /// Change only a direct `<Minsky><title>` text node. Reject titles with
    /// child elements or CDATA rather than risk damaging unknown structure.
    pub fn set_title(&mut self, title: &str) -> Result<(), ParseError> {
        let (start, end) = direct_title_range(&self.current)?;
        let escaped = escape_xml_text(title);
        let mut updated = Vec::with_capacity(self.current.len() + escaped.len());
        updated.extend_from_slice(&self.current[..start]);
        updated.extend_from_slice(escaped.as_bytes());
        updated.extend_from_slice(&self.current[end..]);
        let xml = std::str::from_utf8(&updated).map_err(|e| ParseError(e.to_string()))?;
        inspect_xml(xml)?;
        self.current = updated;
        self.title_changed = true;
        Ok(())
    }

    /// Save to a new path atomically, refusing to overwrite an existing file.
    /// A temporary file is validated, flushed, and hard-linked into place.
    pub fn write_new(&self, destination: impl AsRef<Path>) -> Result<(), ParseError> {
        crate::safe_save::save_new(self, destination.as_ref())
    }

    /// Replace an existing *regular file* only when its bytes match the
    /// document initially loaded. Not safe against simultaneous external writers.
    /// Avoid this API for files subject to concurrent editing.
    pub fn replace_if_unchanged(&self, destination: impl AsRef<Path>) -> Result<(), ParseError> {
        crate::safe_save::replace_if_unchanged(self, destination.as_ref())
    }

    pub(crate) fn original_bytes(&self) -> &[u8] { &self.original }
}

fn escape_xml_text(text: &str) -> String {
    text.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}

/// Byte range of the complete text contents of the sole direct root title.
/// Event offsets come from quick-xml's reader, rather than searching for
/// string patterns that might appear in comments or nested payloads.
fn direct_title_range(bytes: &[u8]) -> Result<(usize, usize), ParseError> {
    let mut reader = Reader::from_reader(bytes);
    let mut stack: Vec<Vec<u8>> = Vec::new();
    let mut found = None;
    let mut text_start = None;
    loop {
        let before = reader.buffer_position() as usize;
        let event = reader.read_event().map_err(|e| ParseError(e.to_string()))?;
        let after = reader.buffer_position() as usize;
        match event {
            Event::Start(e) => {
                if stack.len() == 2 && stack[0] == b"Minsky" && stack[1] == b"title" {
                    return Err(ParseError("title contains nested elements; edit refused".into()));
                }
                let name = e.local_name().as_ref().to_vec();
                if stack.len() == 1 && stack[0] == b"Minsky" && name == b"title" {
                    if found.is_some() || text_start.is_some() { return Err(ParseError("duplicate root title".into())); }
                    text_start = Some(after);
                }
                stack.push(name);
            }
            Event::Empty(e) => {
                let name = e.local_name().as_ref().to_vec();
                if stack.len() == 1 && stack[0] == b"Minsky" && name == b"title" {
                    return Err(ParseError("self-closing title edit is unsupported".into()));
                }
                if stack.len() == 2 && stack[0] == b"Minsky" && stack[1] == b"title" {
                    return Err(ParseError("title contains child elements; edit refused".into()));
                }
            }
            Event::End(e) => {
                let name = e.local_name().as_ref().to_vec();
                if stack.last().map(Vec::as_slice) != Some(name.as_slice()) {
                    return Err(ParseError("invalid XML nesting".into()));
                }
                if stack.len() == 2 && stack[0] == b"Minsky" && stack[1] == b"title" {
                    let start = text_start.take().ok_or_else(|| ParseError("invalid title".into()))?;
                    if found.replace((start, before)).is_some() {
                        return Err(ParseError("duplicate root title".into()));
                    }
                }
                stack.pop();
            }
            Event::CData(_) | Event::Comment(_) | Event::PI(_) if stack.len() == 2 && stack[0] == b"Minsky" && stack[1] == b"title" => {
                return Err(ParseError("title has non-text content; edit refused".into()));
            }
            Event::Eof => break,
            _ => {}
        }
    }
    found.ok_or_else(|| ParseError("missing root title; edit refused".into()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn minimal_schema3() {
        let xml = r#"<Minsky><schemaVersion>3</schemaVersion><title>Model</title><items><Item><id>4</id><type>Variable:flow</type><ports><int>8</int></ports></Item></items><wires><Wire><id>10</id><from>8</from><to>9</to></Wire></wires><groups><Group><id>12</id><items><int>4</int></items></Group></groups></Minsky>"#;
        let model = inspect_xml(xml).unwrap();
        assert_eq!(model.items[0].id, Some(4));
        assert_eq!(model.items[0].ports, vec![8]);
        assert_eq!(model.wires[0].from, Some(8));
        assert_eq!(model.groups[0].child_items, vec![4]);
    }

    #[test]
    fn rejects_unsupported_schema() {
        assert!(inspect_xml("<Minsky><schemaVersion>2</schemaVersion></Minsky>").is_err());
    }

    #[test]
    fn rejects_broken_xml() {
        assert!(inspect_xml("<Minsky><schemaVersion>3</schemaVersion>").is_err());
    }
}

/// Read-only specialized Minsky schema-3 metadata.
pub mod metadata;

/// Read-only tensor CDATA envelope decoding (not classdesc unpacking).
pub mod tensor_payload;

pub mod tensor_binary;

//! Read-only inspection of the top-level Minsky schema-3 XML structure.
//! This is NOT a complete deserializer and must not be used for saving files.

use quick_xml::{events::Event, Reader};
use std::{error::Error, fmt, fs, path::Path};

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

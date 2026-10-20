//! Read-only schema-2 structural migration preview. Not a serializer.
//! Only a strict subset of schema-2 fields is accepted; unsupported fields
//! cause a hard error rather than silently discarding semantics.
use crate::{inspect_xml, ModelSummary, ParseError};
use crate::schema_probe::{probe_schema, SchemaGeneration};
use quick_xml::{events::Event, Reader};

/// A structural view converted from the explicitly supported schema-2 subset.
/// `source_xml` remains the authoritative original; this type MUST NOT be saved as schema 3.
#[derive(Debug)]
pub struct Schema2Preview {
    pub model: ModelSummary,
    pub source_xml: String,
}

fn allowed(path: &[String], name: &str) -> bool {
    match path {
        [] => name == "Minsky",
        [root] if root == "Minsky" => matches!(name,
            "schemaVersion" | "minskyVersion" | "author" | "title" |
            "description" | "items" | "wires" | "groups" | "zoomFactor"),
        [root, col] if root == "Minsky" => match col.as_str() {
            "items" => name == "Item",
            "wires" => name == "Wire",
            "groups" => name == "Group",
            _ => false,
        },
        [root, col, item] if root == "Minsky" &&
            ((col == "items" && item == "Item") ||
             (col == "groups" && item == "Group")) => matches!(name,
            "id" | "type" | "x" | "y" | "rotation" | "zoomFactor" | "ports" |
            "width" | "height" | "name" | "init" | "units" | "items" |
            "inVariables" | "outVariables"),
        [root, col, wire] if root == "Minsky" && col == "wires" && wire == "Wire" =>
            matches!(name, "id" | "from" | "to" | "coords"),
        [root, col, item, list] if root == "Minsky" &&
            ((col == "items" && item == "Item") ||
             (col == "groups" && item == "Group")) &&
            matches!(list.as_str(), "ports" | "items" | "inVariables" | "outVariables") => name == "int",
        [root, col, wire, list] if root == "Minsky" && col == "wires" && wire == "Wire" && list == "coords" => name == "float",
        _ => false,
    }
}

/// Convert known structural fields to an in-memory schema-3 summary, without
/// writing or modifying XML. The source is retained verbatim.
/// This is intentionally NOT general schema-2 migration. Item zoomFactor is
/// validated but not represented in ModelSummary; original XML retains it.
pub fn preview_schema2(xml: &str) -> Result<Schema2Preview, ParseError> {
    if probe_schema(xml)? != SchemaGeneration::V2 {
        return Err(ParseError("expected schema version 2".into()));
    }
    let mut reader = Reader::from_str(xml);
    let mut path: Vec<String> = Vec::new();
    let mut version_text = false;
    loop {
        match reader.read_event().map_err(|e| ParseError(e.to_string()))? {
            Event::Start(e) => {
                let name = String::from_utf8_lossy(e.local_name().as_ref()).into_owned();
                if !allowed(&path, &name) {
                    return Err(ParseError(format!("unsupported schema-2 field: {}/{}", path.join("/"), name)));
                }
                // Namespace declarations on the root are allowed; other attributes are not.
                for attr in e.attributes() {
                    let attr = attr.map_err(|e| ParseError(e.to_string()))?;
                    if !(path.is_empty() && attr.key.as_ref().starts_with(b"xmlns")) {
                        return Err(ParseError(format!("unsupported XML attribute on {name}")));
                    }
                }
                version_text = path == ["Minsky"] && name == "schemaVersion";
                path.push(name);
            }
            Event::Empty(e) => {
                let name = String::from_utf8_lossy(e.local_name().as_ref()).into_owned();
                if !allowed(&path, &name) { return Err(ParseError(format!("unsupported schema-2 field: {name}"))); }
                if e.attributes().next().is_some() { return Err(ParseError("XML attributes not supported".into())); }
            }
            Event::End(_) => { path.pop().ok_or_else(|| ParseError("unbalanced XML".into()))?; version_text = false; }
            Event::Text(e) if version_text => {
                let text = e.unescape().map_err(|e| ParseError(e.to_string()))?;
                if text.trim() != "2" { return Err(ParseError("expected schema version 2".into())); }
            }
            Event::Text(e) if path.last().map(String::as_str) == Some("zoomFactor") => {
                let text = e.unescape().map_err(|e| ParseError(e.to_string()))?;
                text.trim().parse::<f32>().map_err(|_| ParseError("invalid schema-2 zoomFactor".into()))?;
            }
            Event::Eof => break,
            _ => {}
        }
    }
    // Structural inspection is performed against a temporary in-memory copy.
    // This is NOT valid schema-3 serialization: schema-2 defaults differ.
    let needle = "<schemaVersion>2</schemaVersion>";
    if !xml.contains(needle) { return Err(ParseError("schema-2 version must use the simple XML element form".into())); }
    let normalized = xml.replacen(needle, "<schemaVersion>3</schemaVersion>", 1);
    let model = inspect_xml(&normalized)?;
    Ok(Schema2Preview { model, source_xml: xml.to_string() })
}

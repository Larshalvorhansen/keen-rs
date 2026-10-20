//! Read-only schema generation identification. Never migrates or writes legacy files.
use quick_xml::{events::Event, Reader};
use crate::ParseError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchemaGeneration { LegacyUnversioned, V1, V2, V3, Unsupported(u32) }

impl SchemaGeneration {
    pub fn can_inspect_with_current_parser(self) -> bool { self == Self::V3 }
}

/// Detect the declared version on a well-formed `<Minsky>` XML document.
/// No version is classified as *unversioned*, not automatically as schema 0.
/// An unversioned model requires upstream migration inspection before conversion.
pub fn probe_schema(xml: &str) -> Result<SchemaGeneration, ParseError> {
    let mut reader = Reader::from_str(xml);
    let mut depth = 0usize;
    let mut root_seen = false;
    let mut finished = false;
    let mut version: Option<u32> = None;
    let mut in_version = false;
    loop {
        match reader.read_event().map_err(|e| ParseError(e.to_string()))? {
            Event::Start(e) => {
                if depth == 0 {
                    if root_seen || e.local_name().as_ref() != b"Minsky" { return Err(ParseError("expected one <Minsky> root".into())); }
                    root_seen = true;
                } else if depth == 1 && e.local_name().as_ref() == b"schemaVersion" {
                    if version.is_some() || in_version { return Err(ParseError("duplicate schemaVersion".into())); }
                    in_version = true;
                }
                depth += 1;
            }
            Event::Empty(e) => {
                if depth == 0 || (depth == 1 && e.local_name().as_ref() == b"schemaVersion") {
                    return Err(ParseError("invalid empty root or schemaVersion".into()));
                }
            }
            Event::Text(e) if in_version && depth == 2 => {
                let value = e.unescape().map_err(|e| ParseError(e.to_string()))?;
                if version.is_some() { return Err(ParseError("duplicate schemaVersion text".into())); }
                version = Some(value.trim().parse().map_err(|_| ParseError("invalid schemaVersion".into()))?);
            }
            Event::End(e) => {
                if depth == 0 { return Err(ParseError("unmatched closing tag".into())); }
                if depth == 2 && e.local_name().as_ref() == b"schemaVersion" { in_version = false; }
                depth -= 1;
                if depth == 0 { finished = true; }
            }
            Event::Eof => break,
            _ => {}
        }
    }
    if !root_seen || depth != 0 || !finished || in_version { return Err(ParseError("incomplete Minsky XML".into())); }
    Ok(match version { None => SchemaGeneration::LegacyUnversioned, Some(1) => SchemaGeneration::V1, Some(2) => SchemaGeneration::V2, Some(3) => SchemaGeneration::V3, Some(v) => SchemaGeneration::Unsupported(v) })
}

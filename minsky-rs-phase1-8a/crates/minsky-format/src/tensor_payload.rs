//! TensorData envelope from schemaHelper.cc: ASCII85 with `~` standing in
//! for `]`, followed by zlib, then classdesc::pack_t bytes.
//! This module does NOT interpret the classdesc binary layout.
use crate::ParseError;
use flate2::read::ZlibDecoder;
use quick_xml::{events::Event, Reader};
use std::io::Read;

const MAX_PACKED_BYTES: usize = 64 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TensorPayload {
    pub item_id: Option<i64>,
    pub encoded_bytes: usize,
    pub unpacked_bytes: usize,
    /// Decompressed classdesc::pack_t bytes; numerical values and axes
    /// cannot yet be interpreted without a verified classdesc layout.
    pub packed_data: Vec<u8>,
}

pub fn inspect_tensor_payloads(xml: &str) -> Result<Vec<TensorPayload>, ParseError> {
    crate::inspect_xml(xml)?;
    let mut reader = Reader::from_str(xml);
    let mut path: Vec<String> = Vec::new();
    let mut item_id = None;
    let mut payloads = Vec::new();
    let mut value = String::new();
    loop {
        match reader.read_event().map_err(|e| ParseError(e.to_string()))? {
            Event::Start(e) => {
                let name = String::from_utf8_lossy(e.local_name().as_ref()).into_owned();
                if path == ["Minsky", "items"] && name == "Item" { item_id = None; }
                path.push(name);
                value.clear();
            }
            Event::Text(e) => value.push_str(&e.unescape().map_err(|e| ParseError(e.to_string()))?),
            Event::CData(e) => value.push_str(&String::from_utf8_lossy(e.as_ref())),
            Event::End(e) => {
                let name = String::from_utf8_lossy(e.local_name().as_ref()).into_owned();
                if path.last().map(String::as_str) != Some(name.as_ref()) { return Err(ParseError("mismatched XML end tag".into())); }
                if path == ["Minsky", "items", "Item", "id"] { item_id = Some(value.trim().parse::<i64>().map_err(|_| ParseError("bad tensor item ID".into()))?); }
                if path == ["Minsky", "items", "Item", "tensorData"] && !value.trim().is_empty() {
                    let decoded = decode_envelope(&value)?;
                    payloads.push(TensorPayload { item_id, encoded_bytes: value.len(), unpacked_bytes: decoded.len(), packed_data: decoded });
                }
                path.pop(); value.clear();
            }
            Event::Eof => break,
            _ => {}
        }
    }
    Ok(payloads)
}

/// Bounded ASCII85 + zlib decode, implementing the upstream outer envelope.
pub fn decode_envelope(encoded: &str) -> Result<Vec<u8>, ParseError> {
    let cleaned: Vec<u8> = encoded.bytes().filter(|b| !b.is_ascii_whitespace()).map(|b| if b == b'~' { b']' } else { b }).collect();
    if cleaned.len() > MAX_PACKED_BYTES * 2 { return Err(ParseError("tensor payload exceeds encoded size limit".into())); }
    let mut compressed = Vec::with_capacity(cleaned.len() * 4 / 5);
    for group in cleaned.chunks(5) {
        if group.len() == 1 { return Err(ParseError("invalid final ASCII85 group".into())); }
        let mut digits = [84u32; 5];
        for (i, &byte) in group.iter().enumerate() {
            if !(33..=117).contains(&byte) { return Err(ParseError("invalid ASCII85 character".into())); }
            digits[i] = u32::from(byte - 33);
        }
        let mut n: u64 = 0;
        for d in digits { n = n * 85 + u64::from(d); }
        if n > u64::from(u32::MAX) { return Err(ParseError("ASCII85 word overflow".into())); }
        compressed.extend_from_slice(&(n as u32).to_be_bytes()[..if group.len() == 5 { 4 } else { group.len()-1 }]);
    }
    let mut decoder = ZlibDecoder::new(compressed.as_slice());
    let mut output = Vec::new();
    decoder.by_ref().take((MAX_PACKED_BYTES + 1) as u64).read_to_end(&mut output).map_err(|e| ParseError(format!("zlib tensor decode: {e}")))?;
    if output.len() > MAX_PACKED_BYTES { return Err(ParseError("tensor payload exceeds decoded size limit".into())); }
    Ok(output)
}

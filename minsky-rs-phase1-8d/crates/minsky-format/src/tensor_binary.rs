//! Read-only schema3::TensorVal unpacker.
//! The byte order and primitive layout are verified against importedCSV.mky
//! and schema/schema3.cc. Does not modify or re-encode .mky XML.
use crate::ParseError;

const MAX_ELEMENTS: usize = 8_000_000;
const MAX_AXES: usize = 128;
const MAX_LABELS: usize = 2_000_000;

#[derive(Debug, Clone, PartialEq)]
pub struct TensorAxis {
    pub name: String,
    pub dimension_type: u32,
    pub units: String,
    pub labels: Vec<String>,
}
#[derive(Debug, Clone, PartialEq)]
pub struct DecodedTensor {
    /// Stored values in schema3 serialization order; can be sparse.
    pub values: Vec<f64>,
    /// Sparse index positions. Empty means dense storage.
    pub index: Vec<u64>,
    pub axes: Vec<TensorAxis>,
}
impl DecodedTensor {
    pub fn shape(&self) -> Vec<usize> { self.axes.iter().map(|a| a.labels.len()).collect() }
}

struct Cursor<'a> { bytes: &'a [u8], at: usize }
impl<'a> Cursor<'a> {
    fn take(&mut self, n: usize) -> Result<&'a [u8], ParseError> {
        let end = self.at.checked_add(n).ok_or_else(|| ParseError("tensor offset overflow".into()))?;
        let data = self.bytes.get(self.at..end).ok_or_else(|| ParseError("truncated tensor binary".into()))?;
        self.at = end;
        Ok(data)
    }
    fn u64(&mut self) -> Result<u64, ParseError> { Ok(u64::from_le_bytes(self.take(8)?.try_into().unwrap())) }
    fn u32(&mut self) -> Result<u32, ParseError> { Ok(u32::from_le_bytes(self.take(4)?.try_into().unwrap())) }
    fn usize_count(&mut self, max: usize, field: &str) -> Result<usize, ParseError> {
        let n = usize::try_from(self.u64()?).map_err(|_| ParseError(format!("{field} count overflow")))?;
        if n > max { return Err(ParseError(format!("{field} count exceeds limit"))); }
        Ok(n)
    }
    fn string(&mut self) -> Result<String, ParseError> {
        let n = self.usize_count(1024 * 1024, "string")?;
        String::from_utf8(self.take(n)?.to_vec()).map_err(|e| ParseError(format!("invalid tensor UTF-8: {e}")))
    }
}

/// Unpack the explicit schema3::unpack layout. This is verified for little-endian
/// classdesc binary data from pinned upstream fixture `importedCSV.mky`.
/// Other primitive formats and dimension kinds require additional upstream fixtures.
pub fn decode_tensor_binary(bytes: &[u8]) -> Result<DecodedTensor, ParseError> {
    let mut c = Cursor { bytes, at: 0 };
    let count = c.usize_count(MAX_ELEMENTS, "values")?;
    let mut values = Vec::with_capacity(count);
    for _ in 0..count { values.push(f64::from_bits(c.u64()?)); }
    let index_count = c.usize_count(MAX_ELEMENTS, "index")?;
    let mut index = Vec::with_capacity(index_count);
    for _ in 0..index_count { index.push(c.u64()?); }
    if !index.is_empty() && index.len() != values.len() {
        return Err(ParseError("sparse index length differs from values".into()));
    }
    if index.windows(2).any(|w| w[0] >= w[1]) {
        return Err(ParseError("sparse tensor indexes are not strictly increasing".into()));
    }
    let axis_count = c.usize_count(MAX_AXES, "axis")?;
    let mut axes = Vec::with_capacity(axis_count);
    let mut shape_product = 1usize;
    for _ in 0..axis_count {
        let name = c.string()?;
        let dimension_type = c.u32()?;
        if dimension_type > 2 { return Err(ParseError(format!("unsupported dimension type {dimension_type}"))); }
        let units = c.string()?;
        let label_count = c.usize_count(MAX_LABELS, "axis label")?;
        shape_product = shape_product.checked_mul(label_count).ok_or_else(|| ParseError("shape overflow".into()))?;
        let mut labels = Vec::with_capacity(label_count);
        for _ in 0..label_count { labels.push(c.string()?); }
        axes.push(TensorAxis { name, dimension_type, units, labels });
    }
    if c.at != bytes.len() { return Err(ParseError("trailing tensor binary data".into())); }
    if axes.is_empty() { return Err(ParseError("tensor binary has no axes".into())); }
    if index.is_empty() && values.len() != shape_product {
        return Err(ParseError("dense tensor values do not match shape".into()));
    }
    if index.iter().any(|&i| i >= shape_product as u64) {
        return Err(ParseError("sparse index exceeds tensor shape".into()));
    }
    Ok(DecodedTensor { values, index, axes })
}

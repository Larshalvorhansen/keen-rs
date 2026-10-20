//! Conservative, same-directory atomic file writes for schema-3 XML.
//! Only supports byte-preserving documents and the already supported title edit.
use crate::{inspect_xml, validate_structure, ParseError, PreservedDocument};
use std::{fs::{self, File, OpenOptions}, io::{self, Write}, path::{Path, PathBuf}, sync::atomic::{AtomicU64, Ordering}};

static NEXT_TEMP: AtomicU64 = AtomicU64::new(0);

struct TempFile(PathBuf);
impl Drop for TempFile {
    fn drop(&mut self) { let _ = fs::remove_file(&self.0); }
}

fn error(context: &str, err: impl std::fmt::Display) -> ParseError {
    ParseError(format!("{context}: {err}"))
}

fn check_document(doc: &PreservedDocument) -> Result<(), ParseError> {
    let xml = std::str::from_utf8(doc.as_bytes()).map_err(|e| error("invalid UTF-8", e))?;
    let parsed = inspect_xml(xml)?;
    let problems = validate_structure(&parsed);
    if !problems.is_empty() {
        return Err(ParseError(format!("structural validation failed: {}", problems.iter().map(|p| p.message.as_str()).collect::<Vec<_>>().join("; "))));
    }
    Ok(())
}

fn temporary(destination: &Path) -> Result<(TempFile, File), ParseError> {
    let parent = destination.parent().filter(|p| !p.as_os_str().is_empty()).unwrap_or(Path::new("."));
    let name = destination.file_name().ok_or_else(|| ParseError("destination has no file name".into()))?;
    for _ in 0..128 {
        let id = NEXT_TEMP.fetch_add(1, Ordering::Relaxed);
        let mut filename = std::ffi::OsString::from(".");
        filename.push(name);
        filename.push(format!(".minsky-tmp-{}-{id}", std::process::id()));
        let path = parent.join(filename);
        match OpenOptions::new().write(true).create_new(true).open(&path) {
            Ok(file) => return Ok((TempFile(path), file)),
            Err(err) if err.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(err) => return Err(error("cannot create temporary file", err)),
        }
    }
    Err(ParseError("cannot allocate temporary file name".into()))
}

fn prepared(doc: &PreservedDocument, destination: &Path) -> Result<TempFile, ParseError> {
    check_document(doc)?;
    let (guard, mut file) = temporary(destination)?;
    file.write_all(doc.as_bytes()).map_err(|e| error("write failed", e))?;
    file.sync_all().map_err(|e| error("file sync failed", e))?;
    drop(file);
    // Check the exact bytes on disk, not just the in-memory buffer.
    let disk = fs::read(&guard.0).map_err(|e| error("cannot reread temporary file", e))?;
    if disk != doc.as_bytes() { return Err(ParseError("temporary file failed byte comparison".into())); }
    let parsed = inspect_xml(std::str::from_utf8(&disk).map_err(|e| error("invalid UTF-8", e))?)?;
    if !validate_structure(&parsed).is_empty() { return Err(ParseError("temporary file failed structural validation".into())); }
    Ok(guard)
}

fn sync_parent(destination: &Path) -> Result<(), ParseError> {
    let parent = destination.parent().filter(|p| !p.as_os_str().is_empty()).unwrap_or(Path::new("."));
    File::open(parent).and_then(|f| f.sync_all()).map_err(|e| error("directory sync failed (file may already be saved)", e))
}

/// Atomic creation without overwrite: a hard link fails if destination exists.
/// Temporary file resides in the destination directory (same filesystem).
pub fn save_new(doc: &PreservedDocument, destination: &Path) -> Result<(), ParseError> {
    let guard = prepared(doc, destination)?;
    fs::hard_link(&guard.0, destination).map_err(|e| error("cannot create destination (may already exist)", e))?;
    sync_parent(destination)
}

/// Atomic replacement after checking the original bytes. This is *not* a
/// compare-and-swap and must not be used with concurrent writers.
pub fn replace_if_unchanged(doc: &PreservedDocument, destination: &Path) -> Result<(), ParseError> {
    let meta = fs::symlink_metadata(destination).map_err(|e| error("cannot stat destination", e))?;
    if !meta.file_type().is_file() || meta.file_type().is_symlink() {
        return Err(ParseError("replacement requires a regular, non-hard-linked file".into()));
    }
    let disk = fs::read(destination).map_err(|e| error("cannot read existing file", e))?;
    if disk != doc.original_bytes() { return Err(ParseError("destination changed since opening; replacement refused".into())); }
    let guard = prepared(doc, destination)?;
    // Recheck just before rename. Another process could race after this check.
    let latest = fs::read(destination).map_err(|e| error("cannot recheck existing file", e))?;
    if latest != doc.original_bytes() { return Err(ParseError("destination changed before replacement; replacement refused".into())); }
    fs::rename(&guard.0, destination).map_err(|e| error("atomic rename failed", e))?;
    sync_parent(destination)
}

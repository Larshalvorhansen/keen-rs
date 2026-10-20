use minsky_format::PreservedDocument;
use std::{fs, path::PathBuf, sync::atomic::{AtomicU64, Ordering}};
static ID: AtomicU64 = AtomicU64::new(0);
struct TestDir(PathBuf);
impl TestDir {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!("minsky-save-test-{}-{}", std::process::id(), ID.fetch_add(1,Ordering::Relaxed)));
        fs::create_dir(&path).unwrap(); Self(path)
    }
    fn file(&self,name: &str) -> PathBuf { self.0.join(name) }
}
impl Drop for TestDir { fn drop(&mut self) {let _ = fs::remove_dir_all(&self.0);} }
fn upstream() -> Vec<u8> { fs::read(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/upstream/exponentialGrowth.mky")).unwrap() }

#[test]
fn unchanged_new_save_is_byte_identical() {
    let dir=TestDir::new(); let input=upstream(); let doc=PreservedDocument::from_bytes(input.clone()).unwrap();
    let path=dir.file("untouched.mky"); doc.write_new(&path).unwrap();
    assert_eq!(fs::read(path).unwrap(), input);
}
#[test]
fn title_save_keeps_other_bytes_and_does_not_overwrite() {
    let dir=TestDir::new(); let mut doc=PreservedDocument::from_bytes(upstream()).unwrap();
    doc.set_title("New & <title>").unwrap();
    let path=dir.file("edited.mky"); doc.write_new(&path).unwrap();
    assert_eq!(fs::read(&path).unwrap(), doc.as_bytes());
    assert!(doc.write_new(&path).is_err());
    assert_eq!(fs::read(&path).unwrap(), doc.as_bytes());
}
#[test]
fn replacement_rejects_external_edits() {
    let dir=TestDir::new(); let original=upstream(); let path=dir.file("model.mky"); fs::write(&path,&original).unwrap();
    let mut doc=PreservedDocument::from_bytes(original).unwrap(); doc.set_title("Edited").unwrap();
    fs::write(&path,b"external change").unwrap();
    assert!(doc.replace_if_unchanged(&path).is_err());
    assert_eq!(fs::read(&path).unwrap(), b"external change");
}
#[test]
fn replacement_preserves_original_on_success() {
    let dir=TestDir::new(); let original=upstream(); let path=dir.file("model.mky"); fs::write(&path,&original).unwrap();
    let mut doc=PreservedDocument::from_bytes(original).unwrap(); doc.set_title("Edited").unwrap();
    doc.replace_if_unchanged(&path).unwrap();
    assert_eq!(fs::read(&path).unwrap(), doc.as_bytes());
}
#[test]
fn invalid_model_cannot_be_saved() {
    let dir=TestDir::new();
    let xml=b"<Minsky><schemaVersion>3</schemaVersion><title>X</title><wires><Wire><id>1</id><from>777</from><to>778</to></Wire></wires></Minsky>";
    let doc=PreservedDocument::from_bytes(xml.to_vec()).unwrap();
    let path=dir.file("bad.mky"); assert!(doc.write_new(&path).is_err()); assert!(!path.exists());
}

use minsky_format::PreservedDocument;
use std::path::PathBuf;

fn fixture(name: &str) -> Vec<u8> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/upstream").join(name);
    std::fs::read(path).unwrap()
}

#[test]
fn untouched_upstream_is_byte_identical() {
    for name in ["exponentialGrowth.mky", "indexing.mky"] {
        let bytes = fixture(name);
        let doc = PreservedDocument::from_bytes(bytes.clone()).unwrap();
        assert_eq!(doc.as_bytes(), bytes);
        assert!(!doc.is_modified());
        assert!(doc.changed_fields().is_empty());
    }
}

#[test]
fn title_edit_preserves_unrelated_xml_bytes() {
    let input = b"<Minsky><schemaVersion>3</schemaVersion><title>Old &amp; title</title><unknown foo=\"1\"><![CDATA[payload]]></unknown><items/></Minsky>";
    let mut doc = PreservedDocument::from_bytes(input.to_vec()).unwrap();
    doc.set_title("New <title> & data").unwrap();
    let output = std::str::from_utf8(doc.as_bytes()).unwrap();
    assert_eq!(output, "<Minsky><schemaVersion>3</schemaVersion><title>New &lt;title&gt; &amp; data</title><unknown foo=\"1\"><![CDATA[payload]]></unknown><items/></Minsky>");
    assert_eq!(doc.inspect().unwrap().title, "New <title> & data");
    assert_eq!(doc.changed_fields(), vec!["title"]);
}

#[test]
fn refuses_unsafe_title_structure_without_mutating() {
    let input = b"<Minsky><schemaVersion>3</schemaVersion><title><nested/></title></Minsky>";
    let mut doc = PreservedDocument::from_bytes(input.to_vec()).unwrap();
    assert!(doc.set_title("Replacement").is_err());
    assert_eq!(doc.as_bytes(), input);
}

#[test]
fn edit_real_upstream_title_and_keep_payload_intact() {
    let bytes = fixture("indexing.mky");
    let mut doc = PreservedDocument::from_bytes(bytes.clone()).unwrap();
    doc.set_title("Indexing & preserving").unwrap();
    assert_eq!(doc.inspect().unwrap().title, "Indexing & preserving");
    assert!(doc.as_bytes().windows(b"Indexing &amp; preserving".len()).any(|w| w == b"Indexing &amp; preserving"));
    assert!(doc.is_modified());
}

use minsky_format::{tensor_payload::inspect_tensor_payloads, tensor_binary::decode_tensor_binary};

#[test]
fn decodes_dense_and_sparse_upstream_tensors() {
    let xml = include_str!("../../../tests/fixtures/upstream/importedCSV.mky");
    let payloads = inspect_tensor_payloads(xml).unwrap();
    assert_eq!(payloads.len(), 2);
    let dense = decode_tensor_binary(&payloads[0].packed_data).unwrap();
    assert_eq!(dense.shape(), vec![3, 3]);
    assert_eq!(dense.values, vec![1.,4.,7.,2.,5.,8.,3.,6.,9.]);
    assert!(dense.index.is_empty());
    assert_eq!(dense.axes[0].name, "x");
    assert_eq!(dense.axes[1].name, "y");
    assert_eq!(dense.axes[0].labels, vec!["0", "1", "2"]);
    let sparse = decode_tensor_binary(&payloads[1].packed_data).unwrap();
    assert_eq!(sparse.shape(), vec![3, 3]);
    assert_eq!(sparse.values, vec![1., 2., 3.]);
    assert_eq!(sparse.index, vec![0,4,8]);
}

#[test]
fn malformed_binary_never_panics() {
    assert!(decode_tensor_binary(&[]).is_err());
    assert!(decode_tensor_binary(&[1,0,0,0,0,0,0,0]).is_err());
    let xml = include_str!("../../../tests/fixtures/upstream/importedCSV.mky");
    let payloads = inspect_tensor_payloads(xml).unwrap();
    let mut invalid = payloads[0].packed_data.clone();
    invalid.extend_from_slice(&[0]);
    assert!(decode_tensor_binary(&invalid).is_err());
}

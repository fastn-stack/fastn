//! Test Protocol::Generic handling

#[test]
fn test_protocol_generic_equality() {
    // Test if Protocol::Generic values compare properly
    let json1 = serde_json::json!("Echo");
    let json2 = serde_json::json!("Echo");

    let proto1 = fastn_net::Protocol::Generic(json1);
    let proto2 = fastn_net::Protocol::Generic(json2);

    println!("proto1 = {proto1:?}");
    println!("proto2 = {proto2:?}");

    assert_eq!(
        proto1, proto2,
        "Protocol::Generic should be equal for same JSON values"
    );

    let expected = [proto1.clone()];
    assert!(
        expected.contains(&proto2),
        "contains should work for Protocol::Generic"
    );

    println!("✅ Protocol::Generic equality test passed");
}

#[test]
fn test_protocol_generic_serialization() {
    // Test round-trip serialization
    let original = fastn_net::Protocol::Generic(serde_json::json!("Echo"));

    let serialized = serde_json::to_string(&original).unwrap();
    println!("Serialized: {serialized}");

    let deserialized: fastn_net::Protocol = serde_json::from_str(&serialized).unwrap();

    assert_eq!(
        original, deserialized,
        "Protocol::Generic should round-trip serialize correctly"
    );

    println!("✅ Protocol::Generic serialization test passed");
}

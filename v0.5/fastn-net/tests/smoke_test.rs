//! Basic smoke tests for fastn-net

#[test]
fn test_secret_key_generation() {
    let (id52, secret_key) = fastn_net::generate_secret_key().expect("Should generate key");
    assert_eq!(id52.len(), 52, "ID52 should be 52 characters");

    // Verify the generated ID52 matches the public key
    let public_key = secret_key.public_key();
    assert_eq!(public_key.to_string(), id52);
}

#[test]
fn test_protocol_serialization() {
    use fastn_net::Protocol;

    let protocols = vec![
        Protocol::Ping,
        Protocol::WhatTimeIsIt,
        Protocol::Http,
        Protocol::HttpProxy,
        Protocol::Socks5,
        Protocol::Tcp,
    ];

    for protocol in protocols {
        let json = serde_json::to_string(&protocol).expect("Should serialize");
        let deserialized: Protocol = serde_json::from_str(&json).expect("Should deserialize");
        assert_eq!(protocol, deserialized);
    }
}

#[test]
fn test_ack_constant() {
    assert_eq!(fastn_net::ACK, "ack");
}

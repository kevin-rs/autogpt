use iac_rs::prelude::*;

#[test]
fn test_serialize_deserialize_sign_verify() -> Result<()> {
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::DEBUG)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let keypair = KeyPair::generate();
    let signer = Signer::new(keypair.clone());
    let verifier = Verifier::new(vec![signer.verifying_key()]);

    let mut msg = Message {
        msg_id: 1,
        from: "client".to_string(),
        to: "server".to_string(),
        signature: vec![],
        ..Default::default()
    };

    let serialized = msg.serialize()?;
    assert!(!serialized.is_empty());

    let deserialized = Message::deserialize(&serialized)?;
    assert_eq!(deserialized.msg_id, msg.msg_id);

    msg.sign(&signer)?;
    assert!(!msg.signature.is_empty());

    msg.verify(&verifier)?;

    let mut bad_msg = msg.clone();
    bad_msg.signature[0] ^= 0xff;
    assert!(bad_msg.verify(&verifier).is_err());

    Ok(())
}

#[test]
fn test_ping_message() -> Result<()> {
    let keypair = KeyPair::generate();
    let signer = Signer::new(keypair.clone());
    let verifier = Verifier::new(vec![signer.verifying_key()]);

    let mut msg = Message {
        msg_id: 1,
        from: "nodeA".to_string(),
        to: "nodeB".to_string(),
        msg_type: MessageType::Ping,
        payload_json: r#"{"ping": true}"#.to_string(),
        timestamp: 123456,
        session_id: 999,
        signature: vec![],
        ..Default::default()
    };

    let serialized = msg.serialize()?;
    assert!(!serialized.is_empty());

    let deserialized = Message::deserialize(&serialized)?;
    assert_eq!(deserialized.msg_type, MessageType::Ping);

    msg.sign(&signer)?;
    assert!(!msg.signature.is_empty());

    msg.verify(&verifier)?;

    let mut tampered = msg.clone();
    tampered.signature[0] ^= 0xAA;
    assert!(tampered.verify(&verifier).is_err());

    Ok(())
}

#[test]
fn test_broadcast_message() -> Result<()> {
    let keypair = KeyPair::generate();
    let signer = Signer::new(keypair.clone());
    let verifier = Verifier::new(vec![signer.verifying_key()]);

    let mut msg = Message {
        msg_id: 100,
        from: "controller".to_string(),
        to: "all".to_string(),
        msg_type: MessageType::Broadcast,
        payload_json: r#"{"announcement": "Ferris is King!"}"#.to_string(),
        timestamp: 98765431,
        session_id: 123,
        signature: vec![],
        ..Default::default()
    };

    let serialized = msg.serialize()?;
    assert!(!serialized.is_empty());

    let deserialized = Message::deserialize(&serialized)?;
    assert_eq!(deserialized.msg_type, MessageType::Broadcast);

    msg.sign(&signer)?;
    assert!(!msg.signature.is_empty());

    msg.verify(&verifier)?;

    let mut modified = msg.clone();
    modified.signature[0] ^= 0xFF;
    assert!(modified.verify(&verifier).is_err());

    Ok(())
}

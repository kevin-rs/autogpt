use iac_rs::prelude::*;

#[test]
fn test_serialize_deserialize_sign_verify() -> Result<()> {
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::DEBUG)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let keypair = KeyPair::generate();
    let signer = Signer::new(keypair.clone());
    let verifier = Verifier::new(signer.verifying_key());

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

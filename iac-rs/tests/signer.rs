use iac_rs::prelude::*;

#[test]
fn test_sign_and_verify() -> Result<()> {
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::DEBUG)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let keypair = KeyPair::generate();
    let signer = Signer::new(keypair.clone());
    let verifier = Verifier::new(signer.verifying_key());

    let data = b"test data";

    let signature = signer.sign(data)?;
    assert_eq!(signature.len(), 64);

    verifier.verify(data, &signature)?;

    let bad_data = b"bad data";
    assert!(verifier.verify(bad_data, &signature).is_err());

    assert!(verifier.verify(data, &[0u8; 10]).is_err());

    Ok(())
}

use iac_rs::prelude::*;

#[tokio::test]
async fn test_client_connect_and_send() -> Result<()> {
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::DEBUG)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let addr = "127.0.0.1:4433";

    tokio::spawn(async move {
        let mut server = Server::bind(addr).await.unwrap();
        let verifier = Verifier::new(vec![KeyPair::generate().pk]);
        server.run(verifier).await.unwrap();
    });

    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    let signer = Signer::new(KeyPair::generate());
    let client = Client::connect(addr, signer.clone()).await?;

    let mut msg = Message {
        msg_id: 1,
        from: "tester".to_string(),
        to: "server".to_string(),
        signature: vec![],
        ..Default::default()
    };

    msg.sign(&signer)?;
    client.send(msg).await?;

    Ok(())
}

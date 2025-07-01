use iac_rs::prelude::*;

#[tokio::test]
async fn test_server_bind_and_run() -> Result<()> {
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::DEBUG)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let addr = "127.0.0.1:4433";
    let mut server = Server::bind(addr).await?;
    let verifier = Verifier::new(KeyPair::generate().pk);

    let server_task = tokio::spawn(async move {
        server.run(verifier).await.unwrap();
    });

    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    server_task.abort();

    Ok(())
}

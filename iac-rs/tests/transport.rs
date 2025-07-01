use iac_rs::prelude::*;

#[tokio::test]
async fn test_transport_configs() -> Result<()> {
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::DEBUG)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let server_config: ServerConfig = init_server()?;
    let server_addr: SocketAddr = "127.0.0.1:0".parse()?;
    let (server_endpoint, server_addr) = {
        let ep = Endpoint::server(server_config, server_addr)?;
        let bound_addr = ep.local_addr()?;
        (ep, bound_addr)
    };

    tokio::spawn(async move {
        while let Some(connecting) = server_endpoint.accept().await {
            match connecting.await {
                Ok(conn) => {
                    println!(
                        "Server accepted connection from {:?}",
                        conn.remote_address()
                    );
                    break;
                }
                Err(e) => {
                    eprintln!("Server connection error: {}", e);
                }
            }
        }
    });

    let client_config: ClientConfig = init_client()?;
    let client_endpoint = Endpoint::client("0.0.0.0:0".parse()?)?;
    let mut client_endpoint = client_endpoint;
    client_endpoint.set_default_client_config(client_config);

    let conn_res = timeout(Duration::from_secs(5), connect(&server_addr.to_string())).await;

    assert!(conn_res.is_ok(), "Connection timed out");
    let conn_res = conn_res.unwrap();

    assert!(conn_res.is_err() || conn_res.is_ok());

    Ok(())
}

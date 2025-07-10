use anyhow::Result;
use iac_rs::prelude::*;
use tracing_subscriber::{filter::LevelFilter, fmt, prelude::*, reload};

#[tokio::main]
async fn main() -> Result<()> {
    let filter = LevelFilter::DEBUG;
    let (filter, _reload_handle) = reload::Layer::new(filter);
    tracing_subscriber::registry()
        .with(filter)
        .with(fmt::Layer::default())
        .init();

    let signer = Signer::new(KeyPair::generate());
    let verifier = Verifier::new(vec![signer.verifying_key()]);

    let mut server = Server::bind("0.0.0.0:4555").await?;
    server.run(verifier).await?;

    Ok(())
}

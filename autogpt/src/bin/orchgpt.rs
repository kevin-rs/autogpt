/// The main entry point of `orchgpt`.
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    #[cfg(feature = "cli")]
    {
        use autogpt::cli::orchgpt::Cli;
        use autogpt::common::utils::setup_logging;
        use autogpt::orchestrator::Orchestrator;
        use clap::Parser;
        use iac_rs::prelude::*;
        use tracing::error;

        let _args: Cli = Cli::parse();

        setup_logging()?;

        let signer = Signer::new(KeyPair::generate());
        let verifier = Verifier::new(vec![]);

        let mut orchestrator =
            Orchestrator::new("orchestrator".to_string(), signer, verifier).await?;

        if let Err(e) = orchestrator.run().await {
            error!("Orchestrator error: {:?}", e);
        }
    }

    Ok(())
}

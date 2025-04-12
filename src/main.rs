/// The main entry point of `orchgpt`.
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    #[cfg(feature = "cli")]
    {
        use autogpt::common::utils::setup_logging;
        use autogpt::message::Message;
        use autogpt::orchcli::Cli;
        use autogpt::orchestrator::Orchestrator;
        use clap::Parser;
        use tokio::sync::mpsc;

        let _args: Cli = Cli::parse();

        setup_logging()?;

        let (tx, rx) = mpsc::channel(100);

        let orchestrator = Orchestrator::new(rx).await?;

        let msg = Message {
            from: "cli".into(),
            to: "ArchitectGPT".into(),
            msg_type: "create".into(),
            payload_json: "".into(),
            auth_token: "".into(),
        };

        tx.send(msg).await?;

        if let Err(e) = orchestrator.run().await {
            eprintln!("Orchestrator error: {:?}", e);
        }
    }

    Ok(())
}

use anyhow::Result;

/// The main entry point of `autogpt`.
///
/// It parses command-line arguments using the `clap` crate, configures agents based on
/// the provided command-line options, and performs an operation using the specified subcommand.
#[tokio::main]
async fn main() -> Result<()> {
    #[cfg(feature = "cli")]
    {
        use autogpt::common::tls::load_certs;
        use autogpt::common::utils::setup_logging;
        use autogpt::message::encode_message;
        use autogpt::message::Message;
        use colored::*;
        use rustls::pki_types::ServerName;
        use rustls::{ClientConfig, RootCertStore};
        use std::env;
        use std::io::Write;
        use std::sync::Arc;
        use tokio::io::AsyncWriteExt;
        use tokio::net::TcpStream;
        use tokio_rustls::TlsConnector;
        use tracing::{info, warn};

        setup_logging()?;
        let certs = load_certs("certs/cert.pem")?;
        info!(
            "{}",
            "[*] \"AGI\": 🌟 Welcome! What would you like to work on today?".bright_green()
        );

        let mut root = RootCertStore::empty();
        root.add_parsable_certificates(certs);

        let config = ClientConfig::builder()
            .with_root_certificates(root)
            .with_no_client_auth();

        let connector = TlsConnector::from(Arc::new(config));
        let bind_address =
            env::var("ORCHESTRATOR_ADDRESS").unwrap_or_else(|_| "0.0.0.0:8443".to_string());
        let stream = TcpStream::connect(bind_address.to_string()).await?;
        let domain = ServerName::try_from("localhost")?;
        let mut tls_stream = connector.connect(domain, stream).await?;
        loop {
            print!("> ");
            std::io::stdout().flush()?;
            let mut input = String::new();
            std::io::stdin()
                .read_line(&mut input)
                .expect("Failed to read line");

            input = input.trim().to_string();

            if input.is_empty() {
                warn!(
                    "{}",
                    "[*] \"AGI\": 🤔 You've entered an empty project description? What exactly does that entail?"
                        .bright_yellow()
                        .bold()
                );
                continue;
            }

            if !input.starts_with('/') {
                info!(
                    "{}",
                    "[*] \"AGI\": ❌ Command must begin with a '/' followed by the agent name."
                        .bright_red()
                        .bold()
                );
                continue;
            }

            if let Some((to, rest)) = input.split_once(' ') {
                if let Some((input, lang)) = rest.split_once('|') {
                    info!(
                        "{}",
                        "[*] \"AGI\": 🫡 Roger! Executing your command..."
                            .bright_yellow()
                            .bold()
                    );

                    let payload = format!("input={};language={}", input.trim(), lang.trim());

                    let msg = Message {
                        from: "cli".to_string(),
                        to: to.trim_start_matches('/').to_string(),
                        msg_type: "run".into(),
                        payload_json: payload,
                        auth_token: "secret".into(),
                    };

                    let data = encode_message(&msg)?;
                    tls_stream.write_all(&data).await?;

                    info!("{}", "[*] \"AGI\": ✅ Done!".green().bold());
                } else {
                    info!(
                        "{}",
                        "[*] \"AGI\": ❌ Invalid command format. Use: /<agent> <action> <input> | <language>"
                            .bright_red()
                            .bold()
                    );
                }
            } else {
                info!(
                    "{}",
                    "[*] \"AGI\": ❌ Invalid command format. Use: /<agent> <action> <input> | <language>"
                        .bright_red()
                        .bold()
                );
            }
        }
    }
}

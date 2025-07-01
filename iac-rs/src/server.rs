use crate::crypto::Verifier;
use crate::message::Message;
use crate::transport::init_server;
use anyhow::Result;
use quinn::Endpoint;
use tracing::{debug, error, instrument};
use zstd::stream::decode_all;

pub struct Server {
    endpoint: Endpoint,
}

impl Server {
    pub async fn bind(addr: &str) -> Result<Self> {
        let cfg = init_server()?;
        let endpoint = quinn::Endpoint::server(cfg, addr.parse()?)?;
        debug!(address = %addr, "🚀 Server bound and listening");
        Ok(Self { endpoint })
    }

    pub async fn run(&mut self, verifier: Verifier) -> Result<()> {
        while let Some(connecting) = self.endpoint.accept().await {
            let remote = connecting.remote_address().to_string();

            debug!(remote, "🔌 Incoming connection");

            let conn = connecting.await?;
            debug!(peer = %conn.remote_address(), "✅ Connection established");

            tokio::spawn(Self::handle_conn(conn, verifier.clone()));
        }

        Ok(())
    }

    #[instrument(skip(conn, verifier), fields(peer = %conn.remote_address()))]
    async fn handle_conn(conn: quinn::Connection, verifier: Verifier) -> anyhow::Result<()> {
        debug!("🔁 Started handling incoming connection");

        loop {
            debug!("⏳ Waiting for next unidirectional stream...");
            match conn.accept_uni().await {
                Ok(mut stream) => {
                    debug!("📥 Unidirectional stream accepted");

                    let buf = stream.read_to_end(64 * 1024).await?;
                    debug!(bytes = buf.len(), "📦 Raw data received");

                    let decompressed = decode_all(&buf[..])?;
                    debug!(bytes = decompressed.len(), "📉 Data decompressed");

                    let msg = Message::deserialize(&decompressed)?;
                    msg.verify(&verifier)?;

                    debug!(
                        msg_type = ?msg.msg_type(),
                        from = %msg.from,
                        to = %msg.to,
                        "✅ Message verified and processed"
                    );
                }
                Err(e) => {
                    error!(error = %e, "❌ Failed to accept unidirectional stream");
                    break;
                }
            }
        }

        debug!("🛑 Connection handler exiting");
        Ok(())
    }
}

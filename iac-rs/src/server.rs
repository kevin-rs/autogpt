use crate::crypto::Verifier;
use crate::message::Message;
use crate::transport::init_server;
use anyhow::Result;
use quinn::Endpoint;
use tracing::{debug, error};
use zstd::stream::decode_all;

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

type Handler =
    Arc<dyn Fn(Message) -> Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>> + Send + Sync>;

pub struct Server {
    endpoint: Endpoint,
    handler: Option<Handler>,
}

impl Server {
    pub async fn bind(addr: &str) -> Result<Self> {
        let cfg = init_server()?;
        let endpoint = quinn::Endpoint::server(cfg, addr.parse()?)?;
        debug!(address = %addr, "🚀 Server bound and listening");

        Ok(Self {
            endpoint,
            handler: None,
        })
    }

    pub async fn run(&mut self, verifier: Verifier) -> Result<()> {
        while let Some(connecting) = self.endpoint.accept().await {
            let remote = connecting.remote_address().to_string();

            debug!(remote, "🔌 Incoming connection");

            let conn = connecting.await?;
            debug!(peer = %conn.remote_address(), "✅ Connection established");

            tokio::spawn(Self::handle_conn(
                conn,
                verifier.clone(),
                self.handler.clone(),
            ));
        }

        Ok(())
    }

    pub async fn handle_conn(
        conn: quinn::Connection,
        verifier: Verifier,
        handler: Option<Handler>,
    ) -> anyhow::Result<()> {
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

                    if let Some(handler) = &handler {
                        handler(msg).await?;
                    }
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

    pub fn set_handler<F, Fut>(&mut self, handler_fn: F)
    where
        F: Fn(Message) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = anyhow::Result<()>> + Send + 'static,
    {
        self.handler = Some(Arc::new(move |msg| {
            Box::pin(handler_fn(msg)) as Pin<Box<dyn Future<Output = _> + Send>>
        }));
    }
}

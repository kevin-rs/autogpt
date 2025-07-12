use crate::crypto::Verifier;
use crate::message::{Message, MessageType};
use crate::transport::init_server;
use anyhow::Result;
use anyhow::anyhow;
use ed25519_compact::PublicKey;
use quinn::{Connection, Endpoint};
use std::collections::HashMap;
use tokio::sync::RwLock;
use tracing::{debug, error};
use zstd::stream::decode_all;

use crate::crypto::Signer;
use std::fmt;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use zstd::encode_all;

type Handler = Arc<
    dyn Fn(Message, String) -> Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>
        + Send
        + Sync,
>;

#[derive(Clone)]
pub struct Server {
    endpoint: Endpoint,
    handler: Option<Handler>,
    pub connections: Arc<RwLock<HashMap<String, Connection>>>,
}

impl PartialEq for Server {
    fn eq(&self, _other: &Self) -> bool {
        // We assume that 2 quinn servers are always different
        false
    }
}

impl fmt::Debug for Server {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Server")
            .field("endpoint", &"<quinn::Endpoint>")
            .field("handler", &"<Handler>")
            .finish()
    }
}

impl Server {
    pub async fn bind(addr: &str) -> Result<Self> {
        let cfg = init_server()?;
        let endpoint = quinn::Endpoint::server(cfg, addr.parse()?)?;
        debug!(address = %addr, "🚀 Server bound and listening");

        Ok(Self {
            endpoint,
            handler: None,
            connections: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    pub async fn run(&mut self, verifier: Verifier) -> Result<()> {
        while let Some(connecting) = self.endpoint.accept().await {
            let remote = connecting.remote_address();
            let remote_str = remote.to_string();
            debug!(remote = %remote_str, "🔌 Incoming connection");

            let conn = connecting.await?;
            self.connections
                .write()
                .await
                .insert(remote_str.clone(), conn.clone());
            debug!(peer = %conn.remote_address(), "✅ Connection established");

            self.connections
                .write()
                .await
                .insert(remote_str.clone(), conn.clone());

            let verifier = verifier.clone();
            let handler = self.handler.clone();
            let connections = Arc::clone(&self.connections);

            tokio::spawn(async move {
                if let Err(e) = Self::handle_conn(conn, verifier, handler, remote_str.clone()).await
                {
                    error!(error = %e, "❌ Connection handler failed");
                }

                connections.write().await.remove(&remote_str);
                debug!(peer = %remote_str, "🔌 Connection removed from registry");
            });
        }

        Ok(())
    }

    pub async fn handle_conn(
        conn: quinn::Connection,
        mut verifier: Verifier,
        handler: Option<Handler>,
        remote_str: String,
    ) -> Result<()> {
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

                    if msg.msg_type == MessageType::RegisterKey {
                        if let Ok(pk) = PublicKey::from_slice(&msg.extra_data) {
                            verifier.register_key(pk);
                            debug!("🔐 Registered new public key from agent {}", msg.from);
                            continue;
                        } else {
                            error!(
                                "❌ Invalid public key format in RegisterKey from {}",
                                msg.from
                            );
                            continue;
                        }
                    }

                    msg.verify(&verifier)?;

                    debug!(
                        msg_type = ?msg.msg_type,
                        from = %msg.from,
                        to = %msg.to,
                        "✅ Message verified and processed"
                    );

                    if let Some(handler) = &handler {
                        handler(msg, remote_str.clone()).await?;
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
        F: Fn((Message, String)) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
    {
        self.handler = Some(Arc::new(move |msg, conn| {
            Box::pin(handler_fn((msg, conn))) as Pin<Box<dyn Future<Output = _> + Send>>
        }));
    }

    pub async fn send(&self, to: &str, mut msg: Message, signer: &Signer) -> Result<()> {
        msg.sign(signer)?;
        let connections = self.connections.read().await;

        let conn = connections.get(to).ok_or_else(|| {
            error!(
                "❌ No connection found for '{}'. Active connections: {:?}",
                to,
                connections.keys()
            );
            anyhow!("No active connection found for: {}", to)
        })?;

        let mut stream = conn.open_uni().await?;
        let compressed = encode_all(msg.serialize()?.as_slice(), 0)?;
        stream.write_all(&compressed).await?;
        stream.finish()?;

        debug!(to, "📤 Message sent");
        Ok(())
    }
}

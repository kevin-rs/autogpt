use crate::crypto::Signer;
use crate::message::Message;
use crate::transport::connect;
use anyhow::Result;
use quinn::Connection;
use tracing::{debug, instrument};
use zstd::stream::encode_all;

pub struct Client {
    conn: Connection,
    signer: Signer,
}

impl Client {
    #[instrument(skip_all, fields(addr))]
    pub async fn connect(addr: &str, signer: Signer) -> Result<Self> {
        debug!(%addr, "🌐 Connecting to server...");
        let conn = connect(addr).await?;
        debug!("✅ Client connected to {}", addr);
        Ok(Self { conn, signer })
    }

    #[instrument(skip_all, fields(to = %msg.to, from = %msg.from, msg_id = msg.msg_id))]
    pub async fn send(&self, mut msg: Message) -> Result<()> {
        msg.sign(&self.signer)?;
        debug!("🖋️ Message signed");

        let data = msg.serialize()?;
        debug!(original_len = data.len(), "📦 Message serialized");

        let compressed = encode_all(&data[..], 0)?;
        debug!(compressed_len = compressed.len(), "📉 Message compressed");

        debug!("🔓 Opening unidirectional stream");
        let mut stream = self.conn.open_uni().await?;

        debug!("✍️ Writing {} bytes to stream", compressed.len());
        stream.write_all(&compressed).await?;

        debug!("✅ Write complete, finalizing stream...");
        stream.finish()?;

        debug!("📤 Stream finished successfully");
        Ok(())
    }
}

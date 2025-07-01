use anyhow::{Context, Result};
use prost::Message as ProstMessage;
use tracing::{debug, instrument};

include!(concat!(env!("OUT_DIR"), "/iac.rs"));

impl Message {
    #[instrument(skip_all, fields(msg_id = self.msg_id, msg_type = ?self.msg_type()))]
    pub fn serialize(&self) -> Result<Vec<u8>> {
        let mut buf = Vec::with_capacity(self.encoded_len());
        self.encode(&mut buf).context("Failed to encode Message")?;
        debug!(bytes = buf.len(), "✅ Message serialized");
        Ok(buf)
    }

    #[instrument(skip_all)]
    pub fn deserialize(bytes: &[u8]) -> Result<Self> {
        let msg = Message::decode(bytes).context("Failed to decode Message")?;
        debug!(
            msg_id = msg.msg_id,
            msg_type = ?msg.msg_type(),
            "📥 Message deserialized"
        );
        Ok(msg)
    }

    #[instrument(skip_all, fields(msg_id = self.msg_id))]
    pub fn sign(&mut self, signer: &crate::crypto::Signer) -> Result<()> {
        let mut copy = self.clone();
        copy.signature = vec![];
        let data = copy
            .serialize()
            .context("Failed to serialize message for signing")?;
        self.signature = signer.sign(&data).context("Failed to sign message")?;
        debug!(sig_len = self.signature.len(), "✍️ Message signed");
        Ok(())
    }

    #[instrument(skip_all, fields(msg_id = self.msg_id))]
    pub fn verify(&self, verifier: &crate::crypto::Verifier) -> Result<()> {
        let mut copy = self.clone();
        copy.signature = vec![];
        let data = copy
            .serialize()
            .context("Failed to serialize message for verification")?;
        verifier
            .verify(&data, &self.signature)
            .context("Signature verification failed")?;
        debug!("🔐 Message signature verified");
        Ok(())
    }
}

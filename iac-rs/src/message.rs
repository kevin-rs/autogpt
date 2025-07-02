use anyhow::{Context, Result};
use prost::Message as ProstMessage;
use rand::TryRngCore;
use rand::rngs::OsRng;
use std::time::{SystemTime, UNIX_EPOCH};
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

    #[instrument(skip_all, fields(from = from, to = to))]
    pub fn ping(from: &str, to: &str, session_id: u64) -> Self {
        let timestamp = curr_time();
        let msg_id = gen_msg_id();

        let msg = Message {
            from: from.to_string(),
            to: to.to_string(),
            msg_type: MessageType::Ping.into(),
            payload_json: "".to_string(),
            timestamp,
            msg_id,
            session_id,
            signature: vec![],
            extra_data: vec![],
        };

        debug!(
            msg_id = msg.msg_id,
            msg_type = ?msg.msg_type(),
            "📡 Created PING message"
        );

        msg
    }

    #[instrument(skip_all, fields(from = from))]
    pub fn broadcast(from: &str, payload_json: &str, session_id: u64) -> Self {
        let timestamp = curr_time();
        let msg_id = gen_msg_id();

        let msg = Message {
            from: from.to_string(),
            to: "".to_string(),
            msg_type: MessageType::Broadcast.into(),
            payload_json: payload_json.to_string(),
            timestamp,
            msg_id,
            session_id,
            signature: vec![],
            extra_data: vec![],
        };

        debug!(
            msg_id = msg.msg_id,
            msg_type = ?msg.msg_type(),
            payload_len = payload_json.len(),
            "📢 Created BROADCAST message"
        );

        msg
    }
}

fn curr_time() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn gen_msg_id() -> u64 {
    OsRng
        .try_next_u64()
        .expect("Secure RNG failed to initialize")
}

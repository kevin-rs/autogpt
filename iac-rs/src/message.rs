use anyhow::{Context, Result};
use bytes::{Buf, BufMut};
use prost::Message as ProstMessage;
use prost::encoding::{
    DecodeContext, WireType,
    bytes::{encode as encode_bytes, encoded_len as len_bytes, merge as merge_bytes},
    int32::{encode as encode_int32, encoded_len as len_int32, merge as merge_int32},
    skip_field,
    string::{encode as encode_string, encoded_len as len_string, merge as merge_string},
    uint64::{encode as encode_u64, encoded_len as len_u64, merge as merge_uint64},
};
use rand::TryRngCore;
use rand::rngs::OsRng;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{debug, instrument};

#[cfg(feature = "ser")]
use serde::{Deserialize, Serialize};

/// Enum the various types of messages exchanged between IAC server and agents.
///
/// Each variant corresponds to a specific operation type.
#[cfg_attr(feature = "ser", derive(Serialize, Deserialize))]
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum MessageType {
    /// Default/unknown type.
    #[default]
    Unknown = 0,
    /// Health check or keepalive signal.
    Ping = 1,
    /// Broadcast a message to all agents or listeners.
    Broadcast = 2,
    /// Transfer a file as part of orchestration.
    FileTransfer = 3,
    /// Send a command to be executed.
    Command = 4,
    /// Delegate a task for remote execution.
    DelegateTask = 5,
    /// Register a cryptographic key or identity.
    RegisterKey = 6,
}

impl MessageType {
    /// Convert a raw integer into a `MessageType` enum.
    pub fn from_i32(value: i32) -> Self {
        match value {
            1 => MessageType::Ping,
            2 => MessageType::Broadcast,
            3 => MessageType::FileTransfer,
            4 => MessageType::Command,
            5 => MessageType::DelegateTask,
            6 => MessageType::RegisterKey,
            _ => MessageType::Unknown,
        }
    }

    /// Convert a `MessageType` into its corresponding integer.
    pub fn as_i32(&self) -> i32 {
        *self as i32
    }
}

/// Represents a message used for communication between orchestrator and agents.
///
/// This struct is encoded and decoded using Protobuf (via `prost`). It can also optionally
/// support JSON serialization if the `ser` feature is enabled.
#[cfg_attr(feature = "ser", derive(Serialize, Deserialize))]
#[derive(Clone, PartialEq, Debug, Default)]
pub struct Message {
    /// Identifier for the sender of the message.
    pub from: String,

    /// Identifier for the intended recipient of the message.
    pub to: String,

    /// Describes the type of message (e.g., Ping, Broadcast, Command).
    pub msg_type: MessageType,

    /// JSON string payload containing the message content.
    pub payload_json: String,

    /// Unix timestamp (seconds since epoch) when the message was created.
    pub timestamp: u64,

    /// Unique identifier for the message.
    pub msg_id: u64,

    /// Session ID this message is associated with.
    pub session_id: u64,

    /// Digital signature for message authenticity.
    pub signature: Vec<u8>,

    /// Optional binary data used to store extra content.
    pub extra_data: Vec<u8>,
}

impl ProstMessage for Message {
    fn encode_raw(&self, buf: &mut impl BufMut) {
        encode_string(1, &self.from, buf);
        encode_string(2, &self.to, buf);
        encode_int32(3, &self.msg_type.as_i32(), buf);
        encode_string(4, &self.payload_json, buf);
        encode_u64(5, &self.timestamp, buf);
        encode_u64(6, &self.msg_id, buf);
        encode_u64(7, &self.session_id, buf);
        encode_bytes(8, &self.signature, buf);
        encode_bytes(9, &self.extra_data, buf);
    }
    fn merge_field(
        &mut self,
        tag: u32,
        wire_type: WireType,
        buf: &mut impl Buf,
        ctx: DecodeContext,
    ) -> core::result::Result<(), prost::DecodeError> {
        match tag {
            1 => merge_string(wire_type, &mut self.from, buf, ctx),
            2 => merge_string(wire_type, &mut self.to, buf, ctx),
            3 => {
                let mut raw = 0i32;
                merge_int32(wire_type, &mut raw, buf, ctx)?;
                self.msg_type = MessageType::from_i32(raw);
                Ok(())
            }
            4 => merge_string(wire_type, &mut self.payload_json, buf, ctx),
            5 => merge_uint64(wire_type, &mut self.timestamp, buf, ctx),
            6 => merge_uint64(wire_type, &mut self.msg_id, buf, ctx),
            7 => merge_uint64(wire_type, &mut self.session_id, buf, ctx),
            8 => merge_bytes(wire_type, &mut self.signature, buf, ctx),
            9 => merge_bytes(wire_type, &mut self.extra_data, buf, ctx),
            _ => skip_field(wire_type, tag, buf, ctx),
        }
    }

    fn encoded_len(&self) -> usize {
        len_string(1, &self.from)
            + len_string(2, &self.to)
            + len_int32(3, &self.msg_type.as_i32())
            + len_string(4, &self.payload_json)
            + len_u64(5, &self.timestamp)
            + len_u64(6, &self.msg_id)
            + len_u64(7, &self.session_id)
            + len_bytes(8, &self.signature)
            + len_bytes(9, &self.extra_data)
    }

    fn clear(&mut self) {
        *self = Self::default();
    }
}

/// Decodes a `Message` from a raw byte buffer.
///
/// # Arguments
///
/// * `buf` - Byte slice containing a serialized protobuf `Message`.
///
/// # Returns
///
/// * `Ok(Message)` if decoding succeeds.
/// * `Err(prost::DecodeError)` if the data is malformed.
pub fn parse_message(buf: &[u8]) -> Result<Message, prost::DecodeError> {
    Message::decode(buf)
}

impl Message {
    /// Serializes the message to a byte vector using Protobuf encoding.
    #[instrument(skip_all, fields(msg_id = self.msg_id, msg_type = ?self.msg_type))]
    pub fn serialize(&self) -> Result<Vec<u8>> {
        let mut buf = Vec::with_capacity(self.encoded_len());
        self.encode(&mut buf)
            .map_err(|e| anyhow::anyhow!("Failed to encode Message: {}", e))?;
        debug!(bytes = buf.len(), "✅ Message serialized");
        Ok(buf)
    }

    /// Deserializes a message from a byte slice.
    #[instrument(skip_all)]
    pub fn deserialize(bytes: &[u8]) -> Result<Self> {
        let msg = Message::decode(bytes)
            .map_err(|e| anyhow::anyhow!("Failed to decode Message: {}", e))?;
        debug!(
            msg_id = msg.msg_id,
            msg_type = ?msg.msg_type,
            "📥 Message deserialized"
        );
        Ok(msg)
    }

    /// Signs the message using the provided signer, excluding the signature field.
    #[instrument(skip_all, fields(msg_id = self.msg_id))]
    pub fn sign(&mut self, signer: &crate::crypto::Signer) -> Result<()> {
        let mut copy = self.clone();
        copy.signature = Vec::new();
        let data = copy
            .serialize()
            .context("Failed to serialize message for signing")?;
        self.signature = signer.sign(&data).context("Failed to sign message")?;
        debug!(sig_len = self.signature.len(), "✍️ Message signed");
        Ok(())
    }

    /// Verifies the message signature using the provided verifier.
    #[instrument(skip_all, fields(msg_id = self.msg_id))]
    pub fn verify(&self, verifier: &crate::crypto::Verifier) -> Result<()> {
        let mut copy = self.clone();
        copy.signature = Vec::new();
        let data = copy
            .serialize()
            .context("Failed to serialize message for verification")?;
        verifier
            .verify(&data, &self.signature)
            .context("Signature verification failed")?;
        debug!("🔐 Message signature verified");
        Ok(())
    }

    /// Creates a new PING message with the current timestamp and generated message ID.
    #[instrument(skip_all, fields(from = from, to = to))]
    pub fn ping(from: &str, to: &str, session_id: u64) -> Self {
        let timestamp = curr_time();
        let msg_id = gen_msg_id();

        let msg = Message {
            from: from.to_string(),
            to: to.to_string(),
            msg_type: MessageType::Ping,
            payload_json: "".to_string(),
            timestamp,
            msg_id,
            session_id,
            signature: Vec::new(),
            extra_data: Vec::new(),
        };

        debug!(
            msg_id = msg.msg_id,
            msg_type = ?msg.msg_type,
            "📡 Created PING message"
        );

        msg
    }

    /// Creates a new BROADCAST message with the provided payload and current timestamp.
    #[instrument(skip_all, fields(from = from))]
    pub fn broadcast(from: &str, payload_json: &str, session_id: u64) -> Self {
        let timestamp = curr_time();
        let msg_id = gen_msg_id();

        let msg = Message {
            from: from.to_string(),
            to: "".to_string(),
            msg_type: MessageType::Broadcast,
            payload_json: payload_json.to_string(),
            timestamp,
            msg_id,
            session_id,
            signature: Vec::new(),
            extra_data: Vec::new(),
        };

        debug!(
            msg_id = msg.msg_id,
            msg_type = ?msg.msg_type,
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

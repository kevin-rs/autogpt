use anyhow::Result;
use prost::Message as ProstMessage;

/// Represents a message used for communication between orchestrator and agents.
///
/// This struct is encoded and decoded using Protobuf (via `prost`).
#[derive(Clone, PartialEq, prost::Message)]
pub struct Message {
    /// Identifier for the sender of the message.
    #[prost(string, tag = "1")]
    pub from: String,

    /// Identifier for the intended recipient of the message.
    #[prost(string, tag = "2")]
    pub to: String,

    /// Describes the type of message, such as "create", "run", or "terminate".
    #[prost(string, tag = "3")]
    pub msg_type: String,

    /// JSON string payload containing the message content.
    #[prost(string, tag = "4")]
    pub payload_json: String,

    /// Authentication token used for validating the message sender.
    #[prost(string, tag = "5")]
    pub auth_token: String,
}

/// Decodes a `Message` from a raw byte buffer.
pub fn parse_message(buf: &[u8]) -> Result<Message> {
    Ok(Message::decode(buf)?)
}

/// Encodes a `Message` into a byte vector suitable for transmission.
pub fn encode_message(msg: &Message) -> Result<Vec<u8>> {
    let mut buf = Vec::new();
    msg.encode(&mut buf)?;
    Ok(buf)
}

/// Parses a key-value formatted payload string into individual values.
///
/// The function expects a string in the format:
/// `input=some text;language=python`, and extracts the `input` and `language` fields.
///
/// # Arguments
///
/// * `payload` - A string containing key-value pairs separated by semicolons.
///
/// # Returns
///
/// * A tuple `(input, language)` extracted from the payload. Defaults are empty string and "python" respectively
///   if the keys are not present.
pub fn parse_kv(payload: &str) -> (String, String) {
    let mut input = "".to_string();
    let mut lang = "python".to_string();

    for part in payload.split(';') {
        let mut kv = part.splitn(2, '=');
        let key = kv.next().unwrap_or("");
        let val = kv.next().unwrap_or("").to_string();
        if key == "input" {
            input = val;
        } else if key == "language" {
            lang = val;
        }
    }

    (input, lang)
}

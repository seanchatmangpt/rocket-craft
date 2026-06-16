//! Type-safe message codec — serialize/deserialize `ClientMessage` and
//! `ServerMessage` to/from JSON bytes.
//!
//! All game code uses these methods rather than calling `serde_json` directly,
//! so the codec layer can enforce size limits or swap encoding formats in future.

use crate::protocol::{ClientMessage, ServerMessage};

// ── Errors ────────────────────────────────────────────────────────────────────

/// Errors produced by `MessageCodec`.
#[derive(Debug, thiserror::Error)]
pub enum CodecError {
    #[error("serialization error: {0}")]
    SerializeError(String),
    #[error("parse error: {0}")]
    ParseError(String),
    #[error("message too large: {size} bytes (max {max})")]
    MessageTooLarge { size: usize, max: usize },
}

// ── MessageCodec ──────────────────────────────────────────────────────────────

/// Zero-sized codec type; all methods are `fn` (no state needed).
pub struct MessageCodec;

impl MessageCodec {
    /// Encode a `ServerMessage` to JSON bytes for transmission.
    pub fn encode_server(msg: &ServerMessage) -> Result<Vec<u8>, CodecError> {
        serde_json::to_vec(msg).map_err(|e| CodecError::SerializeError(e.to_string()))
    }

    /// Decode a `ClientMessage` from raw JSON bytes received over the wire.
    pub fn decode_client(bytes: &[u8]) -> Result<ClientMessage, CodecError> {
        serde_json::from_slice(bytes).map_err(|e| CodecError::ParseError(e.to_string()))
    }

    /// Encode a `ClientMessage` to JSON bytes (used in tests and the client SDK).
    pub fn encode_client(msg: &ClientMessage) -> Result<Vec<u8>, CodecError> {
        serde_json::to_vec(msg).map_err(|e| CodecError::SerializeError(e.to_string()))
    }

    /// Decode a `ServerMessage` from raw JSON bytes (used in tests and the client SDK).
    pub fn decode_server(bytes: &[u8]) -> Result<ServerMessage, CodecError> {
        serde_json::from_slice(bytes).map_err(|e| CodecError::ParseError(e.to_string()))
    }

    /// Encode a `ServerMessage` to a JSON string for human-readable logging.
    pub fn encode_server_pretty(msg: &ServerMessage) -> Result<String, CodecError> {
        serde_json::to_string_pretty(msg).map_err(|e| CodecError::SerializeError(e.to_string()))
    }

    /// Encode a `ClientMessage` to a JSON string for human-readable logging.
    pub fn encode_client_pretty(msg: &ClientMessage) -> Result<String, CodecError> {
        serde_json::to_string_pretty(msg).map_err(|e| CodecError::SerializeError(e.to_string()))
    }

    /// Validate that a raw byte buffer does not exceed `max_bytes`.
    ///
    /// Call this before `decode_client` on the server to defend against
    /// oversized payloads.
    pub fn check_size(bytes: &[u8], max_bytes: usize) -> Result<(), CodecError> {
        if bytes.len() > max_bytes {
            Err(CodecError::MessageTooLarge { size: bytes.len(), max: max_bytes })
        } else {
            Ok(())
        }
    }
}

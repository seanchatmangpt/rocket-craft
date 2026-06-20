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
            Err(CodecError::MessageTooLarge {
                size: bytes.len(),
                max: max_bytes,
            })
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::{ClientMessage, ServerMessage};

    // ── encode_client / decode_client round-trip ──────────────────────────────

    #[test]
    fn ping_round_trips_through_client_codec() {
        let msg = ClientMessage::Ping { seq: 42 };
        let bytes = MessageCodec::encode_client(&msg).unwrap();
        let decoded = MessageCodec::decode_client(&bytes).unwrap();
        assert!(matches!(decoded, ClientMessage::Ping { seq: 42 }));
    }

    #[test]
    fn join_lobby_round_trips_through_client_codec() {
        let bytes = MessageCodec::encode_client(&ClientMessage::JoinLobby).unwrap();
        assert!(matches!(
            MessageCodec::decode_client(&bytes).unwrap(),
            ClientMessage::JoinLobby
        ));
    }

    #[test]
    fn authenticate_preserves_player_id_and_token() {
        let msg = ClientMessage::Authenticate {
            player_id: 99,
            token: "tok-abc".into(),
        };
        let bytes = MessageCodec::encode_client(&msg).unwrap();
        match MessageCodec::decode_client(&bytes).unwrap() {
            ClientMessage::Authenticate { player_id, token } => {
                assert_eq!(player_id, 99);
                assert_eq!(token, "tok-abc");
            }
            other => panic!("unexpected variant: {other:?}"),
        }
    }

    // ── encode_server / decode_server round-trip ──────────────────────────────

    #[test]
    fn server_pong_round_trips() {
        let msg = ServerMessage::Pong { seq: 7, server_time_ms: 0 };
        let bytes = MessageCodec::encode_server(&msg).unwrap();
        assert!(matches!(
            MessageCodec::decode_server(&bytes).unwrap(),
            ServerMessage::Pong { seq: 7, .. }
        ));
    }

    #[test]
    fn server_error_round_trips() {
        let msg = ServerMessage::Error {
            code: 404,
            message: "not found".into(),
        };
        let bytes = MessageCodec::encode_server(&msg).unwrap();
        match MessageCodec::decode_server(&bytes).unwrap() {
            ServerMessage::Error { code, message } => {
                assert_eq!(code, 404);
                assert_eq!(message, "not found");
            }
            other => panic!("unexpected variant: {other:?}"),
        }
    }

    // ── cross-direction decode errors ─────────────────────────────────────────

    #[test]
    fn client_bytes_fail_to_decode_as_server_message() {
        let bytes = MessageCodec::encode_client(&ClientMessage::JoinLobby).unwrap();
        // ClientMessage has a different `type` tag than any ServerMessage variant
        let result = MessageCodec::decode_server(&bytes);
        assert!(result.is_err(), "client bytes must not decode as ServerMessage");
    }

    // ── check_size ────────────────────────────────────────────────────────────

    #[test]
    fn check_size_passes_when_within_limit() {
        let payload = b"hello";
        assert!(MessageCodec::check_size(payload, 10).is_ok());
    }

    #[test]
    fn check_size_passes_at_exact_limit() {
        let payload = b"hello";
        assert!(MessageCodec::check_size(payload, 5).is_ok());
    }

    #[test]
    fn check_size_fails_when_over_limit() {
        let payload = b"hello";
        let result = MessageCodec::check_size(payload, 4);
        assert!(result.is_err());
        assert!(matches!(result, Err(CodecError::MessageTooLarge { size: 5, max: 4 })));
    }

    #[test]
    fn empty_payload_always_passes_check_size() {
        assert!(MessageCodec::check_size(b"", 0).is_ok());
    }

    // ── pretty encoding is valid JSON ─────────────────────────────────────────

    #[test]
    fn encode_server_pretty_produces_valid_json() {
        let msg = ServerMessage::Pong { seq: 1, server_time_ms: 0 };
        let pretty = MessageCodec::encode_server_pretty(&msg).unwrap();
        let _: serde_json::Value = serde_json::from_str(&pretty).expect("must be valid JSON");
        assert!(pretty.contains('\n'), "pretty output must contain newlines");
    }

    #[test]
    fn encode_client_pretty_produces_valid_json() {
        let msg = ClientMessage::Ping { seq: 0 };
        let pretty = MessageCodec::encode_client_pretty(&msg).unwrap();
        let _: serde_json::Value = serde_json::from_str(&pretty).expect("must be valid JSON");
    }

    // ── malformed bytes ───────────────────────────────────────────────────────

    #[test]
    fn garbage_bytes_fail_to_decode_as_client_message() {
        let result = MessageCodec::decode_client(b"not-json");
        assert!(matches!(result, Err(CodecError::ParseError(_))));
    }

    #[test]
    fn empty_bytes_fail_to_decode_as_client_message() {
        let result = MessageCodec::decode_client(b"");
        assert!(matches!(result, Err(CodecError::ParseError(_))));
    }
}

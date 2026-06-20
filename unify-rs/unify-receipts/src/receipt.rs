use serde::{Deserialize, Serialize};

/// A BLAKE3-backed receipt proving a capability was granted.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Receipt {
    /// The capability key this receipt covers.
    pub key: String,
    /// BLAKE3 hash of the data at grant time, hex-encoded.
    pub hash: String,
    /// Unix timestamp (ms) when the receipt was created.
    pub issued_at: u64,
}

impl Receipt {
    /// Create a new receipt by hashing `data` under `key`.
    pub fn new(key: impl Into<String>, data: &[u8]) -> Self {
        let key = key.into();
        let hash = blake3::hash(data).to_hex().to_string();
        let issued_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);
        Self {
            key,
            hash,
            issued_at,
        }
    }

    /// Verify the receipt matches `data`.
    pub fn verify(&self, data: &[u8]) -> bool {
        let expected = blake3::hash(data).to_hex().to_string();
        self.hash == expected
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Receipt::new ──────────────────────────────────────────────────────────

    #[test]
    fn new_stores_key() {
        let r = Receipt::new("my-key", b"data");
        assert_eq!(r.key, "my-key");
    }

    #[test]
    fn new_hash_is_64_hex_chars() {
        let r = Receipt::new("k", b"hello");
        assert_eq!(r.hash.len(), 64);
        assert!(r.hash.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn new_issued_at_is_nonzero() {
        let r = Receipt::new("k", b"data");
        assert!(r.issued_at > 0);
    }

    #[test]
    fn same_data_produces_same_hash() {
        let a = Receipt::new("k", b"deterministic");
        let b = Receipt::new("k", b"deterministic");
        assert_eq!(a.hash, b.hash);
    }

    #[test]
    fn different_data_produces_different_hash() {
        let a = Receipt::new("k", b"foo");
        let b = Receipt::new("k", b"bar");
        assert_ne!(a.hash, b.hash);
    }

    // ── Receipt::verify ───────────────────────────────────────────────────────

    #[test]
    fn verify_returns_true_for_matching_data() {
        let r = Receipt::new("k", b"payload");
        assert!(r.verify(b"payload"));
    }

    #[test]
    fn verify_returns_false_for_tampered_data() {
        let r = Receipt::new("k", b"original");
        assert!(!r.verify(b"tampered"));
    }

    #[test]
    fn verify_empty_data_round_trips() {
        let r = Receipt::new("k", b"");
        assert!(r.verify(b""));
        assert!(!r.verify(b"not-empty"));
    }

    // ── serde roundtrip ───────────────────────────────────────────────────────

    #[test]
    fn roundtrip_json() {
        let r = Receipt::new("serde-test", b"content");
        let json = serde_json::to_string(&r).unwrap();
        let restored: Receipt = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.key, r.key);
        assert_eq!(restored.hash, r.hash);
        assert_eq!(restored.issued_at, r.issued_at);
    }
}

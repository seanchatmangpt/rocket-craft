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
        Self { key, hash, issued_at }
    }

    /// Verify the receipt matches `data`.
    pub fn verify(&self, data: &[u8]) -> bool {
        let expected = blake3::hash(data).to_hex().to_string();
        self.hash == expected
    }
}

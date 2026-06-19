//! BlueprintReceiptChain — track Blueprint generation provenance using
//! BLAKE3-backed receipts.

use blueprint_core::{Blueprint, T3dSerializer};
use serde::{Deserialize, Serialize};

/// A BLAKE3-backed receipt proving a Blueprint operation was executed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Receipt {
    /// Capability key (e.g. "blueprint.generate", "blueprint.validate").
    pub key: String,
    /// BLAKE3 hash of the relevant data at operation time, hex-encoded.
    pub hash: String,
    /// Unix timestamp (ms) when the receipt was issued.
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

/// A chain of receipts recording Blueprint generation and validation events.
pub struct BlueprintReceiptChain {
    receipts: Vec<Receipt>,
}

impl Default for BlueprintReceiptChain {
    fn default() -> Self {
        Self::new()
    }
}

impl BlueprintReceiptChain {
    /// Create a new, empty receipt chain.
    pub fn new() -> Self {
        Self {
            receipts: Vec::new(),
        }
    }

    /// Append a receipt for generating this Blueprint's T3D output.
    ///
    /// The receipt data is the UTF-8 bytes of the serialized T3D string.
    pub fn record_generation(&mut self, bp: &Blueprint) -> &Receipt {
        let t3d = T3dSerializer::serialize(bp);
        let key = format!("blueprint.generate:{}", bp.name);
        let receipt = Receipt::new(key, t3d.as_bytes());
        self.receipts.push(receipt);
        self.receipts.last().unwrap()
    }

    /// Append a receipt for a Blueprint validation run.
    ///
    /// The key includes `"validation"` and whether the run passed.
    pub fn record_validation(&mut self, bp: &Blueprint, passed: bool) -> &Receipt {
        let status = if passed { "passed" } else { "failed" };
        let data = format!("{}:{}", bp.name, status);
        let key = format!("blueprint.validation.{}:{}", status, bp.name);
        let receipt = Receipt::new(key, data.as_bytes());
        self.receipts.push(receipt);
        self.receipts.last().unwrap()
    }

    /// Return an immutable slice of all recorded receipts.
    pub fn chain(&self) -> &[Receipt] {
        &self.receipts
    }

    /// Returns `true` if at least one receipt has been recorded.
    pub fn is_valid(&self) -> bool {
        !self.receipts.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use blueprint_core::Blueprint;

    fn make_bp() -> Blueprint {
        Blueprint::new("HeroActor", "Actor")
    }

    // ── BlueprintReceiptChain::new ────────────────────────────────────────────

    #[test]
    fn new_chain_is_empty_and_invalid() {
        let chain = BlueprintReceiptChain::new();
        assert!(!chain.is_valid());
        assert!(chain.chain().is_empty());
    }

    // ── record_generation ────────────────────────────────────────────────────

    #[test]
    fn record_generation_adds_receipt() {
        let mut chain = BlueprintReceiptChain::new();
        chain.record_generation(&make_bp());
        assert_eq!(chain.chain().len(), 1);
        assert!(chain.is_valid());
    }

    #[test]
    fn record_generation_key_contains_blueprint_name() {
        let mut chain = BlueprintReceiptChain::new();
        let r = chain.record_generation(&make_bp());
        assert!(r.key.contains("HeroActor"));
        assert!(r.key.contains("blueprint.generate"));
    }

    #[test]
    fn record_generation_hash_is_deterministic() {
        let bp = make_bp();
        let mut c1 = BlueprintReceiptChain::new();
        let mut c2 = BlueprintReceiptChain::new();
        assert_eq!(c1.record_generation(&bp).hash, c2.record_generation(&bp).hash);
    }

    // ── record_validation ────────────────────────────────────────────────────

    #[test]
    fn record_validation_passed_key_contains_passed() {
        let mut chain = BlueprintReceiptChain::new();
        let r = chain.record_validation(&make_bp(), true);
        assert!(r.key.contains("passed"));
    }

    #[test]
    fn record_validation_failed_key_contains_failed() {
        let mut chain = BlueprintReceiptChain::new();
        let r = chain.record_validation(&make_bp(), false);
        assert!(r.key.contains("failed"));
    }

    #[test]
    fn chain_accumulates_all_receipts() {
        let mut chain = BlueprintReceiptChain::new();
        let bp = make_bp();
        chain.record_generation(&bp);
        chain.record_validation(&bp, true);
        assert_eq!(chain.chain().len(), 2);
    }
}

//! GC-MECHBIRTH-002: Tamper-Evident Receipt Chain
//! blake3-hashed, sequenced chain of admission events.

use crate::error::RefusalReason;
use blake3;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AdmissionStatus {
    Admitted,
    Refused,
    Residual,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceiptEntry {
    pub sequence: u64,
    pub event_type: String,
    pub objects: Vec<String>,
    pub prev_hash: String,
    pub receipt: String,
    pub status: AdmissionStatus,
    pub residuals: Vec<String>,
}

impl ReceiptEntry {
    /// Compute the receipt hash from this entry's content.
    pub fn compute_hash(&self) -> String {
        let mut hasher = blake3::Hasher::new();
        hasher.update(self.sequence.to_le_bytes().as_slice());
        hasher.update(self.event_type.as_bytes());
        hasher.update(self.prev_hash.as_bytes());
        for obj in &self.objects {
            hasher.update(obj.as_bytes());
        }
        hasher.finalize().to_hex().to_string()
    }
}

/// Chain of tamper-evident receipt entries.
#[derive(Debug, Default)]
pub struct ReceiptChain {
    pub entries: Vec<ReceiptEntry>,
}

impl ReceiptChain {
    pub fn append(
        &mut self,
        event_type: String,
        objects: Vec<String>,
        status: AdmissionStatus,
        residuals: Vec<String>,
    ) -> String {
        let prev_hash = self
            .entries
            .last()
            .map(|e| e.receipt.clone())
            .unwrap_or_else(|| {
                "0000000000000000000000000000000000000000000000000000000000000000".into()
            });
        let sequence = self.entries.len() as u64 + 1;
        let mut entry = ReceiptEntry {
            sequence,
            event_type,
            objects,
            prev_hash,
            receipt: String::new(),
            status,
            residuals,
        };
        entry.receipt = entry.compute_hash();
        let receipt = entry.receipt.clone();
        self.entries.push(entry);
        receipt
    }

    /// Verify the full chain: each entry's prev_hash must equal the prior entry's receipt.
    pub fn verify(&self) -> Result<(), RefusalReason> {
        for i in 1..self.entries.len() {
            let expected = &self.entries[i - 1].receipt;
            let actual = &self.entries[i].prev_hash;
            if expected != actual {
                return Err(RefusalReason::ReceiptChainBroken {
                    sequence: self.entries[i].sequence,
                    expected: expected.clone(),
                    actual: actual.clone(),
                });
            }
        }
        Ok(())
    }

    /// Verify recomputed hash for each entry matches stored receipt.
    pub fn verify_hashes(&self) -> Result<(), RefusalReason> {
        for entry in &self.entries {
            let computed = entry.compute_hash();
            if computed != entry.receipt {
                return Err(RefusalReason::ReceiptChainBroken {
                    sequence: entry.sequence,
                    expected: computed,
                    actual: entry.receipt.clone(),
                });
            }
        }
        Ok(())
    }
}

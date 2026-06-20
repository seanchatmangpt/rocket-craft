use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ReceiptEvent {
    pub sequence: u64,
    pub event_type: String,
    pub surface: String,
    pub objects: Vec<String>,
    pub input_hash: String,
    pub output_hash: String,
    pub prev_hash: Option<String>,
    pub receipt: String,
    pub status: String,
    pub residuals: Vec<String>,
}

pub fn generate_hash(payload: &str) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(payload.as_bytes());
    hasher.finalize().to_hex().to_string()
}

pub fn verify_receipt_chain(chain: &[ReceiptEvent]) -> anyhow::Result<()> {
    let mut expected_prev: Option<String> = None;
    let mut expected_seq = 1;
    for receipt in chain {
        if receipt.sequence != expected_seq {
            anyhow::bail!("Sequence mismatch");
        }
        if receipt.prev_hash != expected_prev {
            anyhow::bail!("Broken prev_hash");
        }
        let payload = format!("{}:{}:{}", receipt.sequence, receipt.event_type, receipt.status);
        let expected_hash = generate_hash(&payload);
        if receipt.receipt != expected_hash {
            anyhow::bail!("Mutated event");
        }
        expected_prev = Some(receipt.receipt.clone());
        expected_seq += 1;
    }
    Ok(())
}

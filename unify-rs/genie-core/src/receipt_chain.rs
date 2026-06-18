use crate::errors::GenieError;
use crate::spec::WorldSpec;
use unify_receipts::receipt::Receipt;

/// Module for handling BLAKE3 cryptographic receipt chains to verify world history integrity.
pub struct ReceiptChainManager;

impl ReceiptChainManager {
    /// Deterministically generates a chain of receipts for all history events in `WorldSpec`.
    ///
    /// The first receipt (genesis) binds the first history event and the provided `secret_salt`.
    /// Each subsequent receipt binds the current history event and the hash of the preceding receipt.
    /// This forms an immutable chain proving the order and content of all events.
    pub fn generate_receipt_chain(
        spec: &mut WorldSpec,
        secret_salt: &[u8],
    ) -> Result<(), GenieError> {
        // Clear any existing receipts to rebuild the chain from scratch
        spec.receipts.clear();

        if spec.history.is_empty() {
            return Ok(());
        }

        // Sort history events by timestamp then ID to guarantee deterministic ordering
        let mut sorted_history = spec.history.clone();
        sorted_history.sort_by(|a, b| a.timestamp_ms.cmp(&b.timestamp_ms).then(a.id.cmp(&b.id)));

        for (idx, event) in sorted_history.iter().enumerate() {
            let event_bytes = serde_json::to_vec(event).map_err(|e| {
                GenieError::Evolution(format!("Failed to serialize history event: {}", e))
            })?;

            let receipt = if idx == 0 {
                // First event: concat secret_salt + engine_version + serialized event
                let engine_bytes = spec.engine_version.as_bytes();
                let mut data =
                    Vec::with_capacity(secret_salt.len() + engine_bytes.len() + event_bytes.len());
                data.extend_from_slice(secret_salt);
                data.extend_from_slice(engine_bytes);
                data.extend_from_slice(&event_bytes);
                Receipt::new(format!("history_receipt_{}", event.id), &data)
            } else {
                // Subsequent event: concat secret_salt + previous receipt hash (hex bytes) + serialized event
                let prev_hash = spec.receipts.last().unwrap().hash.as_bytes();
                let mut data =
                    Vec::with_capacity(secret_salt.len() + prev_hash.len() + event_bytes.len());
                data.extend_from_slice(secret_salt);
                data.extend_from_slice(prev_hash);
                data.extend_from_slice(&event_bytes);
                Receipt::new(format!("history_receipt_{}", event.id), &data)
            };

            spec.receipts.push(receipt);
        }

        Ok(())
    }

    /// Verifies the cryptographic integrity of the receipt chain in `WorldSpec`.
    ///
    /// Returns `true` only if all receipts in the spec correspond to the history events
    /// and form a valid cryptographic BLAKE3 chain originating from the `secret_salt`.
    pub fn verify_receipt_chain(spec: &WorldSpec, secret_salt: &[u8]) -> bool {
        if spec.history.is_empty() {
            return spec.receipts.is_empty();
        }

        if spec.receipts.len() != spec.history.len() {
            return false;
        }

        // Sort history events identically to how they were chained
        let mut sorted_history = spec.history.clone();
        sorted_history.sort_by(|a, b| a.timestamp_ms.cmp(&b.timestamp_ms).then(a.id.cmp(&b.id)));

        for (idx, event) in sorted_history.iter().enumerate() {
            let event_bytes = match serde_json::to_vec(event) {
                Ok(bytes) => bytes,
                Err(_) => return false,
            };

            let receipt = &spec.receipts[idx];

            // Verify receipt key matches
            let expected_key = format!("history_receipt_{}", event.id);
            if receipt.key != expected_key {
                return false;
            }

            // Verify hash match
            let matches = if idx == 0 {
                let engine_bytes = spec.engine_version.as_bytes();
                let mut data =
                    Vec::with_capacity(secret_salt.len() + engine_bytes.len() + event_bytes.len());
                data.extend_from_slice(secret_salt);
                data.extend_from_slice(engine_bytes);
                data.extend_from_slice(&event_bytes);
                receipt.verify(&data)
            } else {
                let prev_hash = spec.receipts[idx - 1].hash.as_bytes();
                let mut data =
                    Vec::with_capacity(secret_salt.len() + prev_hash.len() + event_bytes.len());
                data.extend_from_slice(secret_salt);
                data.extend_from_slice(prev_hash);
                data.extend_from_slice(&event_bytes);
                receipt.verify(&data)
            };

            if !matches {
                return false;
            }
        }

        true
    }
}

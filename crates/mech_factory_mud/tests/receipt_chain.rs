use mech_factory_mud::receipt::{ReceiptEvent, verify_receipt_chain, generate_hash};

#[test]
fn test_valid_receipt_chain_passes() {
    let r1 = create_receipt(1, "A", None);
    let r2 = create_receipt(2, "B", Some(r1.receipt.clone()));
    assert!(verify_receipt_chain(&[r1, r2]).is_ok());
}

#[test]
fn test_broken_prev_hash_fails() {
    let r1 = create_receipt(1, "A", None);
    let r2 = create_receipt(2, "B", Some("wrong".to_string()));
    assert!(verify_receipt_chain(&[r1, r2]).is_err());
}

#[test]
fn test_mutated_event_fails() {
    let r1 = create_receipt(1, "A", None);
    let mut r2 = create_receipt(2, "B", Some(r1.receipt.clone()));
    r2.event_type = "Mutated".to_string(); // Mutate event without updating hash
    assert!(verify_receipt_chain(&[r1, r2]).is_err());
}

#[test]
fn test_missing_sequence_fails() {
    let r1 = create_receipt(1, "A", None);
    let r3 = create_receipt(3, "C", Some(r1.receipt.clone()));
    assert!(verify_receipt_chain(&[r1, r3]).is_err());
}

#[test]
fn test_duplicate_sequence_fails() {
    let r1 = create_receipt(1, "A", None);
    let r2 = create_receipt(1, "A", Some(r1.receipt.clone()));
    assert!(verify_receipt_chain(&[r1, r2]).is_err());
}

fn create_receipt(seq: u64, evt: &str, prev: Option<String>) -> ReceiptEvent {
    let payload = format!("{}:{}:ADMITTED", seq, evt);
    let hash = generate_hash(&payload);
    ReceiptEvent {
        sequence: seq,
        event_type: evt.to_string(),
        surface: "test".to_string(),
        objects: vec![],
        input_hash: "".to_string(),
        output_hash: "".to_string(),
        prev_hash: prev,
        receipt: hash,
        status: "ADMITTED".to_string(),
        residuals: vec![],
    }
}

use genie_core::spec::{WorldSpec, HistoryEvent};
use genie_core::receipt_chain::ReceiptChainManager;

#[test]
fn should_generate_and_verify_cryptographic_receipt_chain() {
    // 1. Setup
    let mut spec = WorldSpec::new();
    let secret_salt = b"test_salt";

    let mut event1 = HistoryEvent::new("event_1", 1000, "create_place");
    event1.details.insert("modification_intent".to_string(), serde_json::Value::String("create place room_1".to_string()));
    spec.history.push(event1);

    let mut event2 = HistoryEvent::new("event_2", 2000, "create_actor");
    event2.details.insert("modification_intent".to_string(), serde_json::Value::String("create actor bot_1".to_string()));
    spec.history.push(event2);

    // 2. Act - Generate Chain
    assert_eq!(spec.engine_version, "UE4.27-ES3", "WorldSpec should default to UE4.27-ES3");
    let result = ReceiptChainManager::generate_receipt_chain(&mut spec, secret_salt);

    // 3. Assert Generation
    assert!(result.is_ok());
    assert_eq!(spec.receipts.len(), 2, "Should generate exactly 2 receipts");
    assert_eq!(spec.receipts[0].key, "history_receipt_event_1");
    assert_eq!(spec.receipts[1].key, "history_receipt_event_2");

    // 4. Act & Assert Verification
    let is_valid = ReceiptChainManager::verify_receipt_chain(&spec, secret_salt);
    assert!(is_valid, "Valid receipt chain must verify successfully with UE4.27-ES3");

    // 5. Tamper Detection (Behavioral failure condition)
    
    // Test 5a: Modify a past event to simulate tampering
    let mut spec_tampered_event = spec.clone();
    spec_tampered_event.history[0].details.insert("modification_intent".to_string(), serde_json::Value::String("create place room_tampered".to_string()));
    
    let is_valid_after_tamper = ReceiptChainManager::verify_receipt_chain(&spec_tampered_event, secret_salt);
    assert!(!is_valid_after_tamper, "Tampered event must fail verification");

    // Test 5b: Modify engine version to simulate tampering
    let mut spec_tampered_engine = spec.clone();
    spec_tampered_engine.engine_version = "UE5.0-ES3".to_string();
    
    let is_valid_after_engine_tamper = ReceiptChainManager::verify_receipt_chain(&spec_tampered_engine, secret_salt);
    assert!(!is_valid_after_engine_tamper, "Tampered engine version must fail verification");
}

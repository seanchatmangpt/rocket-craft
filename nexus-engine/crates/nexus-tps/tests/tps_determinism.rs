use nexus_tps::{generate_part, PartStateVector, PartSlot, TpsReceipt};

#[test]
fn test_tps_receipt_byte_determinism() {
    let state = PartStateVector {
        civilization_id: 2,
        frame_id: 42,
        armor_profile: 0.75,
        joint_profile: 0.25,
        mass_profile: 0.125,
        weapon_profile: 0.5,
        motion_profile: 80.0,
        part_slot: PartSlot::Torso,
    };
    let part = generate_part(&state).unwrap();
    
    // Successive generation of receipts with identical inputs
    let receipt1 = TpsReceipt::generate(&state, &part, vec!["GateA".to_string(), "GateB".to_string()]);
    let bytes1 = serde_json::to_vec(&receipt1).expect("Failed to serialize receipt1");

    // Sleep to ensure time differences would affect the receipt if it used any timestamp
    std::thread::sleep(std::time::Duration::from_millis(15));

    let receipt2 = TpsReceipt::generate(&state, &part, vec!["GateA".to_string(), "GateB".to_string()]);
    let bytes2 = serde_json::to_vec(&receipt2).expect("Failed to serialize receipt2");

    assert_eq!(bytes1, bytes2, "TpsReceipt serialization did not yield byte-identical outputs across successive runs");
}

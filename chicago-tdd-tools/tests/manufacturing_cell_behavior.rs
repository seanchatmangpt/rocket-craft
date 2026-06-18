use genie_core::spec::{WorldSpec, HistoryEvent, Place, Bounds3D, Vector3};
use genie_core::receipt_chain::ReceiptChainManager;
use genie_core::layout::LayoutCompiler;

#[test]
fn should_verify_427_es3_t3d_semantic_correctness() {
    // 1. Setup - Create a world spec reflecting 4.27 ES3 requirements
    let mut spec = WorldSpec::new();
    
    let room = Place::new("control_room", "Control Room", Bounds3D {
        center: Vector3::new(100.0, 200.0, 0.0),
        half_extents: Vector3::new(500.0, 500.0, 10.0),
    });
    spec.places.push(room);

    // 2. Act - Compile to T3D
    let t3d_output = LayoutCompiler::compile(&spec);

    // 3. Assert - Verify 4.27/ES3 compatibility
    // Check for standard UE4 actor classes
    assert!(t3d_output.contains("Begin Actor Class=StaticMeshActor"));
    assert!(t3d_output.contains("Archetype=StaticMeshActor'/Script/Engine.Default__StaticMeshActor'"));
    
    // Verify spatial layout (Floor positioning logic in LayoutCompiler)
    // LayoutCompiler adds (center.z - half_extents.z - 50.0) to floor Z
    // 0.0 - 10.0 - 50.0 = -60.0
    assert!(t3d_output.contains("RelativeLocation=(X=100.000000,Y=200.000000,Z=-60.000000)"));
    
    // Verify meshes used are available in standard UE4 engine content
    assert!(t3d_output.contains("StaticMesh=StaticMesh'/Engine/BasicShapes/Cube.Cube'"));
}

#[test]
fn should_capture_427_es3_metadata_in_receipt_chain() {
    // 1. Setup
    let mut spec = WorldSpec::new();
    let secret_salt = b"manufacturing_salt";

    let mut event = HistoryEvent::new("evolve_1", 1700000000000, "Evolve");
    event.details.insert("engine_version".to_string(), serde_json::Value::String("UE4.27-ES3".to_string()));
    event.details.insert("rendering_api".to_string(), serde_json::Value::String("WebGL-ES3".to_string()));
    event.details.insert("modification_intent".to_string(), serde_json::Value::String("Add factory floor".to_string()));
    spec.history.push(event);

    // 2. Act
    ReceiptChainManager::generate_receipt_chain(&mut spec, secret_salt).expect("Chain generation failed");

    // 3. Assert
    assert_eq!(spec.receipts.len(), 1);
    let receipt = &spec.receipts[0];
    assert_eq!(receipt.key, "history_receipt_evolve_1");
    assert!(!receipt.hash.is_empty());
    
    // Verify the receipt actually binds the metadata
    // We do this by checking if the chain verifies only with the correct data
    let is_valid = ReceiptChainManager::verify_receipt_chain(&spec, secret_salt);
    assert!(is_valid, "Chain must be valid with original metadata");

    // Counterfactual: Modify metadata and expect failure
    let mut tampered_spec = spec.clone();
    tampered_spec.history[0].details.insert("engine_version".to_string(), serde_json::Value::String("UE5.0".to_string()));
    
    let is_valid_tampered = ReceiptChainManager::verify_receipt_chain(&tampered_spec, secret_salt);
    assert!(!is_valid_tampered, "Tampered metadata must fail verification");
}

use genie_core::{
    deployment::DeploymentManager,
    evolution::WorldEvolver,
    layout::LayoutCompiler,
    parse_intent,
    receipt_chain::ReceiptChainManager,
    spec::{Actor, Bounds3D, Object, Place, Vector3, WorldSpec},
};
use std::fs;

#[test]
fn test_layout_compiler_custom_class_paths() {
    let mut spec = WorldSpec::new();
    let bounds = Bounds3D::new(
        Vector3::new(100.0, -100.0, 50.0),
        Vector3::new(200.0, 200.0, 50.0),
    );
    spec.places
        .push(Place::new("sector_7", "Sector Seven Floor", bounds));

    // Actor with relative role (not starting with /)
    let mut actor1 = Actor::new("spy_1", "Infiltrator", "Agent", "sector_7");
    actor1.placement.position = Vector3::new(10.0, 20.0, 0.0);
    spec.actors.push(actor1);

    // Actor with absolute role (starting with /)
    let mut actor2 = Actor::new(
        "spy_2",
        "Master spy",
        "/Game/BP_MasterSpy.BP_MasterSpy_C",
        "sector_7",
    );
    actor2.placement.position = Vector3::new(30.0, 40.0, 0.0);
    spec.actors.push(actor2);

    // Object with relative class
    let mut obj1 = Object::new("term_1", "Console", "Terminal", "sector_7");
    obj1.placement.position = Vector3::new(-10.0, -20.0, 0.0);
    spec.objects.push(obj1);

    let t3d = LayoutCompiler::compile(&spec);

    // Assert places floor bounds and coordinates
    assert!(t3d.contains("Begin Map"));
    assert!(t3d.contains("Place_sector_7"));
    assert!(t3d.contains("ActorLabel=\"Floor_Sector Seven Floor\""));
    // Z coordinate = center.z - half_extents.z - 50.0 => 50.0 - 50.0 - 50.0 = -50.0
    assert!(t3d.contains("RelativeLocation=(X=100.000000,Y=-100.000000,Z=-50.000000)"));
    // Scale X = half_extents.x / 50.0 => 200 / 50 = 4.0; Scale Y = 4.0; Scale Z = 1.0
    assert!(t3d.contains("RelativeScale3D=(X=4.000000,Y=4.000000,Z=1.000000)"));

    // Assert actors
    assert!(t3d.contains("Actor_spy_1"));
    assert!(t3d.contains("BP_Agent_C"));
    assert!(t3d.contains("/Game/BP_Agent.Default__BP_Agent_C"));
    assert!(t3d.contains("ActorLabel=\"Infiltrator\""));

    assert!(t3d.contains("Actor_spy_2"));
    assert!(t3d.contains("BP_MasterSpy_C"));
    assert!(t3d.contains("/Game/BP_MasterSpy.Default__BP_MasterSpy_C"));

    // Assert objects
    assert!(t3d.contains("Object_term_1"));
    assert!(t3d.contains("BP_Terminal_C"));
}

#[test]
fn test_world_evolver_complex_intent() {
    // Initialize with intent
    let init_intent = r#"
        create place room_a name "Room A" at (0.0, 0.0, 0.0) bounds (50.0, 50.0, 50.0)
        create actor operator name "Operator 1" role HumanOperator in room_a
    "#;
    let spec = parse_intent(init_intent).unwrap();

    // Evolve: add object, delete actor, update place description
    let modification = r#"
        create object console name "Control Console" class Console in room_a
        delete actor operator
    "#;
    let mut evolved = WorldEvolver::evolve(&spec, modification).unwrap();

    assert_eq!(evolved.places.len(), 1);
    assert_eq!(evolved.actors.len(), 0);
    assert_eq!(evolved.objects.len(), 1);
    assert_eq!(evolved.objects[0].id, "console");

    // Evolve again with receipt chaining
    ReceiptChainManager::generate_receipt_chain(&mut evolved, b"test_salt").unwrap();
    assert!(ReceiptChainManager::verify_receipt_chain(
        &evolved,
        b"test_salt"
    ));
}

#[test]
#[ignore = "requires UE4_ROOT pointing to a built engine with RunUAT.sh (takes hours); \
            run manually: cargo test -p genie-core -- --ignored test_deployment_manager_files_and_logs"]
fn test_deployment_manager_files_and_logs() {
    // Use the real engine root if set; fall back only for manual invocation
    if std::env::var("UE4_ROOT").is_err() {
        std::env::set_var("UE4_ROOT", "/Users/sac/ue-4.27-html5-es3");
    }
    let mut spec = WorldSpec::new();
    let bounds = Bounds3D::new(
        Vector3::new(10.0, 10.0, 10.0),
        Vector3::new(10.0, 10.0, 10.0),
    );
    spec.places.push(Place::new("p1", "Test Room", bounds));

    let project_root = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    let manufactured_dir = project_root.join("pwa-staff").join("manufactured");
    let log_path = manufactured_dir.join("deploy.log");

    // Ensure destination directory exists and is clear of old test files
    let _ = fs::create_dir_all(&manufactured_dir);
    let spec_json_path = manufactured_dir.join("spec.json");
    let receipt_path = manufactured_dir.join("receipt.json");

    let html_path = manufactured_dir.join("Brm-HTML5-Shipping.html");
    let js_path = manufactured_dir.join("Brm-HTML5-Shipping.js");
    let wasm_path = manufactured_dir.join("Brm-HTML5-Shipping.wasm");
    let data_path = manufactured_dir.join("Brm-HTML5-Shipping.data");

    let files_to_clean = [
        &log_path,
        &spec_json_path,
        &receipt_path,
        &html_path,
        &js_path,
        &wasm_path,
        &data_path,
    ];

    for file in &files_to_clean {
        if file.exists() {
            let _ = fs::remove_file(file);
        }
    }

    // Deploy
    let res = DeploymentManager::deploy(&spec, &log_path);
    assert!(res.is_ok(), "Deployment failed: {:?}", res.err());

    // 1. Verify deployment log append
    assert!(log_path.exists(), "deploy.log was not created");
    let log_content = fs::read_to_string(&log_path).unwrap();
    assert!(
        log_content.contains("Genie 26 Deployment Log"),
        "Log missing header"
    );
    assert!(log_content.contains("Place: p1"), "Log missing Place info");
    assert!(
        log_content.contains("Pipeline Status: SUCCESS"),
        "Log missing SUCCESS status"
    );

    // 2. Verify spec.json output
    assert!(spec_json_path.exists(), "spec.json was not created");
    let spec_content = fs::read_to_string(&spec_json_path).unwrap();
    assert!(
        spec_content.contains("p1"),
        "spec.json does not contain place id 'p1'"
    );
    assert!(
        spec_content.contains("Test Room"),
        "spec.json does not contain place name 'Test Room'"
    );

    // 3. Verify receipt.json output
    assert!(receipt_path.exists(), "receipt.json was not created");
    let receipt_content = fs::read_to_string(&receipt_path).unwrap();
    assert!(
        receipt_content.contains("success"),
        "receipt.json status is not success"
    );

    // 4. Verify generated staged HTML5/WASM files under pwa-staff/manufactured
    assert!(html_path.exists(), "HTML5 file was not generated");
    assert!(js_path.exists(), "JS helper file was not generated");
    assert!(wasm_path.exists(), "WASM binary file was not generated");
    assert!(data_path.exists(), "Data pack file was not generated");

    // Clean up
    for file in &files_to_clean {
        let _ = fs::remove_file(file);
    }
}

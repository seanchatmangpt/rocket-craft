use genie_core::{
    deployment::DeploymentManager,
    evolution::WorldEvolver,
    layout::LayoutCompiler,
    parse_intent,
    spec::{Actor, Bounds3D, Object, Place, Vector3, WorldSpec},
};
use std::fs;

#[test]
fn test_milestone3_layout_compiler() {
    let mut spec = WorldSpec::new();
    let bounds = Bounds3D::new(
        Vector3::new(10.0, 20.0, 30.0),
        Vector3::new(50.0, 50.0, 50.0),
    );
    spec.places
        .push(Place::new("room_1", "Control Room", bounds));

    let mut actor = Actor::new("bot_1", "Welder", "Robot", "room_1");
    actor.placement.position = Vector3::new(5.0, 5.0, 0.0);
    actor.placement.rotation = Vector3::new(0.0, 90.0, 0.0);
    spec.actors.push(actor);

    let mut object = Object::new("cnc_1", "CNC Alpha", "CNC_Machine", "room_1");
    object.placement.position = Vector3::new(-5.0, -5.0, 0.0);
    spec.objects.push(object);

    let t3d = LayoutCompiler::compile(&spec);

    assert!(
        t3d.contains("Begin Map"),
        "T3D map does not contain 'Begin Map'"
    );
    assert!(
        t3d.contains("Begin Level"),
        "T3D map does not contain 'Begin Level'"
    );
    assert!(
        t3d.contains("Place_room_1"),
        "T3D map does not contain place room_1"
    );
    assert!(
        t3d.contains("Actor_bot_1"),
        "T3D map does not contain actor bot_1"
    );
    assert!(
        t3d.contains("Object_cnc_1"),
        "T3D map does not contain object cnc_1"
    );
    assert!(
        t3d.contains("Cube.Cube"),
        "T3D map does not contain Cube mesh reference"
    );
    assert!(
        t3d.contains("Cylinder.Cylinder"),
        "T3D map does not contain Cylinder mesh reference"
    );
    assert!(
        t3d.contains("Sphere.Sphere"),
        "T3D map does not contain Sphere mesh reference"
    );
    assert!(
        t3d.contains("End Level"),
        "T3D map does not contain 'End Level'"
    );
    assert!(
        t3d.contains("End Map"),
        "T3D map does not contain 'End Map'"
    );
}

#[test]
fn test_milestone4_deployment_manager() {
    if std::env::var("UE4_ROOT").is_err() {
        std::env::set_var("UE4_ROOT", "/Users/sac/ue4-sim");
    }
    let mut spec = WorldSpec::new();
    let bounds = Bounds3D::new(Vector3::new(0.0, 0.0, 0.0), Vector3::new(10.0, 10.0, 10.0));
    spec.places
        .push(Place::new("room_1", "Control Room", bounds));

    let temp_dir = std::env::temp_dir();
    let log_path = temp_dir.join("genie_deploy_test.log");

    let res = DeploymentManager::deploy(&spec, &log_path);
    assert!(res.is_ok(), "Deployment logging failed: {:?}", res.err());
    assert!(log_path.exists(), "Deployment log file was not created");

    let log_content = fs::read_to_string(&log_path).unwrap();
    assert!(
        log_content.contains("Genie 26 Deployment Log"),
        "Log missing header"
    );
    assert!(
        log_content.contains("Place: room_1"),
        "Log missing Place info"
    );

    // Clean up
    let _ = fs::remove_file(log_path);
}

#[test]
fn test_milestone5_world_evolver() {
    // Start with an initial spec
    let intent = r#"
        create place room_1 name "Control Room" at (0.0, 0.0, 0.0) bounds (10.0, 10.0, 10.0)
        create actor bot_1 name "Welder Bot" role RoboticWelder in room_1
    "#;
    let spec = parse_intent(intent).unwrap();
    assert_eq!(spec.places.len(), 1);
    assert_eq!(spec.actors.len(), 1);

    // Evolve: add a new place room_2, update bot_1 position, delete nothing
    let modification = r#"
        create place room_2 name "Storage Room" at (100.0, 0.0, 0.0) bounds (20.0, 20.0, 10.0)
        update actor bot_1 position (10.0, 20.0, 30.0)
        update actor bot_1 rotation (0.0, 180.0, 0.0)
    "#;

    let evolved = WorldEvolver::evolve(&spec, modification).unwrap();

    // Verify places
    assert_eq!(evolved.places.len(), 2, "Expected 2 places after evolution");
    assert!(evolved.places.iter().any(|p| p.id == "room_1"));
    assert!(evolved.places.iter().any(|p| p.id == "room_2"));

    // Verify actor update
    let evolved_bot = evolved.actors.iter().find(|a| a.id == "bot_1").unwrap();
    assert_eq!(
        evolved_bot.placement.position,
        Vector3::new(10.0, 20.0, 30.0)
    );
    assert_eq!(
        evolved_bot.placement.rotation,
        Vector3::new(0.0, 180.0, 0.0)
    );

    // Verify history events
    assert_eq!(
        evolved.history.len(),
        1,
        "Expected 1 history event from evolution"
    );
    assert_eq!(evolved.history[0].activity, "Evolve");
    assert!(evolved.history[0]
        .details
        .contains_key("modification_intent"));

    // Verify receipt chain
    assert!(
        !evolved.receipts.is_empty(),
        "Expected receipts to be generated"
    );

    // Delete bot_1 and room_2
    let delete_mod = "delete actor bot_1\ndelete place room_2";
    let evolved_deleted = WorldEvolver::evolve(&evolved, delete_mod).unwrap();
    assert_eq!(evolved_deleted.places.len(), 1);
    assert_eq!(evolved_deleted.actors.len(), 0);
}

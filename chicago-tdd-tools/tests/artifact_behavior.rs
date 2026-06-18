use genie_core::spec::{WorldSpec, Place, Bounds3D, Vector3, Actor, Placement};
use genie_core::layout::LayoutCompiler;

#[test]
fn should_compile_world_spec_to_t3d_map() {
    // 1. Setup Data (SUT state)
    let mut spec = WorldSpec::new();
    
    let room = Place::new("room_test", "Test Room", Bounds3D {
        center: Vector3 { x: 0.0, y: 0.0, z: 0.0 },
        half_extents: Vector3 { x: 100.0, y: 100.0, z: 50.0 },
    });
    spec.places.push(room);

    let mut welder = Actor::new("bot_1", "Welder", "RoboticWelder", "room_test");
    welder.placement = Placement {
        position: Vector3 { x: 10.0, y: 10.0, z: 0.0 },
        rotation: Vector3 { x: 0.0, y: 0.0, z: 0.0 },
    };
    spec.actors.push(welder);

    // 2. Act
    let t3d_output = LayoutCompiler::compile(&spec);

    // 3. Assert (Behavioral verification: The output must contain the map structure and the specific actors)
    assert!(t3d_output.starts_with("Begin Map"));
    assert!(t3d_output.contains("   Begin Level"));
    
    // Verify Place actor exists with proper T3D structure
    assert!(t3d_output.contains("      Begin Actor Class=StaticMeshActor Name=Place_room_test"));
    assert!(t3d_output.contains("         Begin Object Name=\"StaticMeshComponent0\""));
    assert!(t3d_output.contains("            RelativeLocation=(X=0.000000,Y=0.000000,Z=-100.000000)"));
    assert!(t3d_output.contains("            RelativeScale3D=(X=2.000000,Y=2.000000,Z=1.000000)"));
    assert!(t3d_output.contains("         End Object"));
    assert!(t3d_output.contains("         ActorLabel=\"Floor_Test Room\""));
    assert!(t3d_output.contains("      End Actor"));

    // Verify Welder bot exists with proper T3D structure
    assert!(t3d_output.contains("      Begin Actor Class=BP_RoboticWelder_C Name=Actor_bot_1 Archetype=BP_RoboticWelder_C'/Game/BP_RoboticWelder.Default__BP_RoboticWelder_C'"));
    assert!(t3d_output.contains("         RelativeLocation=(X=10.000000,Y=10.000000,Z=0.000000)"));
    assert!(t3d_output.contains("         ActorLabel=\"Welder\""));
    assert!(t3d_output.contains("      End Actor"));

    assert!(t3d_output.contains("   End Level"));
    assert!(t3d_output.ends_with("End Map\n"));
}

#[test]
fn should_handle_relative_positioning_of_actors_in_different_places() {
    let mut spec = WorldSpec::new();
    
    // Place 1 at origin
    let room1 = Place::new("room_1", "Room 1", Bounds3D {
        center: Vector3 { x: 0.0, y: 0.0, z: 0.0 },
        half_extents: Vector3 { x: 100.0, y: 100.0, z: 50.0 },
    });
    spec.places.push(room1);

    // Place 2 shifted
    let room2 = Place::new("room_2", "Room 2", Bounds3D {
        center: Vector3 { x: 500.0, y: 0.0, z: 0.0 },
        half_extents: Vector3 { x: 100.0, y: 100.0, z: 50.0 },
    });
    spec.places.push(room2);

    // Actor in room 1
    let mut actor1 = Actor::new("actor_1", "Actor 1", "RoboticWelder", "room_1");
    actor1.placement.position = Vector3 { x: 50.0, y: 0.0, z: 0.0 };
    spec.actors.push(actor1);

    // Actor in room 2
    let mut actor2 = Actor::new("actor_2", "Actor 2", "RoboticWelder", "room_2");
    actor2.placement.position = Vector3 { x: 50.0, y: 0.0, z: 0.0 };
    spec.actors.push(actor2);

    let t3d_output = LayoutCompiler::compile(&spec);

    // Actor 1 should be at (0+50, 0+0, 0+0) = (50, 0, 0)
    assert!(t3d_output.contains("Begin Actor Class=BP_RoboticWelder_C Name=Actor_actor_1"));
    assert!(t3d_output.contains("RelativeLocation=(X=50.000000,Y=0.000000,Z=0.000000)"));

    // Actor 2 should be at (500+50, 0+0, 0+0) = (550, 0, 0)
    assert!(t3d_output.contains("Begin Actor Class=BP_RoboticWelder_C Name=Actor_actor_2"));
    assert!(t3d_output.contains("RelativeLocation=(X=550.000000,Y=0.000000,Z=0.000000)"));
}

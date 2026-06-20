use genie_core::layout::LayoutCompiler;
use genie_core::spec::{Actor, Bounds3D, Place, Placement, Vector3, WorldSpec};

#[test]
fn should_compile_world_spec_to_t3d_map() {
    // 1. Setup Data (SUT state)
    let mut spec = WorldSpec::new();

    let room = Place::new(
        "room_test",
        "Test Room",
        Bounds3D {
            center: Vector3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            half_extents: Vector3 {
                x: 100.0,
                y: 100.0,
                z: 50.0,
            },
        },
    );
    spec.places.push(room);

    let mut welder = Actor::new("bot_1", "Welder", "RoboticWelder", "room_test");
    welder.placement = Placement {
        position: Vector3 {
            x: 10.0,
            y: 10.0,
            z: 0.0,
        },
        rotation: Vector3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        },
    };
    spec.actors.push(welder);

    // 2. Act
    let t3d_output = LayoutCompiler::compile(&spec);

    // 3. Assert - Verify exact T3D output structure
    // Floor Z: center.z(0) - half_extents.z(50) - 50 = -100
    // Scale: half_extents.x/50=2, half_extents.y/50=2, z=1
    let expected_place_actor = concat!(
        "      Begin Actor Class=StaticMeshActor Name=Place_room_test Archetype=StaticMeshActor'/Script/Engine.Default__StaticMeshActor'\n",
        "         Begin Object Class=StaticMeshComponent Name=StaticMeshComponent0 ObjName=StaticMeshComponent0 Archetype=StaticMeshComponent'/Script/Engine.Default__StaticMeshActor:StaticMeshComponent0'\n",
        "         End Object\n",
        "         Begin Object Name=\"StaticMeshComponent0\"\n",
        "            StaticMesh=StaticMesh'/Engine/BasicShapes/Cube.Cube'\n",
        "            RelativeLocation=(X=0.000000,Y=0.000000,Z=-100.000000)\n",
        "            RelativeRotation=(Pitch=0.000000,Yaw=0.000000,Roll=0.000000)\n",
        "            RelativeScale3D=(X=2.000000,Y=2.000000,Z=1.000000)\n",
        "         End Object\n",
        "         StaticMeshComponent=StaticMeshComponent0\n",
        "         RootComponent=StaticMeshComponent0\n",
        "         ActorLabel=\"Floor_Test Room\"\n",
        "      End Actor\n",
    );

    // Actor absolute position: parent_center(0,0,0) + relative(10,10,0) = (10,10,0)
    let expected_welder_actor = concat!(
        "      Begin Actor Class=BP_RoboticWelder_C Name=Actor_bot_1 Archetype=BP_RoboticWelder_C'/Game/BP_RoboticWelder.Default__BP_RoboticWelder_C'\n",
        "         Begin Object Class=StaticMeshComponent Name=StaticMeshComponent0 ObjName=StaticMeshComponent0\n",
        "         End Object\n",
        "         Begin Object Name=\"StaticMeshComponent0\"\n",
        "            StaticMesh=StaticMesh'/Engine/BasicShapes/Cylinder.Cylinder'\n",
        "            RelativeLocation=(X=10.000000,Y=10.000000,Z=0.000000)\n",
        "            RelativeRotation=(Pitch=0.000000,Yaw=0.000000,Roll=0.000000)\n",
        "            RelativeScale3D=(X=1.000000,Y=1.000000,Z=2.000000)\n",
        "         End Object\n",
        "         StaticMeshComponent=StaticMeshComponent0\n",
        "         RootComponent=StaticMeshComponent0\n",
        "         ActorLabel=\"Welder\"\n",
        "      End Actor\n",
    );

    let expected = format!(
        "Begin Map\n   Begin Level\n{expected_place_actor}{expected_welder_actor}   End Level\nEnd Map\n"
    );

    assert_eq!(t3d_output, expected);
}

#[test]
fn should_handle_relative_positioning_of_actors_in_different_places() {
    let mut spec = WorldSpec::new();

    // Place 1 at origin
    let room1 = Place::new(
        "room_1",
        "Room 1",
        Bounds3D {
            center: Vector3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            half_extents: Vector3 {
                x: 100.0,
                y: 100.0,
                z: 50.0,
            },
        },
    );
    spec.places.push(room1);

    // Place 2 shifted
    let room2 = Place::new(
        "room_2",
        "Room 2",
        Bounds3D {
            center: Vector3 {
                x: 500.0,
                y: 0.0,
                z: 0.0,
            },
            half_extents: Vector3 {
                x: 100.0,
                y: 100.0,
                z: 50.0,
            },
        },
    );
    spec.places.push(room2);

    // Actor in room 1
    let mut actor1 = Actor::new("actor_1", "Actor 1", "RoboticWelder", "room_1");
    actor1.placement.position = Vector3 {
        x: 50.0,
        y: 0.0,
        z: 0.0,
    };
    spec.actors.push(actor1);

    // Actor in room 2: absolute X = parent_center.x(500) + relative.x(50) = 550
    let mut actor2 = Actor::new("actor_2", "Actor 2", "RoboticWelder", "room_2");
    actor2.placement.position = Vector3 {
        x: 50.0,
        y: 0.0,
        z: 0.0,
    };
    spec.actors.push(actor2);

    let t3d_output = LayoutCompiler::compile(&spec);

    // Verify actor2's absolute X position is 550 (parent 500 + relative 50)
    // Extract the RelativeLocation line for Actor_actor_2 by parsing the output structurally
    let actor2_block_start = t3d_output
        .find("Name=Actor_actor_2")
        .expect("Actor_actor_2 must appear in output");
    let actor2_block = &t3d_output[actor2_block_start..];
    let location_line_start = actor2_block
        .find("RelativeLocation=")
        .expect("RelativeLocation must appear in Actor_actor_2 block");
    let location_line_end = actor2_block[location_line_start..]
        .find('\n')
        .expect("RelativeLocation line must end with newline");
    let location_line = &actor2_block[location_line_start..location_line_start + location_line_end];

    assert_eq!(
        location_line,
        "RelativeLocation=(X=550.000000,Y=0.000000,Z=0.000000)"
    );
}

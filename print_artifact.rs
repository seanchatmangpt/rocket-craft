use genie_core::spec::{WorldSpec, Place, Bounds3D, Vector3, Actor, Placement};
use genie_core::layout::LayoutCompiler;

fn main() {
    let mut spec = WorldSpec::new();
    let room = Place::new("room_test", "Test Room", Bounds3D { center: Vector3 { x: 0.0, y: 0.0, z: 0.0 }, half_extents: Vector3 { x: 100.0, y: 100.0, z: 50.0 } });
    spec.places.push(room);
    let mut welder = Actor::new("bot_1", "Welder", "RoboticWelder", "room_test");
    welder.placement = Placement { position: Vector3 { x: 10.0, y: 10.0, z: 0.0 }, rotation: Vector3 { x: 0.0, y: 0.0, z: 0.0 } };
    spec.actors.push(welder);
    println!("--- TEST 1 ---");
    println!("{}", LayoutCompiler::compile(&spec));

    let mut spec2 = WorldSpec::new();
    let room1 = Place::new("room_1", "Room 1", Bounds3D { center: Vector3 { x: 0.0, y: 0.0, z: 0.0 }, half_extents: Vector3 { x: 100.0, y: 100.0, z: 50.0 } });
    spec2.places.push(room1);
    let room2 = Place::new("room_2", "Room 2", Bounds3D { center: Vector3 { x: 500.0, y: 0.0, z: 0.0 }, half_extents: Vector3 { x: 100.0, y: 100.0, z: 50.0 } });
    spec2.places.push(room2);
    let mut actor1 = Actor::new("actor_1", "Actor 1", "RoboticWelder", "room_1");
    actor1.placement.position = Vector3 { x: 50.0, y: 0.0, z: 0.0 };
    spec2.actors.push(actor1);
    let mut actor2 = Actor::new("actor_2", "Actor 2", "RoboticWelder", "room_2");
    actor2.placement.position = Vector3 { x: 50.0, y: 0.0, z: 0.0 };
    spec2.actors.push(actor2);
    println!("--- TEST 2 ---");
    println!("{}", LayoutCompiler::compile(&spec2));
}

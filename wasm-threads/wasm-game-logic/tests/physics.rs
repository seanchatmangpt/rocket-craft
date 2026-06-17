use chicago_tdd_tools::{Logger, TuiBufferSink};
use proptest::prelude::*;
use wasm_game_logic::{PhysicsSystem, Position, Velocity, World};

fn log() -> Logger {
    let mut l = Logger::new();
    let (sink, _) = TuiBufferSink::new();
    l.add_sink(Box::new(sink));
    l
}

#[test]
fn physics_moves_entity_by_velocity() {
    let mut log = log();
    log.info("Given a World with an entity at (0,0) and velocity (10,0)");
    let mut world = World::new();
    let e = world.spawn();
    world.add_position(e, Position { x: 0.0, y: 0.0 });
    world.add_velocity(e, Velocity { dx: 10.0, dy: 0.0 });

    log.info("When PhysicsSystem::run is called with delta_ms=1000 (1 second)");
    PhysicsSystem::run(&mut world, 1000);

    log.info("Then position x is 10.0 and y is 0.0");
    let pos = world.get_position(e).unwrap();
    assert!((pos.x - 10.0).abs() < 0.001);
    assert!((pos.y - 0.0).abs() < 0.001);
}

#[test]
fn position_depends_on_velocity_not_constant() {
    let mut log = log();
    log.info("Given two entities at the same origin with different velocities");
    let mut world = World::new();
    let e1 = world.spawn();
    let e2 = world.spawn();
    world.add_position(e1, Position { x: 0.0, y: 0.0 });
    world.add_position(e2, Position { x: 0.0, y: 0.0 });
    world.add_velocity(e1, Velocity { dx: 5.0, dy: 0.0 });
    world.add_velocity(e2, Velocity { dx: 20.0, dy: 0.0 });

    log.info("When PhysicsSystem::run is called");
    PhysicsSystem::run(&mut world, 1000);

    log.info("Then the entities end up at different x positions — falsifies constant-position mock");
    let x1 = world.get_position(e1).unwrap().x;
    let x2 = world.get_position(e2).unwrap().x;
    assert_ne!(x1, x2, "different velocities must produce different positions");
}

#[test]
fn entity_without_velocity_stays_still() {
    let mut log = log();
    log.info("Given a World with an entity at (3,7) and no Velocity component");
    let mut world = World::new();
    let e = world.spawn();
    world.add_position(e, Position { x: 3.0, y: 7.0 });

    log.info("When PhysicsSystem::run is called");
    PhysicsSystem::run(&mut world, 1000);

    log.info("Then the position is unchanged");
    let pos = world.get_position(e).unwrap();
    assert!((pos.x - 3.0).abs() < 0.001);
    assert!((pos.y - 7.0).abs() < 0.001);
}

#[test]
fn physics_applies_both_axes() {
    let mut log = log();
    log.info("Given an entity at (0,0) with velocity (3,4)");
    let mut world = World::new();
    let e = world.spawn();
    world.add_position(e, Position { x: 0.0, y: 0.0 });
    world.add_velocity(e, Velocity { dx: 3.0, dy: 4.0 });

    log.info("When physics runs for 2 seconds (2000 ms)");
    PhysicsSystem::run(&mut world, 2000);

    log.info("Then x=6 and y=8");
    let pos = world.get_position(e).unwrap();
    assert!((pos.x - 6.0).abs() < 0.001);
    assert!((pos.y - 8.0).abs() < 0.001);
}

#[test]
fn physics_handles_negative_velocity() {
    let mut log = log();
    log.info("Given an entity at (10,10) with velocity (-5,-2)");
    let mut world = World::new();
    let e = world.spawn();
    world.add_position(e, Position { x: 10.0, y: 10.0 });
    world.add_velocity(e, Velocity { dx: -5.0, dy: -2.0 });

    log.info("When physics runs for 1 second");
    PhysicsSystem::run(&mut world, 1000);

    log.info("Then position is (5, 8)");
    let pos = world.get_position(e).unwrap();
    assert!((pos.x - 5.0).abs() < 0.001);
    assert!((pos.y - 8.0).abs() < 0.001);
}

// ── Property-based tests ──────────────────────────────────────────────────────

proptest! {
    #[test]
    fn physics_position_is_deterministic(
        x0 in -1000.0f32..1000.0,
        y0 in -1000.0f32..1000.0,
        vx in -100.0f32..100.0,
        vy in -100.0f32..100.0,
        dt in 1u64..5000,
    ) {
        let mut world1 = World::new();
        let mut world2 = World::new();
        let e1 = world1.spawn();
        let e2 = world2.spawn();
        world1.add_position(e1, Position { x: x0, y: y0 });
        world2.add_position(e2, Position { x: x0, y: y0 });
        world1.add_velocity(e1, Velocity { dx: vx, dy: vy });
        world2.add_velocity(e2, Velocity { dx: vx, dy: vy });
        PhysicsSystem::run(&mut world1, dt);
        PhysicsSystem::run(&mut world2, dt);
        let p1 = world1.get_position(e1).unwrap();
        let p2 = world2.get_position(e2).unwrap();
        prop_assert!((p1.x - p2.x).abs() < 0.001);
        prop_assert!((p1.y - p2.y).abs() < 0.001);
    }
}

use chicago_tdd_tools::{Logger, TuiBufferSink};
use proptest::prelude::*;
use wasm_game_logic::{Attack, Entity, Health, Player, Position, Velocity, World};

fn log() -> Logger {
    let mut l = Logger::new();
    let (sink, _) = TuiBufferSink::new();
    l.add_sink(Box::new(sink));
    l
}

#[test]
fn spawning_entity_increases_world_count() {
    let log = log();
    log.info("Given an empty World");
    let mut world = World::new();
    assert_eq!(world.entity_count(), 0);

    log.info("When we spawn an entity");
    world.spawn();

    log.info("Then entity count increases to 1");
    assert_eq!(world.entity_count(), 1);

    log.info("When we spawn a second entity");
    world.spawn();

    log.info("Then entity count increases to 2");
    assert_eq!(world.entity_count(), 2);
}

#[test]
fn despawning_entity_decreases_world_count() {
    let log = log();
    log.info("Given a World with one spawned entity");
    let mut world = World::new();
    let e = world.spawn();

    log.info("When we despawn that entity");
    world.despawn(e);

    log.info("Then entity count returns to 0 and entity is not alive");
    assert_eq!(world.entity_count(), 0);
    assert!(!world.is_alive(e));
}

#[test]
fn component_roundtrip_health() {
    let log = log();
    log.info("Given a World with one entity");
    let mut world = World::new();
    let e = world.spawn();

    log.info("When we add a Health component with max 100");
    world.add_health(e, Health::new(100));

    log.info("Then we can retrieve the component with correct values");
    let hp = world.get_health(e).unwrap();
    assert_eq!(hp.current, 100);
    assert_eq!(hp.max, 100);
}

#[test]
fn despawned_entity_components_are_removed() {
    let log = log();
    log.info("Given a World with an entity that has Health and Position");
    let mut world = World::new();
    let e = world.spawn();
    world.add_health(e, Health::new(50));
    world.add_position(e, Position { x: 1.0, y: 2.0 });

    log.info("When we despawn that entity");
    world.despawn(e);

    log.info("Then all components for that entity are gone");
    assert!(world.get_health(e).is_none());
    assert!(world.get_position(e).is_none());
}

#[test]
fn entities_with_position_query_returns_alive_only() {
    let log = log();
    log.info("Given a World with two entities that have Position");
    let mut world = World::new();
    let e1 = world.spawn();
    let e2 = world.spawn();
    world.add_position(e1, Position { x: 0.0, y: 0.0 });
    world.add_position(e2, Position { x: 1.0, y: 1.0 });

    log.info("When we despawn one of them");
    world.despawn(e1);

    log.info("Then the position query returns only the alive entity");
    let with_pos: Vec<Entity> = world.entities_with_position().collect();
    assert_eq!(with_pos.len(), 1);
    assert!(with_pos.contains(&e2));
}

#[test]
fn entities_with_health_query_returns_alive_only() {
    let log = log();
    log.info("Given a World with two entities that have Health");
    let mut world = World::new();
    let e1 = world.spawn();
    let e2 = world.spawn();
    world.add_health(e1, Health::new(100));
    world.add_health(e2, Health::new(50));

    log.info("When we despawn e1");
    world.despawn(e1);

    log.info("Then entities_with_health yields only e2");
    let with_hp: Vec<Entity> = world.entities_with_health().collect();
    assert_eq!(with_hp.len(), 1);
    assert!(with_hp.contains(&e2));
}

#[test]
fn add_and_mutate_velocity_component() {
    let log = log();
    log.info("Given a World with an entity that has Velocity(0,0)");
    let mut world = World::new();
    let e = world.spawn();
    world.add_velocity(e, Velocity { dx: 0.0, dy: 0.0 });

    log.info("When we mutably update the velocity");
    if let Some(vel) = world.get_velocity_mut(e) {
        vel.dx = 5.0;
        vel.dy = -3.0;
    }

    log.info("Then the stored velocity reflects the change");
    let vel = world.get_velocity(e).unwrap();
    assert!((vel.dx - 5.0).abs() < f32::EPSILON);
    assert!((vel.dy - (-3.0)).abs() < f32::EPSILON);
}

#[test]
fn add_and_retrieve_player_component() {
    let log = log();
    log.info("Given a World with one entity");
    let mut world = World::new();
    let e = world.spawn();

    log.info("When we attach a Player component");
    world.add_player(
        e,
        Player {
            name: "Tester".into(),
            score: 42,
        },
    );

    log.info("Then the player name and score can be retrieved");
    let p = world.get_player(e).unwrap();
    assert_eq!(p.name, "Tester");
    assert_eq!(p.score, 42);
}

#[test]
fn add_and_retrieve_attack_component() {
    let log = log();
    log.info("Given a World with one entity");
    let mut world = World::new();
    let e = world.spawn();

    log.info("When we attach an Attack component");
    world.add_attack(
        e,
        Attack {
            damage: 15,
            range: 2.5,
            cooldown_ms: 500,
        },
    );

    log.info("Then the attack stats can be retrieved");
    let atk = world.get_attack(e).unwrap();
    assert_eq!(atk.damage, 15);
    assert!((atk.range - 2.5).abs() < f32::EPSILON);
    assert_eq!(atk.cooldown_ms, 500);
}

// ── Property-based tests ──────────────────────────────────────────────────────

proptest! {
    #[test]
    fn entity_count_never_negative_after_operations(
        spawns in 1usize..20,
        despawns in 0usize..20,
    ) {
        let mut world = World::new();
        let entities: Vec<Entity> = (0..spawns).map(|_| world.spawn()).collect();
        let to_despawn = despawns.min(spawns);
        for &e in entities.iter().take(to_despawn) {
            world.despawn(e);
        }
        prop_assert_eq!(world.entity_count(), spawns - to_despawn);
    }
}

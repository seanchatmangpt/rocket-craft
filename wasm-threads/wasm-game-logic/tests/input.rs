use chicago_tdd_tools::{Logger, TuiBufferSink};
use wasm_game_logic::{Attack, Health, InputCommand, InputSystem, Velocity, World};

fn log() -> Logger {
    let mut l = Logger::new();
    let (sink, _) = TuiBufferSink::new();
    l.add_sink(Box::new(sink));
    l
}

#[test]
fn input_move_updates_velocity() {
    let mut log = log();
    log.info("Given a World with an entity that has Velocity(0,0)");
    let mut world = World::new();
    let e = world.spawn();
    world.add_velocity(e, Velocity { dx: 0.0, dy: 0.0 });

    log.info("When InputCommand::Move is processed with dx=3.5, dy=-1.2");
    InputSystem::process(
        &mut world,
        InputCommand::Move {
            entity: e.0,
            dx: 3.5,
            dy: -1.2,
        },
    );

    log.info("Then the stored velocity is updated to (3.5, -1.2)");
    let vel = world.get_velocity(e).unwrap();
    assert!((vel.dx - 3.5).abs() < 0.001);
    assert!((vel.dy - (-1.2)).abs() < 0.001);
}

#[test]
fn input_move_is_no_op_when_entity_has_no_velocity_component() {
    let mut log = log();
    log.info("Given a World with an entity that has no Velocity component");
    let mut world = World::new();
    let e = world.spawn();

    log.info("When InputCommand::Move is processed");
    InputSystem::process(
        &mut world,
        InputCommand::Move {
            entity: e.0,
            dx: 10.0,
            dy: 10.0,
        },
    );

    log.info("Then no velocity component is created and no panic occurs");
    assert!(world.get_velocity(e).is_none());
}

#[test]
fn input_attack_reduces_target_health() {
    let mut log = log();
    log.info("Given a World with an attacker (damage=25) and a target (HP=100)");
    let mut world = World::new();
    let attacker = world.spawn();
    let target = world.spawn();
    world.add_attack(
        attacker,
        Attack {
            damage: 25,
            range: 1.0,
            cooldown_ms: 0,
        },
    );
    world.add_health(target, Health::new(100));

    log.info("When InputCommand::Attack is processed");
    InputSystem::process(
        &mut world,
        InputCommand::Attack {
            attacker: attacker.0,
            target: target.0,
        },
    );

    log.info("Then target HP is reduced to 75");
    assert_eq!(world.get_health(target).unwrap().current, 75);
}

#[test]
fn input_use_item_1_heals_entity_by_ten() {
    let mut log = log();
    log.info("Given a World with an entity at 50 HP (max 100)");
    let mut world = World::new();
    let e = world.spawn();
    let mut hp = Health::new(100);
    hp.apply_damage(50);
    world.add_health(e, hp);

    log.info("When InputCommand::UseItem with item_id=1 is processed");
    InputSystem::process(
        &mut world,
        InputCommand::UseItem {
            entity: e.0,
            item_id: 1,
        },
    );

    log.info("Then HP is healed by 10 to become 60");
    assert_eq!(world.get_health(e).unwrap().current, 60);
}

#[test]
fn input_use_item_unknown_id_does_nothing() {
    let mut log = log();
    log.info("Given a World with an entity at 50 HP (max 100)");
    let mut world = World::new();
    let e = world.spawn();
    let mut hp = Health::new(100);
    hp.apply_damage(50);
    world.add_health(e, hp);

    log.info("When InputCommand::UseItem with an unknown item_id=99 is processed");
    InputSystem::process(
        &mut world,
        InputCommand::UseItem {
            entity: e.0,
            item_id: 99,
        },
    );

    log.info("Then HP is unchanged at 50");
    assert_eq!(world.get_health(e).unwrap().current, 50);
}

#[test]
fn input_use_item_1_does_not_exceed_max_health() {
    let mut log = log();
    log.info("Given an entity at full health (100/100)");
    let mut world = World::new();
    let e = world.spawn();
    world.add_health(e, Health::new(100));

    log.info("When InputCommand::UseItem with item_id=1 is processed");
    InputSystem::process(
        &mut world,
        InputCommand::UseItem {
            entity: e.0,
            item_id: 1,
        },
    );

    log.info("Then HP is still capped at 100");
    assert_eq!(world.get_health(e).unwrap().current, 100);
}

#[test]
fn input_attack_with_no_attacker_component_does_not_crash() {
    let mut log = log();
    log.info("Given a World with an attacker missing an Attack component and a target at 100 HP");
    let mut world = World::new();
    let attacker = world.spawn();
    let target = world.spawn();
    world.add_health(target, Health::new(100));

    log.info("When InputCommand::Attack is processed");
    InputSystem::process(
        &mut world,
        InputCommand::Attack {
            attacker: attacker.0,
            target: target.0,
        },
    );

    log.info("Then target HP is unchanged at 100");
    assert_eq!(world.get_health(target).unwrap().current, 100);
}

use chicago_tdd_tools::{Logger, TuiBufferSink};
use proptest::prelude::*;
use wasm_game_logic::{Attack, CombatSystem, Health, World};

fn log() -> Logger {
    let mut l = Logger::new();
    let (sink, _) = TuiBufferSink::new();
    l.add_sink(Box::new(sink));
    l
}

#[test]
fn damage_scales_with_attack_stat_not_constant() {
    let log = log();
    log.info("Given two identical targets with different attacker attack stats");
    let mut world = World::new();
    let attacker = world.spawn();
    let target1 = world.spawn();
    let target2 = world.spawn();
    world.add_health(target1, Health::new(1000));
    world.add_health(target2, Health::new(1000));

    log.info("When we apply damage with attack stat 10");
    world.add_attack(
        attacker,
        Attack {
            damage: 10,
            range: 1.0,
            cooldown_ms: 0,
        },
    );
    let dmg1 = CombatSystem::apply_damage(&mut world, attacker, target1);

    log.info("And then apply damage with attack stat 50");
    *world.get_attack_mut(attacker).unwrap() = Attack {
        damage: 50,
        range: 1.0,
        cooldown_ms: 0,
    };
    let dmg2 = CombatSystem::apply_damage(&mut world, attacker, target2);

    log.info("Then damage output must differ — falsifies any constant-returning mock");
    assert_ne!(dmg1, dmg2, "damage must depend on attack stat");
    assert_eq!(dmg1, 10);
    assert_eq!(dmg2, 50);
}

#[test]
fn zero_attack_deals_zero_damage() {
    let log = log();
    log.info("Given an attacker with 0 damage and a target with 100 HP");
    let mut world = World::new();
    let attacker = world.spawn();
    let target = world.spawn();
    world.add_health(target, Health::new(100));
    world.add_attack(
        attacker,
        Attack {
            damage: 0,
            range: 1.0,
            cooldown_ms: 0,
        },
    );

    log.info("When apply_damage is called");
    let dmg = CombatSystem::apply_damage(&mut world, attacker, target);

    log.info("Then damage dealt is 0 and target HP is unchanged");
    assert_eq!(dmg, 0);
    assert_eq!(world.get_health(target).unwrap().current, 100);
}

#[test]
fn no_attack_component_deals_zero_damage() {
    let log = log();
    log.info("Given an attacker with no Attack component and a target with 100 HP");
    let mut world = World::new();
    let attacker = world.spawn();
    let target = world.spawn();
    world.add_health(target, Health::new(100));

    log.info("When apply_damage is called");
    let dmg = CombatSystem::apply_damage(&mut world, attacker, target);

    log.info("Then damage dealt is 0");
    assert_eq!(dmg, 0);
}

#[test]
fn health_cannot_go_below_zero() {
    let log = log();
    log.info("Given a Health component with 10 HP");
    let mut hp = Health::new(10);

    log.info("When 1000 damage is applied");
    hp.apply_damage(1000);

    log.info("Then current HP is clamped to 0 and is_dead returns true");
    assert_eq!(hp.current, 0);
    assert!(hp.is_dead());
}

#[test]
fn health_cannot_exceed_max_on_heal() {
    let log = log();
    log.info("Given a full-health Health component with max 100");
    let mut hp = Health::new(100);

    log.info("When we heal by 10000");
    hp.heal(10000);

    log.info("Then current HP is still capped at max");
    assert_eq!(hp.current, 100);
}

#[test]
fn health_percentage_at_full() {
    let log = log();
    log.info("Given a full-health Health component");
    let hp = Health::new(100);

    log.info("When we check the percentage");
    log.info("Then it is 1.0");
    assert!((hp.percentage() - 1.0).abs() < f32::EPSILON);
}

#[test]
fn health_percentage_at_zero() {
    let log = log();
    log.info("Given a Health component with 100 max HP");
    let mut hp = Health::new(100);

    log.info("When 100 damage is applied");
    hp.apply_damage(100);

    log.info("Then percentage is 0.0");
    assert!((hp.percentage() - 0.0).abs() < f32::EPSILON);
}

#[test]
fn dead_entities_are_removed_by_cleanup() {
    let log = log();
    log.info("Given a World with an attacker (10 damage) and a target (1 HP)");
    let mut world = World::new();
    let attacker = world.spawn();
    let target = world.spawn();
    world.add_health(target, Health::new(1));
    world.add_attack(
        attacker,
        Attack {
            damage: 10,
            range: 1.0,
            cooldown_ms: 0,
        },
    );

    log.info("When damage is applied and the target dies");
    CombatSystem::apply_damage(&mut world, attacker, target);
    assert!(world.get_health(target).unwrap().is_dead());

    log.info("And run_cleanup is called");
    CombatSystem::run_cleanup(&mut world);

    log.info("Then the dead target is removed and only the attacker remains");
    assert!(!world.is_alive(target));
    assert_eq!(world.entity_count(), 1);
}

// ── Property-based tests ──────────────────────────────────────────────────────

proptest! {
    #[test]
    fn health_current_never_exceeds_max(max in 1u32..10000, damage in 0u32..10000) {
        let mut hp = Health::new(max);
        hp.apply_damage(damage);
        prop_assert!(hp.current <= hp.max);
        prop_assert!(hp.percentage() >= 0.0);
        prop_assert!(hp.percentage() <= 1.0);
    }

    #[test]
    fn health_is_dead_iff_current_is_zero(max in 1u32..10000, damage in 0u32..50000) {
        let mut hp = Health::new(max);
        hp.apply_damage(damage);
        let dead = hp.is_dead();
        prop_assert_eq!(dead, hp.current == 0);
    }

    #[test]
    fn damage_applied_reduces_health_by_exact_amount(
        max_hp in 1u32..10000,
        damage in 1u32..1000,
    ) {
        // Only interesting when damage < max_hp (otherwise clamped).
        prop_assume!(damage < max_hp);
        let mut hp = Health::new(max_hp);
        hp.apply_damage(damage);
        prop_assert_eq!(hp.current, max_hp - damage);
    }

    #[test]
    fn heal_after_damage_bounded_by_max(
        max_hp in 1u32..10000,
        damage in 1u32..10000,
        heal in 0u32..10000,
    ) {
        let mut hp = Health::new(max_hp);
        hp.apply_damage(damage);
        hp.heal(heal);
        prop_assert!(hp.current <= hp.max);
    }
}

#[test]
fn combat_range_check_enforced() {
    use wasm_game_logic::Position;
    let mut world = World::new();
    let attacker = world.spawn();
    let target = world.spawn();
    world.add_health(target, Health::new(100));
    world.add_attack(
        attacker,
        Attack {
            damage: 15,
            range: 10.0,
            cooldown_ms: 0,
        },
    );

    // Attacker at (0, 0), Target at (15, 0) -> distance 15.0 > range 10.0
    world.add_position(attacker, Position { x: 0.0, y: 0.0 });
    world.add_position(target, Position { x: 15.0, y: 0.0 });

    let dmg = CombatSystem::apply_damage(&mut world, attacker, target);
    assert_eq!(dmg, 0, "No damage should be dealt when target is out of range");
    assert_eq!(world.get_health(target).unwrap().current, 100);

    // Target moves to (8, 0) -> distance 8.0 <= range 10.0
    *world.get_position_mut(target).unwrap() = Position { x: 8.0, y: 0.0 };

    let dmg2 = CombatSystem::apply_damage(&mut world, attacker, target);
    assert_eq!(dmg2, 15, "Damage should be dealt when target is in range");
    assert_eq!(world.get_health(target).unwrap().current, 85);
}

#[test]
fn combat_cooldown_enforced() {
    let mut world = World::new();
    let attacker = world.spawn();
    let target = world.spawn();
    world.add_health(target, Health::new(100));
    world.add_attack(
        attacker,
        Attack {
            damage: 10,
            range: 100.0,
            cooldown_ms: 100,
        },
    );

    // First attack at t=0
    world.current_time_ms = 0;
    let dmg1 = CombatSystem::apply_damage(&mut world, attacker, target);
    assert_eq!(dmg1, 10, "First attack should succeed");
    assert_eq!(world.get_health(target).unwrap().current, 90);

    // Second attack at t=50 (cooldown is 100ms)
    world.current_time_ms = 50;
    let dmg2 = CombatSystem::apply_damage(&mut world, attacker, target);
    assert_eq!(dmg2, 0, "Second attack should be blocked by cooldown");
    assert_eq!(world.get_health(target).unwrap().current, 90);

    // Third attack at t=100
    world.current_time_ms = 100;
    let dmg3 = CombatSystem::apply_damage(&mut world, attacker, target);
    assert_eq!(dmg3, 10, "Third attack should succeed after cooldown expires");
    assert_eq!(world.get_health(target).unwrap().current, 80);
}

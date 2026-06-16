use proptest::prelude::*;
use nexus_ecs::{world::GameWorld, components::*, systems::*, scheduler::SystemScheduler};
use hecs::World;

#[test]
fn spawn_player_has_all_components() {
    let mut world = GameWorld::new();
    let entity = world.spawn_player(1, "Amuro", 100.0, 500);

    assert!(world.inner.get::<&Health>(entity).is_ok());
    assert!(world.inner.get::<&Gold>(entity).is_ok());
    assert!(world.inner.get::<&PlayerControlled>(entity).is_ok());
    assert!(
        !world.inner.get::<&AiControlled>(entity).is_ok(),
        "player should not be AI-controlled"
    );
}

#[test]
fn spawn_enemy_has_ai_component() {
    let mut world = GameWorld::new();
    let entity = world.spawn_enemy(1, "StoneTitan", 200.0, 30.0, 1);
    assert!(world.inner.get::<&AiState>(entity).is_ok());
    assert!(world.inner.get::<&AiControlled>(entity).is_ok());
}

#[test]
fn health_take_damage_never_goes_negative() {
    let mut hp = Health::new(100.0);
    hp.take_damage(9999.0);
    assert_eq!(hp.current, 0.0);
    assert!(!hp.is_alive());
}

#[test]
fn despawn_dead_removes_entities() {
    let mut world = GameWorld::new();
    let _alive = world.spawn_player(1, "Alive", 100.0, 0);
    let dead_entity = world.spawn_enemy(1, "Dead", 100.0, 30.0, 1);
    // Kill the enemy directly
    world.inner.get::<&mut Health>(dead_entity).unwrap().current = 0.0;

    let despawned = world.despawn_dead();
    assert_eq!(despawned.len(), 1);
    assert!(
        world.inner.get::<&Health>(dead_entity).is_err(),
        "dead entity should be gone"
    );
}

#[test]
fn trans_am_expires_after_correct_turns() {
    let mut world = GameWorld::new();
    let entity = world.spawn_player(1, "Setsuna", 100.0, 0);
    world.attach_suit(
        entity,
        "00-Raiser".to_string(),
        GundamSeries::DoubleO,
        SpecialAbility::TransAm,
    );

    // Activate Trans-Am for 3 turns
    let mut suit = world.inner.get::<&mut MobileSuit>(entity).unwrap();
    suit.is_trans_am_active = true;
    suit.trans_am_turns_remaining = 3;
    drop(suit);

    let mut ecs_world = world.inner;
    system_tick_trans_am(&mut ecs_world);
    system_tick_trans_am(&mut ecs_world);
    system_tick_trans_am(&mut ecs_world);
    // After 3 ticks, turns_remaining reaches 0 but still active
    system_tick_trans_am(&mut ecs_world);
    // After 4th tick (when remaining was 0 at start of tick), should deactivate
    let suit = ecs_world.get::<&MobileSuit>(entity).unwrap();
    assert!(
        !suit.is_trans_am_active,
        "Trans-Am should expire after turns run out"
    );
}

#[test]
fn projectile_expires_after_lifetime() {
    let mut world = World::new();
    world.spawn((
        Projectile {
            owner_id: 1,
            damage: 50.0,
            lifetime_remaining: 0.5,
            magic_type: MagicType::BeamSaber,
        },
        Position::default(),
        Velocity { dx: 1.0, dy: 0.0, dz: 0.0 },
    ));
    assert_eq!(world.len(), 1);
    system_tick_projectiles(&mut world, 0.6); // dt > lifetime
    assert_eq!(world.len(), 0, "expired projectile should be despawned");
}

#[test]
fn scheduler_runs_full_tick_without_panic() {
    let mut world = GameWorld::new();
    let _p = world.spawn_player(1, "Setsuna", 100.0, 0);
    let _e = world.spawn_enemy(1, "Titan", 200.0, 20.0, 1);
    world.spawn_projectile(
        1,
        Position::default(),
        Velocity { dx: 1.0, dy: 0.0, dz: 0.0 },
        30.0,
        MagicType::Fire,
    );
    let scheduler = SystemScheduler::new(0.016);
    scheduler.tick(&mut world.inner);
}

proptest! {
    // Health can never be negative
    #[test]
    fn health_never_negative(max in 1.0f32..10_000.0, damage in 0.0f32..100_000.0) {
        let mut hp = Health::new(max);
        hp.take_damage(damage);
        prop_assert!(hp.current >= 0.0);
        prop_assert!(hp.current <= max);
    }

    // Spawning N players creates exactly N entities
    #[test]
    fn spawn_n_players_gives_n_entities(n in 1usize..20) {
        let mut world = GameWorld::new();
        for i in 0..n {
            world.spawn_player(i as u64, "test", 100.0, 0);
        }
        prop_assert_eq!(world.entity_count() as usize, n);
    }
}

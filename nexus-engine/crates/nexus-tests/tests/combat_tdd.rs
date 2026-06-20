//! TDD-style tests for combat damage formulas using chicago-tdd-tools.
//!
//! Tests cover:
//! 1. UseSpecial — higher ability_id → more damage
//! 2. CastMagic — different magic types → different damage
//! 3. StatType::Magic allocation → increments player.magic by 5
//! 4. Higher magic stat → more magic damage

use chicago_tdd_tools::TestEnvironment;
use nexus_integration::game_loop::{GameCommand, GameSession, StatType};
use nexus_net::{
    protocol::{CombatAction, CombatOutcome},
    room::{GameRoom, RoomPlayer, RoomState},
};
use nexus_types::{Damage, Hp, MagicType};

// ── Helpers ───────────────────────────────────────────────────────────────────

fn make_room_player(id: u64, hp: f32, attack: f32, magic: f32) -> RoomPlayer {
    RoomPlayer {
        player_id: id,
        name: format!("Player{}", id),
        suit_id: "RX-78-2".to_string(),
        hp: Hp::new(hp),
        max_hp: Hp::new(hp),
        attack: Damage::new(attack),
        magic: Damage::new(magic),
        combo_depth: 0,
    }
}

fn active_room(p1: RoomPlayer, p2: RoomPlayer) -> GameRoom {
    let mut room = GameRoom::new(1, p1, p2);
    room.state = RoomState::Active;
    room
}

fn extract_damage(outcome: &CombatOutcome) -> f32 {
    match outcome {
        CombatOutcome::Hit { damage, .. } => *damage,
        CombatOutcome::PlayerDied { .. } => f32::MAX, // player died — treat as max damage
        other => panic!("Expected Hit or PlayerDied, got {:?}", other),
    }
}

// ── Test 1: UseSpecial — higher ability_id deals more damage ─────────────────

#[test]
fn use_special_higher_ability_id_deals_more_damage() {
    let _env = TestEnvironment::new().expect("failed to create test environment");

    // Both players share the same stats; only ability_id differs.
    let attack = 30.0_f32;

    let mut room_low = active_room(
        make_room_player(1, 1000.0, attack, 50.0),
        make_room_player(2, 1000.0, attack, 50.0),
    );
    let outcome_low = room_low
        .apply_action(1, CombatAction::UseSpecial { ability_id: 1 })
        .expect("action should succeed");

    let mut room_high = active_room(
        make_room_player(1, 1000.0, attack, 50.0),
        make_room_player(2, 1000.0, attack, 50.0),
    );
    let outcome_high = room_high
        .apply_action(1, CombatAction::UseSpecial { ability_id: 5 })
        .expect("action should succeed");

    let dmg_low = extract_damage(&outcome_low);
    let dmg_high = extract_damage(&outcome_high);

    assert!(
        dmg_high > dmg_low,
        "ability_id=5 should deal more damage than ability_id=1, got {} vs {}",
        dmg_high,
        dmg_low,
    );

    // Verify the formula: attack * 2.0 + ability_id * 5.0
    let expected_low = attack * 2.0 + 1.0 * 5.0;
    let expected_high = attack * 2.0 + 5.0 * 5.0;
    assert_eq!(
        dmg_low, expected_low,
        "UseSpecial formula mismatch for ability_id=1"
    );
    assert_eq!(
        dmg_high, expected_high,
        "UseSpecial formula mismatch for ability_id=5"
    );
}

// ── Test 2: CastMagic — different magic types deal different damage ───────────

#[test]
fn cast_magic_different_types_deal_different_damage() {
    let _env = TestEnvironment::new().expect("failed to create test environment");

    let magic_stat = 50.0_f32;

    // magic_type=0 (Fire, +20) vs magic_type=3 (Dark, +35)
    let mut room_fire = active_room(
        make_room_player(1, 1000.0, 30.0, magic_stat),
        make_room_player(2, 1000.0, 30.0, magic_stat),
    );
    let outcome_fire = room_fire
        .apply_action(
            1,
            CombatAction::CastMagic {
                magic_type: MagicType::Fire,
            },
        )
        .expect("fire cast should succeed");

    let mut room_dark = active_room(
        make_room_player(1, 1000.0, 30.0, magic_stat),
        make_room_player(2, 1000.0, 30.0, magic_stat),
    );
    let outcome_dark = room_dark
        .apply_action(
            1,
            CombatAction::CastMagic {
                magic_type: MagicType::Dark,
            },
        )
        .expect("dark cast should succeed");

    let dmg_fire = extract_damage(&outcome_fire);
    let dmg_dark = extract_damage(&outcome_dark);

    assert!(
        dmg_dark > dmg_fire,
        "Dark magic (type=3) should deal more damage than Fire (type=0), got {} vs {}",
        dmg_dark,
        dmg_fire,
    );

    // Verify exact formula: magic * 1.5 + type_bonus
    let expected_fire = magic_stat * 1.5 + 20.0; // Fire bonus = 20
    let expected_dark = magic_stat * 1.5 + 35.0; // Dark bonus = 35
    assert_eq!(dmg_fire, expected_fire, "CastMagic Fire formula mismatch");
    assert_eq!(dmg_dark, expected_dark, "CastMagic Dark formula mismatch");

    // Also verify Lightning (+30), Ice (+15), and Light (+25)
    let magic_values: &[(MagicType, f32, &str)] = &[
        (MagicType::Lightning, 30.0, "Lightning"),
        (MagicType::Ice, 15.0, "Ice"),
        (MagicType::Light, 25.0, "Light"),
    ];
    for &(mt, bonus, label) in magic_values {
        let mut room = active_room(
            make_room_player(1, 1000.0, 30.0, magic_stat),
            make_room_player(2, 1000.0, 30.0, magic_stat),
        );
        let outcome = room
            .apply_action(1, CombatAction::CastMagic { magic_type: mt })
            .expect("cast should succeed");
        let dmg = extract_damage(&outcome);
        let expected = magic_stat * 1.5 + bonus;
        assert_eq!(dmg, expected, "CastMagic {} formula mismatch", label);
    }
}

// ── Test 3: StatType::Magic allocation increments player.magic by 5 ──────────

#[test]
fn stat_magic_allocation_increments_magic_by_5() {
    let _env = TestEnvironment::new().expect("failed to create test environment");

    // Use a seeded session; manually grant stat points.
    let mut session = GameSession::new(1, "TestPilot", 42);

    // Grant a stat point so the allocation is accepted.
    session.player.stat_points = 1;

    let magic_before = session.player.magic;
    let events = session.dispatch(GameCommand::AllocateStat(StatType::Magic));

    assert_eq!(
        session.player.magic,
        magic_before + 5,
        "allocating Magic stat should increase player.magic by 5"
    );
    assert_eq!(
        session.player.stat_points, 0,
        "stat_points should be consumed"
    );
    assert!(
        events.iter().any(|e| matches!(
            e.kind,
            nexus_integration::game_loop::EventKind::StatAllocated(StatType::Magic)
        )),
        "StatAllocated(Magic) event should be emitted"
    );
}

// ── Test 4: Higher magic stat → more magic damage ────────────────────────────

#[test]
fn higher_magic_stat_deals_more_magic_damage() {
    let _env = TestEnvironment::new().expect("failed to create test environment");

    // Two otherwise identical rooms; player1 differs only in magic stat.
    let low_magic = 20.0_f32;
    let high_magic = 80.0_f32;
    let magic_type = MagicType::Lightning; // Lightning (+30)

    let mut room_low = active_room(
        make_room_player(1, 1000.0, 30.0, low_magic),
        make_room_player(2, 1000.0, 30.0, 50.0),
    );
    let outcome_low = room_low
        .apply_action(1, CombatAction::CastMagic { magic_type })
        .expect("cast should succeed");

    let mut room_high = active_room(
        make_room_player(1, 1000.0, 30.0, high_magic),
        make_room_player(2, 1000.0, 30.0, 50.0),
    );
    let outcome_high = room_high
        .apply_action(1, CombatAction::CastMagic { magic_type })
        .expect("cast should succeed");

    let dmg_low = extract_damage(&outcome_low);
    let dmg_high = extract_damage(&outcome_high);

    assert!(
        dmg_high > dmg_low,
        "player with magic={} should deal more damage than magic={}, got {} vs {}",
        high_magic,
        low_magic,
        dmg_high,
        dmg_low,
    );

    // Verify the linear scaling: difference should be exactly 1.5 * (high - low)
    let expected_diff = 1.5 * (high_magic - low_magic);
    let actual_diff = dmg_high - dmg_low;
    assert!(
        (actual_diff - expected_diff).abs() < 1e-4,
        "damage difference should scale linearly with magic stat: expected {}, got {}",
        expected_diff,
        actual_diff,
    );
}

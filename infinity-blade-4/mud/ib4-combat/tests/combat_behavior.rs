#![allow(unused_imports)]
use ib4_combat::{
    combo::ComboTracker,
    parry::{ParryResolver, ParryIntent, ParryOutcome},
    damage::{calc_player_damage, calc_enemy_damage},
};
use ib4_core::types::AttackDir;

#[test]
fn should_return_1_0x_at_combo_depth_1() {
    let mut c = ComboTracker::new(2);
    c.on_hit();
    assert_eq!(c.multiplier(), 1.0);
}

#[test]
fn should_return_1_5x_at_combo_depth_2() {
    let mut c = ComboTracker::new(2);
    c.on_hit();
    c.on_hit();
    assert_eq!(c.multiplier(), 1.5);
}

#[test]
fn should_return_3_0x_at_combo_depth_4() {
    let mut c = ComboTracker::new(2);
    for _ in 0..4 {
        c.on_hit();
    }
    assert_eq!(c.multiplier(), 3.0);
}

#[test]
fn should_reset_combo_after_2_non_attack_turns() {
    let mut c = ComboTracker::new(2);
    c.on_hit();
    c.on_hit();
    c.on_non_attack_turn();
    assert_eq!(c.depth, 2); // not reset yet
    c.on_non_attack_turn();
    assert_eq!(c.depth, 0); // now reset
}

#[test]
fn should_not_reset_combo_with_threshold_3_after_2_turns() {
    let mut c = ComboTracker::new(3);
    c.on_hit();
    c.on_hit();
    c.on_non_attack_turn();
    c.on_non_attack_turn();
    assert_eq!(c.depth, 2); // ComboMaster: 3 turns needed
}

#[test]
fn should_normal_parry_on_any_parry_intent() {
    let outcome = ParryResolver::resolve(AttackDir::Overhead, ParryIntent::AnyParry);
    assert_eq!(outcome, ParryOutcome::NormalParry);
}

#[test]
fn should_perfect_parry_when_direction_matches() {
    let outcome = ParryResolver::resolve(
        AttackDir::Left,
        ParryIntent::DirectionalParry(AttackDir::Left),
    );
    assert_eq!(outcome, ParryOutcome::PerfectParry);
}

#[test]
fn should_degrade_to_normal_parry_on_wrong_direction() {
    let outcome = ParryResolver::resolve(
        AttackDir::Overhead,
        ParryIntent::DirectionalParry(AttackDir::Right),
    );
    assert_eq!(outcome, ParryOutcome::NormalParry);
}

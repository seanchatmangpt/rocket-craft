use ib4_core::{player::PlayerState, types::MagicType};
use ib4_progression::{
    xp::{XPSystem, xp_for_level, xp_threshold, MAX_LEVEL},
    bloodline::BloodlineSystem,
    perks::PerkTree,
};

#[test]
fn should_compute_xp_for_level_1_as_100() {
    assert_eq!(xp_for_level(1), 100);
}

#[test]
fn should_level_up_when_xp_crosses_threshold() {
    let mut p = PlayerState::new("Siris");
    let events = XPSystem::add_xp(&mut p, xp_threshold(1), 1.0);
    assert_eq!(events.len(), 1);
    assert_eq!(p.level, 2);
    assert_eq!(p.stat_points, 2);
}

#[test]
fn should_scale_xp_by_multiplier() {
    let mut p = PlayerState::new("Siris");
    let events = XPSystem::add_xp(&mut p, 50, 2.0); // 50 raw × 2.0 = 100 = level threshold
    assert_eq!(events.len(), 1);
}

#[test]
fn should_not_exceed_level_cap() {
    let mut p = PlayerState::new("Siris");
    p.level = MAX_LEVEL;
    p.xp = xp_threshold(MAX_LEVEL) * 100;
    let events = XPSystem::add_xp(&mut p, 999999, 1.0);
    assert_eq!(events.len(), 0);
    assert_eq!(p.level, MAX_LEVEL);
}

#[test]
fn should_rebirth_reset_gold_and_equipment() {
    let mut p = PlayerState::new("Siris");
    p.gold = 5000;
    p.weapon = Some(ib4_core::equipment::Weapon::starter());
    BloodlineSystem::trigger_rebirth(&mut p);
    assert_eq!(p.gold, 0);
    assert!(p.weapon.is_none());
}

#[test]
fn should_rebirth_preserve_level_and_xp() {
    let mut p = PlayerState::new("Siris");
    p.level = 10;
    p.xp = 5000;
    BloodlineSystem::trigger_rebirth(&mut p);
    assert_eq!(p.level, 10);
    assert_eq!(p.xp, 5000);
}

#[test]
fn should_unlock_lightning_at_bloodline_3() {
    let mut p = PlayerState::new("Siris");
    p.bloodline = 2;
    BloodlineSystem::trigger_rebirth(&mut p); // now at BL3
    assert!(p.magic_unlocks.iter().any(|m| m == &MagicType::Lightning));
}

#[test]
fn should_deny_perk_without_prereq() {
    let mut p = PlayerState::new("Siris");
    p.perk_points = 5;
    let tree = PerkTree::new();
    let result = tree.select_perk(&mut p, "DeadlyPrecision"); // requires BloodyResolve
    assert!(result.is_err());
}

#[test]
fn should_deny_tier2_perk_below_bloodline_5() {
    let mut p = PlayerState::new("Siris");
    p.perk_points = 5;
    p.bloodline = 3;
    // First get the prereq
    let _ = tree_with_point(&mut p, "BloodyResolve");
    let tree = PerkTree::new();
    let result = tree.select_perk(&mut p, "DeadlyPrecision");
    assert!(result.is_err());
}

#[test]
fn should_select_tier1_perk_with_point() {
    let mut p = PlayerState::new("Siris");
    p.perk_points = 1;
    let tree = PerkTree::new();
    let result = tree.select_perk(&mut p, "BloodyResolve");
    assert!(result.is_ok());
    assert_eq!(p.perk_points, 0);
    assert!(p.selected_perks.iter().any(|perk| perk == "BloodyResolve"));
}

#[test]
fn should_aggregate_perks_correctly() {
    let tree = PerkTree::new();
    let selected = vec!["BloodyResolve".to_string(), "IronHide".to_string()];
    let agg = tree.compute_aggregate(&selected);
    assert!((agg.attack_mult - 1.10).abs() < 0.001);
    assert!((agg.defense_mult - 1.10).abs() < 0.001);
}

fn tree_with_point(p: &mut PlayerState, perk_id: &str) {
    p.perk_points += 1;
    let _ = PerkTree::new().select_perk(p, perk_id);
}

use ib4_ai::{
    godking::{GodKingAI, GodKingEvent},
    roster::{arena_sequence, spawn_enemy},
    titan::TitanAI,
};
use ib4_core::{enemy::EnemyInstance, player::PlayerState};

fn make_enemy(id: &str) -> EnemyInstance {
    spawn_enemy(id, 0).expect("valid enemy id")
}

#[test]
fn should_find_all_15_enemies() {
    use ib4_ai::roster::all_enemies;
    assert_eq!(all_enemies().len(), 15);
}

#[test]
fn should_scale_hp_with_bloodline() {
    let e0 = make_enemy("LightTitan");
    let e1 = spawn_enemy("LightTitan", 1).unwrap();
    assert!(e1.base_hp > e0.base_hp);
}

#[test]
fn should_arena_sequence_end_with_corrupted_galath() {
    let seq = arena_sequence(0);
    assert_eq!(*seq.last().unwrap(), "CorruptedGalath");
}

#[test]
fn should_arena_sequence_have_4_enemies() {
    let seq = arena_sequence(5);
    assert_eq!(seq.len(), 4);
}

#[test]
fn should_titan_phase2_transition_at_60_percent() {
    let mut e = make_enemy("LightTitan");
    e.current_hp = e.base_hp * 0.59; // just below 60%
    let transition = TitanAI::check_phase_transition(&mut e);
    assert_eq!(transition, Some(2));
    assert_eq!(e.phase, 2);
}

#[test]
fn should_titan_phase3_transition_at_30_percent() {
    let mut e = make_enemy("LightTitan");
    e.phase = 2;
    e.current_hp = e.base_hp * 0.29;
    let transition = TitanAI::check_phase_transition(&mut e);
    assert_eq!(transition, Some(3));
}

#[test]
fn should_not_retransition_to_lower_phase() {
    let mut e = make_enemy("LightTitan");
    e.phase = 3;
    e.current_hp = e.base_hp * 0.50; // would be phase 2, but already phase 3
    let transition = TitanAI::check_phase_transition(&mut e);
    assert!(transition.is_none());
}

#[test]
fn should_godking_shield_break_after_3_perfect_parries() {
    let mut e = spawn_enemy("CorruptedGalath", 0).unwrap();
    e.shield_active = true;
    e.perfect_parries_received = 0;
    let r1 = GodKingAI::register_perfect_parry(&mut e);
    assert!(r1.is_none());
    let r2 = GodKingAI::register_perfect_parry(&mut e);
    assert!(r2.is_none());
    let r3 = GodKingAI::register_perfect_parry(&mut e);
    assert!(matches!(r3, Some(GodKingEvent::ShieldBroken)));
    assert!(!e.shield_active);
    assert_eq!(e.phase, 2);
}

#[test]
fn should_godking_qip_scar_force_rebirth_at_3_stacks() {
    let mut p = PlayerState::new("Siris");
    p.qip_scar_stacks = 0;
    let r1 = GodKingAI::apply_qip_scar(&mut p);
    assert!(r1.is_none());
    let r2 = GodKingAI::apply_qip_scar(&mut p);
    assert!(r2.is_none());
    let r3 = GodKingAI::apply_qip_scar(&mut p);
    assert!(matches!(r3, Some(GodKingEvent::ForcedRebirth)));
    assert_eq!(p.qip_scar_stacks, 0); // reset after forced rebirth
}

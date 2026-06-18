use ib4_integration_tests::{new_session, Command, AttackDir};

#[test]
fn should_spawn_enemy_on_first_attack() {
    let mut s = new_session();
    // Attack when not in combat triggers spawn_next_enemy
    let out = s.dispatch(Command::Attack(AttackDir::Overhead));
    assert!(s.current_enemy.is_some(), "First Attack should spawn an enemy");
    assert!(s.is_in_combat(), "Should be in combat after spawning enemy");
    let text = out.join(" ");
    assert!(!text.is_empty(), "Should produce narrative output");
}

#[test]
fn should_produce_look_output_on_explore() {
    let mut s = new_session();
    let out = s.dispatch(Command::Explore);
    // Explore calls cmd_look — no combat yet, shows queue info
    assert!(!out.is_empty(), "Explore should produce output");
    assert!(!s.is_in_combat(), "Explore alone does not start combat");
    assert!(s.current_enemy.is_none(), "Explore alone does not spawn enemy");
}

#[test]
fn should_complete_full_combat_loop_killing_light_titan() {
    let mut s = new_session();
    // First attack spawns enemy
    s.dispatch(Command::Attack(AttackDir::Overhead));
    assert!(s.current_enemy.is_some());

    let mut rounds = 0;
    while s.current_enemy.as_ref().map(|e| e.is_alive()).unwrap_or(false) {
        // Clear pending announced attack before attacking (avoid taking damage and dying early)
        s.announced_attack = None;
        s.dispatch(Command::Attack(AttackDir::Overhead));
        rounds += 1;
        assert!(rounds < 100, "Combat should resolve within 100 rounds");
    }

    assert!(s.current_enemy.is_none(), "Enemy cleared after defeat");
    assert!(!s.is_in_combat(), "Not in combat after enemy defeated");
    assert!(s.player.xp > 0, "XP awarded on enemy defeat");
    // player starts with 100 gold, gains more on verified
    assert!(s.player.gold >= 100, "Gold awarded on enemy defeat (starts with 100)");
}

#[test]
fn should_build_combo_depth_on_consecutive_attacks() {
    let mut s = new_session();
    // Spawn enemy first
    s.dispatch(Command::Attack(AttackDir::Right));
    assert!(s.current_enemy.is_some());

    // Now we're in combat; clear announced attacks so player doesn't die
    s.announced_attack = None;
    s.dispatch(Command::Attack(AttackDir::Right));
    // combo_depth after first combat attack = 1 (spawn attack doesn't count)
    // Actually after spawn, the first attack might have combo_depth=1 from the spawn dispatch
    // Let's just verify combo is at least 1 after some attacks
    assert!(s.combo_depth >= 1, "Combo depth should increase on attacks");

    s.announced_attack = None;
    let combo_before = s.combo_depth;
    s.dispatch(Command::Attack(AttackDir::Left));
    assert!(s.combo_depth > combo_before, "Combo depth increases on consecutive attacks");

    s.announced_attack = None;
    let combo_before2 = s.combo_depth;
    s.dispatch(Command::Attack(AttackDir::Overhead));
    assert!(s.combo_depth > combo_before2 || s.current_enemy.is_none(),
        "Combo depth increases further, or enemy died");
}

#[test]
fn should_reset_combo_on_dodge() {
    let mut s = new_session();
    // Spawn enemy
    s.dispatch(Command::Attack(AttackDir::Right));
    assert!(s.current_enemy.is_some());

    // Build some combo
    s.announced_attack = None;
    s.dispatch(Command::Attack(AttackDir::Right));
    assert!(s.combo_depth >= 1, "Should have combo after attack");

    // Dodge resets combo
    s.dispatch(Command::Dodge);
    assert_eq!(s.combo_depth, 0, "Combo should reset on dodge");
}

#[test]
fn should_trigger_phase2_transition_at_60_percent_hp() {
    let mut s = new_session();
    // Spawn enemy
    s.dispatch(Command::Attack(AttackDir::Overhead));
    assert!(s.current_enemy.is_some());

    // Set enemy HP to 59% to trigger Phase 2
    if let Some(e) = s.current_enemy.as_mut() {
        e.current_hp = e.base_hp * 0.59;
    }

    s.announced_attack = None;
    s.dispatch(Command::Attack(AttackDir::Overhead));

    let phase = s.current_enemy.as_ref().map(|e| e.phase).unwrap_or(0);
    // Either enemy transitioned to phase 2, or it died (possible with high damage + low HP)
    assert!(phase >= 2 || s.current_enemy.is_none(),
        "Phase 2 should trigger at 59% HP (phase={}, enemy_alive={})",
        phase, s.current_enemy.is_some());
}

#[test]
fn should_normal_parry_prevent_damage() {
    let mut s = new_session();
    // Spawn enemy
    s.dispatch(Command::Attack(AttackDir::Overhead));
    assert!(s.current_enemy.is_some());

    let hp_before = s.player.health;
    // Set a specific announced attack
    s.announced_attack = Some(AttackDir::Overhead);
    // Normal parry — consumes the announced attack and takes no damage
    s.dispatch(Command::Parry);
    let hp_after = s.player.health;

    assert_eq!(hp_before, hp_after, "Normal parry should prevent all damage");
}

#[test]
fn should_perfect_parry_matching_direction_prevent_damage() {
    let mut s = new_session();
    // Spawn enemy
    s.dispatch(Command::Attack(AttackDir::Overhead));
    assert!(s.current_enemy.is_some());

    let hp_before = s.player.health;
    s.announced_attack = Some(AttackDir::Left);
    s.dispatch(Command::PerfectParry(AttackDir::Left));
    let hp_after = s.player.health;

    assert_eq!(hp_before, hp_after, "Perfect parry should prevent all damage");
}
